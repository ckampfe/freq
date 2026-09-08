#!/usr/bin/env sh

echo "high: sort | uniq -c: 3 warmup runs"
sort bench/bench-high.txt | uniq -c >/dev/null
sort bench/bench-high.txt | uniq -c >/dev/null
sort bench/bench-high.txt | uniq -c >/dev/null
echo "high: sort | uniq -c: real run"
/usr/bin/time sort bench/bench-high.txt | uniq -c >/dev/null

echo "mid: sort | uniq -c: 3 warmup runs"
sort bench/bench-mid.txt | uniq -c >/dev/null
sort bench/bench-mid.txt | uniq -c >/dev/null
sort bench/bench-mid.txt | uniq -c >/dev/null
echo "mid: sort | uniq -c: real run"
/usr/bin/time sort bench/bench-mid.txt | uniq -c >/dev/null

echo "low: sort | uniq -c: 3 warmup runs"
sort bench/bench-low.txt | uniq -c >/dev/null
sort bench/bench-low.txt | uniq -c >/dev/null
sort bench/bench-low.txt | uniq -c >/dev/null
echo "low: sort | uniq -c: real run"
/usr/bin/time sort bench/bench-low.txt | uniq -c >/dev/null

hyperfine -w3 -r9 --output=null "freq bench/bench-high.txt" 2>/dev/null
hyperfine -w3 -r9 --output=null "freq bench/bench-mid.txt" 2>/dev/null
hyperfine -w3 -r9 --output=null "freq bench/bench-low.txt" 2>/dev/null
