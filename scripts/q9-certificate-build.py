#!/usr/bin/env python3
"""Measure the extra native certificate build after a completed load."""

import argparse
import json
import pathlib
import statistics
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=pathlib.Path, required=True)
    parser.add_argument('--certifier', type=pathlib.Path, required=True)
    parser.add_argument('--table', required=True)
    parser.add_argument('--group', required=True)
    parser.add_argument('--distinct', required=True)
    parser.add_argument('--rounds', type=int, default=7)
    parser.add_argument('--json', type=pathlib.Path, required=True)
    parser.add_argument('--helper', type=pathlib.Path,
                        default=pathlib.Path(__file__).with_name('measure-child'))
    args = parser.parse_args()
    if args.rounds < 1:
        parser.error('--rounds must be positive')

    records = []
    with tempfile.TemporaryDirectory(prefix='q9-certificate-') as directory:
        path = pathlib.Path(directory) / 'working.db'
        resource = pathlib.Path(directory) / 'resource.json'
        for trial in range(args.rounds):
            subprocess.run(['cp', '--reflink=auto', args.source, path], check=True)
            resource.unlink(missing_ok=True)
            process = subprocess.run([
                args.helper, resource, args.certifier, path,
                args.table, args.group, args.distinct,
            ], capture_output=True, check=False)
            measured = json.loads(resource.read_text()) if resource.exists() else {}
            if process.returncode or measured.get('exit_code') != 0:
                raise RuntimeError(f'certificate trial {trial}: {process.stderr.decode()}')
            records.append({
                'trial': trial,
                'wall_ns': round(measured['wall_s'] * 1e9),
                'cpu_s': measured['user_s'] + measured['system_s'],
                'peak_rss_bytes': measured['peak_rss_bytes'],
                'added_bytes': path.stat().st_size - args.source.stat().st_size,
            })
            path.unlink()

    args.json.parent.mkdir(parents=True, exist_ok=True)
    args.json.write_text(json.dumps(records, indent=2) + '\n')
    print(
        f"wall={statistics.median(row['wall_ns'] for row in records) / 1e6:.3f} ms",
        f"cpu={statistics.median(row['cpu_s'] for row in records) * 1000:.3f} ms",
        f"rss={statistics.median(row['peak_rss_bytes'] for row in records) / 2**20:.2f} MiB",
        f"added={statistics.median(row['added_bytes'] for row in records):.0f} bytes",
    )


if __name__ == '__main__':
    main()
