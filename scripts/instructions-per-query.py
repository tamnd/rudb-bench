#!/usr/bin/env python3
"""Count the instructions one engine retires on every query of a suite.

`scripts/tpch-instructions-ab.py` compares two binaries and needs both databases on disk at once,
which a machine short of disk cannot always give it. This measures one side at a time with the same
protocol, so two runs of it against two engines give the same numbers the A/B script would: one
query per process, one thread, several rounds, the median taken, and the count of `SELECT 1` on the
same binary subtracted so what is left is the query rather than the process start up. It borrows the
query reader, the command lines and the perf parsing from that script rather than keeping a second
copy of them.

Usage:

    cargo run --example export_queries -- clickbench duckdb /tmp/clickbench.sql
    scripts/instructions-per-query.py --engine duckdb --binary /usr/local/bin/duckdb \\
        --database /path/to/hits.duckdb --queries /tmp/clickbench.sql --rounds 3
"""

import argparse
import importlib.util
import pathlib
import statistics
import sys

HERE = pathlib.Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location('ab', HERE / 'tpch-instructions-ab.py')
ab = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ab)


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--engine', required=True, choices=('rudb', 'duckdb'),
                        help='how to invoke the binary')
    parser.add_argument('--binary', required=True, type=pathlib.Path, help='the engine binary')
    parser.add_argument('--database', required=True, type=pathlib.Path,
                        help='the database file holding the suite')
    parser.add_argument('--queries', required=True, type=pathlib.Path,
                        help='the query file, one query a line under a -- qNN comment')
    parser.add_argument('--only', nargs='+', metavar='qNN', help='measure only these queries')
    parser.add_argument('--rounds', type=int, default=3, help='runs a query, the median taken')
    parser.add_argument('--threads', type=int, default=1, help='what to SET threads to')
    args = parser.parse_args()
    if args.rounds < 1:
        parser.error('--rounds must be positive')
    if sys.platform != 'linux':
        parser.error('perf is Linux only')

    def count(sql):
        return ab.ran(args.engine, args.binary, args.database, sql, args.threads)[0]

    named = ab.queries(args.queries, args.only)
    base = statistics.median(count('SELECT 1') for _ in range(args.rounds))
    print(f'{args.engine}, {len(named)} queries, {args.rounds} rounds, threads={args.threads}')
    print(f'SELECT 1  {base / 1e6:.1f}M, subtracted from every query below')
    total = 0.0
    for name, sql in named:
        took = statistics.median(count(sql) for _ in range(args.rounds)) - base
        total += took
        print(f'{name}  {took / 1e6:.1f}M', flush=True)
    print(f'\nsuite  {total / 1e9:.3f}G')


if __name__ == '__main__':
    main()
