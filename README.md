# freq

Count the frequency of lines in a file, like `sort | uniq -c` but much faster.

[![Rust](https://github.com/ckampfe/freq/actions/workflows/rust.yml/badge.svg)](https://github.com/ckampfe/freq/actions/workflows/rust.yml)

## Install

```
$ RUSTFLAGS="-C target-cpu=native" cargo install --git https://github.com/ckampfe/freq
```

## Example

```sh
$ /bin/cat doc.txt
orange
banana
apple
orange
banana
orange
banana
orange
banana
apple
banana
apple

```


```sh
$ freq doc.txt
5 banana
4 orange
3 apple

$ freq --order=asc doc.txt
3 apple
4 orange
5 banana

```

Compare to the classic `sort | uniq -c`:

```sh
$ sort doc.txt | uniq -c
   3 apple
   5 banana
   4 orange

```

## Help

```
$ freq -h
Count how many times each line occurs in the input, sorted by count

Usage: freq [OPTIONS] [INPUT_FILE] [OUTPUT_FILE]

Arguments:
  [INPUT_FILE]   Input file; `-` or absent reads standard input
  [OUTPUT_FILE]  Output file; absent writes to standard output

Options:
  -o, --order <ORDER>  The order to sort the output by count [default: desc] [possible values: asc, desc]
  -h, --help           Print help
```

## Benchmarks


```
at [ 19:02:21 ] ➜ ./bench.sh
high: sort | uniq -c: 3 warmup runs
high: sort | uniq -c: real run
       16.07 real        15.89 user         0.17 sys
mid: sort | uniq -c: 3 warmup runs
mid: sort | uniq -c: real run
       12.43 real        12.26 user         0.15 sys
low: sort | uniq -c: 3 warmup runs
low: sort | uniq -c: real run
        7.76 real         7.61 user         0.14 sys
Benchmark 1: freq bench/bench-high.txt
  Time (mean ± σ):     723.6 ms ±  16.1 ms    [User: 674.7 ms, System: 47.5 ms]
  Range (min … max):   702.8 ms … 743.7 ms    9 runs

Benchmark 1: freq bench/bench-mid.txt
  Time (mean ± σ):     233.5 ms ±   4.7 ms    [User: 209.8 ms, System: 23.0 ms]
  Range (min … max):   229.0 ms … 241.3 ms    9 runs

Benchmark 1: freq bench/bench-low.txt
  Time (mean ± σ):     199.6 ms ±   0.9 ms    [User: 178.3 ms, System: 20.8 ms]
  Range (min … max):   198.6 ms … 200.8 ms    9 runs
```
