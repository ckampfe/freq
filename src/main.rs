use bumpalo::Bump;
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use bstr::io::BufReadExt;
use clap::{Parser, ValueEnum};
use core::fmt::NumBuffer;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

#[derive(Parser)]
/// Count how many times each line occurs in the input, sorted by count.
struct Options {
    /// Input file; `-` or absent reads standard input
    input_file: Option<PathBuf>,
    /// Output file; absent writes to standard output
    output_file: Option<PathBuf>,
    /// The order to sort the output by count
    #[arg(value_enum, short, long, default_value_t)]
    order: Order,
}

#[derive(Clone, Default, ValueEnum)]
enum Order {
    Asc,
    #[default]
    Desc,
}

fn main() -> std::io::Result<()> {
    // https://github.com/rust-lang/rust/issues/46016
    #[cfg(target_family = "unix")]
    {
        use nix::sys::signal;
        let _ = unsafe { signal::signal(signal::Signal::SIGPIPE, signal::SigHandler::SigDfl)? };
    }

    let options = Options::parse();

    let mut input: Box<dyn BufRead> = match options.input_file {
        Some(path) if path.as_os_str() != "-" => Box::new(BufReader::new(File::open(path)?)),
        _ => Box::new(std::io::stdin().lock()),
    };

    let lines_arena = Bump::new();

    let mut frequencies: HashMap<&[u8], usize> = HashMap::default();

    input.for_byte_line(|line| {
        match frequencies.get_mut(line) {
            Some(count) => *count += 1,
            None => {
                let line_ref = lines_arena.alloc_slice_copy(line);
                frequencies.insert(line_ref, 1);
            }
        }
        Ok(true)
    })?;

    let inner: Box<dyn Write> = match options.output_file {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(std::io::stdout().lock()),
    };

    let mut out_writer = BufWriter::new(inner);

    let mut frequencies_sorted: Vec<_> = frequencies.into_iter().collect();

    match options.order {
        Order::Asc => {
            frequencies_sorted.sort_by_key(|el| el.1);
        }
        Order::Desc => {
            frequencies_sorted.sort_by_key(|el| std::cmp::Reverse(el.1));
        }
    }

    let mut num_buffer = NumBuffer::new();

    for (line, count) in frequencies_sorted {
        out_writer.write_all(count.format_into(&mut num_buffer).as_bytes())?;
        out_writer.write_all(b" ")?;
        out_writer.write_all(line)?;
        out_writer.write_all(b"\n")?;
    }

    out_writer.flush()
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};

    /// Path to the `freq` binary. `cargo test` on a binary crate compiles this
    /// file as a test harness but does not build the binary itself, so build it
    /// once, into the same profile directory this test is running out of.
    fn freq_bin() -> PathBuf {
        static BUILD: std::sync::Once = std::sync::Once::new();

        BUILD.call_once(|| {
            let mut build = Command::new(env!("CARGO"));
            build
                .args(["build", "--quiet", "--bin", "freq"])
                .current_dir(env!("CARGO_MANIFEST_DIR"));

            if !cfg!(debug_assertions) {
                build.arg("--release");
            }

            assert!(build.status().unwrap().success(), "cargo build failed");
        });

        let mut path = std::env::current_exe().unwrap();
        path.pop();
        if path.ends_with("deps") {
            path.pop();
        }
        path.join("freq")
    }

    /// Run `freq` with `args`, feeding `input` on stdin, and return its stdout.
    fn freq(input: &str, args: &[&str]) -> String {
        let mut child = Command::new(freq_bin())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();

        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "freq failed: {:?}", output.status);

        String::from_utf8(output.stdout).unwrap()
    }

    /// A scratch path in the temp dir, unique to this test process and `tag`.
    fn temp_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("freq-{tag}-{}.txt", std::process::id()))
    }

    /// Split each `<count> <line>` output line back into its two halves.
    fn parse(output: &str) -> Vec<(usize, String)> {
        output
            .lines()
            .map(|out_line| {
                let (count, line) = out_line
                    .split_once(' ')
                    .unwrap_or_else(|| panic!("malformed output line: {out_line:?}"));
                (count.parse().unwrap(), line.to_string())
            })
            .collect()
    }

    /// Assert that the counts are exactly `expected`, in some order consistent
    /// with `order`. Lines that share a count may come out in any order relative
    /// to one another, so only the count sequence is checked for ordering.
    fn assert_counts(output: &str, order: Order, expected: &[(&str, usize)]) {
        let parsed = parse(output);

        let mut got: Vec<(String, usize)> = parsed
            .iter()
            .map(|(count, line)| (line.clone(), *count))
            .collect();
        got.sort();

        let mut want: Vec<(String, usize)> = expected
            .iter()
            .map(|(line, count)| (line.to_string(), *count))
            .collect();
        want.sort();

        assert_eq!(got, want, "counts differ (output: {output:?})");

        let counts: Vec<usize> = parsed.iter().map(|(count, _)| *count).collect();
        let sorted = match order {
            Order::Asc => counts.windows(2).all(|w| w[0] <= w[1]),
            Order::Desc => counts.windows(2).all(|w| w[0] >= w[1]),
        };
        assert!(sorted, "counts not sorted: {counts:?} (output: {output:?})");
    }

    #[test]
    fn counts_repeated_lines() {
        assert_counts(
            &freq("a\nb\na\nc\nb\na\n", &[]),
            Order::Desc,
            &[("a", 3), ("b", 2), ("c", 1)],
        );
    }

    /// The interesting difference from a bare `uniq`: duplicates that are not
    /// adjacent still get collapsed into one count.
    #[test]
    fn counts_non_adjacent_duplicates() {
        assert_counts(
            &freq("b\na\nb\na\nb\n", &[]),
            Order::Desc,
            &[("b", 3), ("a", 2)],
        );
    }

    #[test]
    fn empty_input_produces_no_output() {
        assert_eq!(freq("", &[]), "");
    }

    #[test]
    fn single_line() {
        assert_counts(&freq("only\n", &[]), Order::Desc, &[("only", 1)]);
    }

    /// A final line without a newline is still a line, and still counted.
    #[test]
    fn missing_trailing_newline() {
        assert_counts(&freq("a\nb\na", &[]), Order::Desc, &[("a", 2), ("b", 1)]);
    }

    /// Blank lines and leading whitespace are part of the line, not stripped.
    #[test]
    fn blank_and_indented_lines() {
        assert_counts(
            &freq("\n  b\na\n\n  b\n", &[]),
            Order::Desc,
            &[("", 2), ("  b", 2), ("a", 1)],
        );
    }

    #[test]
    fn non_ascii_lines() {
        assert_counts(
            &freq("é\nz\né\nこんにちは\na\n", &[]),
            Order::Desc,
            &[("é", 2), ("z", 1), ("こんにちは", 1), ("a", 1)],
        );
    }

    /// Lines that look like numbers are counted as text, not compared as numbers.
    #[test]
    fn numeric_looking_lines() {
        assert_counts(
            &freq("10\n2\n10\n02\n2\n10\n", &[]),
            Order::Desc,
            &[("10", 3), ("2", 2), ("02", 1)],
        );
    }

    /// Counting is case sensitive: `Apple` and `apple` are distinct lines.
    #[test]
    fn case_sensitive() {
        assert_counts(
            &freq("Apple\napple\nApple\n", &[]),
            Order::Desc,
            &[("Apple", 2), ("apple", 1)],
        );
    }

    #[test]
    fn descending_is_the_default_order() {
        let counts: Vec<usize> = parse(&freq("a\nb\nb\nc\nc\nc\n", &[]))
            .iter()
            .map(|(count, _)| *count)
            .collect();

        assert_eq!(counts, [3, 2, 1]);
    }

    #[test]
    fn ascending_order_flag() {
        for args in [&["--order", "asc"], &["-o", "asc"]] {
            let counts: Vec<usize> = parse(&freq("a\nb\nb\nc\nc\nc\n", args))
                .iter()
                .map(|(count, _)| *count)
                .collect();

            assert_eq!(counts, [1, 2, 3], "args: {args:?}");
        }
    }

    #[test]
    fn explicit_descending_order_flag() {
        let counts: Vec<usize> = parse(&freq("a\nb\nb\nc\nc\nc\n", &["--order", "desc"]))
            .iter()
            .map(|(count, _)| *count)
            .collect();

        assert_eq!(counts, [3, 2, 1]);
    }

    /// Every distinct line appears exactly once in the output, whatever the order.
    #[test]
    fn each_distinct_line_reported_once() {
        let input: String = (0..100).map(|i| format!("line {}\n", i % 7)).collect();

        let expected: Vec<(String, usize)> = (0..7)
            .map(|r| (format!("line {r}"), (0..100).filter(|i| i % 7 == r).count()))
            .collect();
        let expected: Vec<(&str, usize)> = expected
            .iter()
            .map(|(line, count)| (line.as_str(), *count))
            .collect();

        assert_counts(&freq(&input, &[]), Order::Desc, &expected);
        assert_counts(&freq(&input, &["--order", "asc"]), Order::Asc, &expected);
    }

    #[test]
    fn counts_larger_than_a_single_digit() {
        let input = "x\n".repeat(1234) + &"y\n".repeat(12);

        assert_counts(&freq(&input, &[]), Order::Desc, &[("x", 1234), ("y", 12)]);
    }

    #[test]
    fn reads_named_input_file() {
        let path = temp_path("input");
        std::fs::write(&path, "b\na\nb\n").unwrap();

        let output = freq("", &[path.to_str().unwrap()]);
        std::fs::remove_file(&path).unwrap();

        assert_counts(&output, Order::Desc, &[("b", 2), ("a", 1)]);
    }

    #[test]
    fn writes_named_output_file() {
        let path = temp_path("output");

        let stdout = freq("b\na\nb\n", &["-", path.to_str().unwrap()]);
        let written = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(&path).unwrap();

        assert_eq!(stdout, "");
        assert_counts(&written, Order::Desc, &[("b", 2), ("a", 1)]);
    }
}
