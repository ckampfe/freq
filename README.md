# freq

Count the occurrences of lines in a file, like `sort | uniq -c` but much faster.

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
$ ./bench.sh
high: sort | uniq -c: 3 warmup runs
high: sort | uniq -c: real run
       15.99 real        15.81 user         0.16 sys
mid: sort | uniq -c: 3 warmup runs
mid: sort | uniq -c: real run
       12.46 real        12.30 user         0.15 sys
low: sort | uniq -c: 3 warmup runs
low: sort | uniq -c: real run
        7.84 real         7.67 user         0.15 sys
Benchmark 1: freq bench/bench-high.txt
  Time (mean ± σ):      1.081 s ±  0.023 s    [User: 1.031 s, System: 0.049 s]
  Range (min … max):    1.068 s …  1.142 s    9 runs

Benchmark 1: freq bench/bench-mid.txt
  Time (mean ± σ):     374.7 ms ±   7.8 ms    [User: 350.5 ms, System: 23.6 ms]
  Range (min … max):   370.2 ms … 394.5 ms    9 runs

Benchmark 1: freq bench/bench-low.txt
  Time (mean ± σ):     335.8 ms ±   3.0 ms    [User: 312.5 ms, System: 22.8 ms]
  Range (min … max):   331.5 ms … 340.8 ms    9 runs
```