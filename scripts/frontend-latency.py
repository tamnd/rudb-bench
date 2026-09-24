#!/usr/bin/env python3
"""Time rudb's frontend, parse, bind and optimize, on every query of a suite.

The harness already prints planning next to each query, but that column is everything before the
first row moved, the physical build included, and it is read from runs that use every thread. The
compiler's frontend budget in tamnd/rudb#1828 is about parse, bind, rewrite and the logical
optimizer only, taken single threaded, so this reads those phases on their own out of the metrics
document rudb writes with `--metrics`.

The protocol is one query per process, one thread, several rounds, the median of each phase taken
per query. The suite summary is the median and the max of those per query medians, which is how the
budget is stated. The fastest round is printed beside the median because these are wall clock
spans, and on a shared machine the fastest is the one least inflated by somebody else's work. The
physical build and the execute span are printed too so a reader can see how the frontend compares
with the rest of the statement.

Usage:

    cargo run --example export_queries -- clickbench rudb /tmp/clickbench.sql
    scripts/frontend-latency.py --rudb /path/to/rudb --database /path/to/hits.rudb \\
        --queries /tmp/clickbench.sql --rounds 5
"""

import argparse
import json
import pathlib
import statistics
import subprocess
import sys
import tempfile

PHASES = ('parse_ns', 'bind_ns', 'optimize_ns', 'physical_ns', 'execute_ns')


def queries(path):
    """The named queries of a file holding one query a line under a `-- qNN` comment."""
    named = []
    name = None
    for line in path.read_text().splitlines():
        line = line.strip()
        if line.startswith('-- q') and line[4:].isdigit():
            name = line[3:]
        elif line and not line.startswith('--') and name:
            named.append((name, line))
            name = None
    if not named:
        sys.exit(f'no queries found in {path}')
    return named


def timing(binary, database, sql, threads, scratch):
    """The timing block of the metrics document one query wrote in one new process."""
    metrics = scratch / 'metrics.jsonl'
    metrics.unlink(missing_ok=True)
    made = subprocess.run(
        [str(binary), '--set', f'threads={threads}', '--metrics', str(metrics), '-readonly',
         str(database), '-c', sql],
        capture_output=True, text=True, check=False)
    if made.returncode:
        sys.exit(f'{binary} failed on {sql[:70]}: {made.stderr[-600:]}')
    lines = [line for line in metrics.read_text().splitlines() if line.strip()]
    if not lines:
        sys.exit(f'no metrics document for {sql[:70]}')
    return json.loads(lines[-1])['timing']


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--rudb', required=True, type=pathlib.Path, help='the rudb binary')
    parser.add_argument('--database', required=True, type=pathlib.Path,
                        help='the rudb database file holding the suite')
    parser.add_argument('--queries', required=True, type=pathlib.Path,
                        help='the query file, one query a line under a -- qNN comment')
    parser.add_argument('--rounds', type=int, default=5, help='runs a query, the median taken')
    parser.add_argument('--threads', type=int, default=1, help='what to SET threads to')
    args = parser.parse_args()
    if args.rounds < 1:
        parser.error('--rounds must be positive')

    named = queries(args.queries)
    print(f'{len(named)} queries, {args.rounds} rounds, threads={args.threads}, times in us')
    print('query  parse  bind  optimize  frontend  fastest  physical  execute')
    frontends = []
    fastest = []
    worst = 0.0
    with tempfile.TemporaryDirectory() as scratch:
        scratch = pathlib.Path(scratch)
        for name, sql in named:
            runs = [timing(args.rudb, args.database, sql, args.threads, scratch)
                    for _ in range(args.rounds)]
            mid = {phase: statistics.median(run.get(phase, 0) for run in runs) / 1e3
                   for phase in PHASES}
            spans = [(run['parse_ns'] + run['bind_ns'] + run['optimize_ns']) / 1e3 for run in runs]
            front = statistics.median(spans)
            worst = max([worst] + spans)
            frontends.append(front)
            fastest.append(min(spans))
            print(f'{name}  {mid["parse_ns"]:.1f}  {mid["bind_ns"]:.1f}  {mid["optimize_ns"]:.1f}  '
                  f'{front:.1f}  {min(spans):.1f}  {mid["physical_ns"]:.1f}  {mid["execute_ns"]:.1f}',
                  flush=True)
    print(f'\nfrontend over the per query medians: median {statistics.median(frontends):.1f}us, '
          f'max {max(frontends):.1f}us')
    print(f'frontend over the per query fastest rounds: median {statistics.median(fastest):.1f}us, '
          f'max {max(fastest):.1f}us')
    print(f'frontend slowest single run: {worst:.1f}us')


if __name__ == '__main__':
    main()
