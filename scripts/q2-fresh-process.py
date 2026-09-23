#!/usr/bin/env python3
"""Measure ClickBench Q2 with one SQL statement in each new process."""

import argparse
import json
import os
import pathlib
import statistics
import subprocess
import sys
import tempfile
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rudb', required=True)
    parser.add_argument('--duckdb', required=True)
    parser.add_argument('--rudb-db', required=True)
    parser.add_argument('--duckdb-db', required=True)
    parser.add_argument('--expected', required=True, help='Expected count')
    parser.add_argument('--rounds', type=int, default=51)
    parser.add_argument('--json', type=pathlib.Path, required=True)
    args = parser.parse_args()
    if args.rounds < 1:
        parser.error('--rounds must be positive')

    sql = 'SELECT COUNT(*) FROM hits WHERE AdvEngineID <> 0'
    cases = [('rudb', args.rudb, args.rudb_db), ('duckdb', args.duckdb, args.duckdb_db)]
    records = []
    with tempfile.TemporaryDirectory(prefix='q2-fresh-') as directory:
        stdout_path = pathlib.Path(directory) / 'stdout'
        stderr_path = pathlib.Path(directory) / 'stderr'
        for trial in range(args.rounds):
            for engine, binary, database in cases[trial % 2:] + cases[:trial % 2]:
                command = [binary, '-readonly', database, '-noheader', '-csv', '-c', sql]
                with stdout_path.open('wb') as stdout, stderr_path.open('wb') as stderr:
                    start = time.perf_counter_ns()
                    process = subprocess.Popen(command, stdout=stdout, stderr=stderr)
                    _, status, usage = os.wait4(process.pid, 0)
                    wall_ns = time.perf_counter_ns() - start
                output = stdout_path.read_text()
                if status != 0 or output != args.expected + '\n':
                    raise RuntimeError(
                        f'{engine} trial {trial}: status={status}, stdout={output!r}, '
                        f'stderr={stderr_path.read_text()!r}'
                    )
                records.append({
                    'engine': engine,
                    'trial': trial,
                    'wall_ns': wall_ns,
                    'cpu_s': usage.ru_utime + usage.ru_stime,
                    'peak_rss_bytes': usage.ru_maxrss * (1024 if sys.platform == 'linux' else 1),
                })

    args.json.parent.mkdir(parents=True, exist_ok=True)
    args.json.write_text(json.dumps(records, indent=2) + '\n')
    for engine, _, _ in cases:
        rows = [row for row in records if row['engine'] == engine]
        print(
            engine,
            f"wall={statistics.median(row['wall_ns'] for row in rows) / 1e6:.3f} ms",
            f"cpu={statistics.median(row['cpu_s'] for row in rows) * 1000:.3f} ms",
            f"rss={statistics.median(row['peak_rss_bytes'] for row in rows) / 2**20:.2f} MiB",
        )


if __name__ == '__main__':
    main()
