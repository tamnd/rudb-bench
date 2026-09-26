#!/bin/bash
# Box 1 of rudb#1828 on gamingpc. W holds bench/ (rudb-bench at main), bin/ (rudb, rudb-bench, clickhouse) and data/hits.parquet. ClickBench over the full hits table, one engine at a time on the P-cores, 1 cold and 3 hot tries per query, median of the hot tries read from the progress lines. Runs as root because the harness drops the page cache before each query.
set -u
W=${W:-/home/gopher/c0-gpc}
cd $W
mkdir -p out scratch
export RUDB_BENCH_DATA=$W/data RUDB_BENCH_RUDB=$W/bin/rudb RUDB_BENCH_CLICKHOUSE=$W/bin/clickhouse \
  RUDB_BENCH_STORED_ANSWERS=off RUDB_BENCH_SCRATCH=$W/scratch \
  RUDB_BENCH_THREADS=16 RUDB_BENCH_MEMORY=20GiB RUDB_BENCH_LOAD_LIMIT=2 RUDB_BENCH_LOAD_WAIT=14400 RUDB_BENCH_PROGRESS=1
CPUS=0-15
( while sleep 10; do echo "$(date +%s) $(cut -d" " -f1-3 /proc/loadavg) gpu $(nvidia-smi --query-gpu=utilization.gpu,memory.used --format=csv,noheader,nounits 2>/dev/null | tr -d ' ')"; done ) >> out/load.log &
SAMPLER=$!
for engine in ${ENGINES:-clickhouse-server duckdb rudb}; do
  echo "$(date -u) $engine start, $(df -h $W | tail -1), load $(cut -d" " -f1-3 /proc/loadavg)" >> out/progress.log
  ( cd bench && taskset -c $CPUS ../bin/rudb-bench run clickbench --engines $engine --runs 4 --protocol upstream --timeout 600 --report ) 2>&1 \
    | while IFS= read -r line; do echo "$(date +%s) $line"; done > out/$engine.harness.log
  cp "$(ls -t bench/reports/*/run-clickbench-*.md | head -1)" out/$engine.run.md 2>/dev/null
  for dir in scratch/rudb-bench-*; do [ -e "$dir" ] && rm -rf "$dir"; done
  echo "$(date -u) $engine done, scratch cleared, $(df -h $W | tail -1)" >> out/progress.log
done
kill $SAMPLER
touch out/DONE
