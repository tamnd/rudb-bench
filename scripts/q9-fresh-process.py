#!/usr/bin/env python3
"""Measure ClickBench Q9 with one SQL statement in each new process."""

import argparse
import json
import os
import pathlib
import statistics
import subprocess
import sys
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rudb', required=True)
    parser.add_argument('--duckdb', required=True)
    parser.add_argument('--rudb-db', required=True)
    parser.add_argument('--duckdb-db', required=True)
    parser.add_argument('--rudb-main', help='Optional current-main binary for a three-way run')
    parser.add_argument('--rudb-main-db', help='Native file readable by the current-main binary')
    parser.add_argument('--expected-file', type=pathlib.Path, required=True, help='Expected CSV output')
    parser.add_argument('--rounds', type=int, default=51)
    parser.add_argument('--json', type=pathlib.Path, required=True)
    parser.add_argument('--helper', type=pathlib.Path,
                        default=pathlib.Path(__file__).with_name('measure-child'))
    args = parser.parse_args()
    if args.rounds < 1:
        parser.error('--rounds must be positive')
    if sys.platform != 'linux':
        parser.error('the resource helper requires Linux wait4')
    if not args.helper.is_file():
        parser.error('compile scripts/measure-child.c first')

    sql = 'SELECT RegionID, COUNT(DISTINCT UserID) AS u FROM hits GROUP BY RegionID ORDER BY u DESC LIMIT 10'
    expected = args.expected_file.read_text()
    expected_rows = sorted(expected.splitlines())
    cases = [('rudb', args.rudb, args.rudb_db), ('duckdb', args.duckdb, args.duckdb_db)]
    if args.rudb_main:
        cases.insert(0, ('rudb-main', args.rudb_main, args.rudb_main_db or args.rudb_db))
    records = []
    with tempfile.TemporaryDirectory(prefix='q9-fresh-') as directory:
        stdout_path = pathlib.Path(directory) / 'stdout'
        stderr_path = pathlib.Path(directory) / 'stderr'
        resource_path = pathlib.Path(directory) / 'resource.json'
        for trial in range(args.rounds):
            offset = trial % len(cases)
            for engine, binary, database in cases[offset:] + cases[:offset]:
                command = [binary, '-readonly', database, '-noheader', '-csv', '-c', sql]
                with stdout_path.open('wb') as stdout, stderr_path.open('wb') as stderr:
                    resource_path.unlink(missing_ok=True)
                    process = subprocess.run(
                        [str(args.helper), str(resource_path), *command],
                        stdout=stdout, stderr=stderr, check=False,
                        env={**os.environ, 'LC_ALL': 'C', 'TZ': 'UTC'},
                    )
                resource = json.loads(resource_path.read_text()) if resource_path.exists() else {}
                output = stdout_path.read_text()
                counts = [int(row.rsplit(',', 1)[1]) for row in output.splitlines()] if output else []
                ordered = all(left >= right for left, right in zip(counts, counts[1:]))
                if (process.returncode != 0 or resource.get('exit_code') != 0
                        or sorted(output.splitlines()) != expected_rows or not ordered):
                    raise RuntimeError(
                        f'{engine} trial {trial}: status={process.returncode}, stdout={output!r}, '
                        f'stderr={stderr_path.read_text()!r}'
                    )
                records.append({
                    'engine': engine,
                    'trial': trial,
                    'wall_ns': round(resource['wall_s'] * 1e9),
                    'cpu_s': resource['user_s'] + resource['system_s'],
                    'peak_rss_bytes': resource['peak_rss_bytes'],
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
