#!/usr/bin/env python3
"""Measure the sorted-row prototype and DuckDB in fresh processes."""

import argparse
import json
import pathlib
import statistics
import subprocess
import tempfile


SQL = (
    "SELECT RegionID, COUNT(DISTINCT UserID) AS u FROM hits "
    "GROUP BY RegionID ORDER BY u DESC LIMIT 10"
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--projection", required=True)
    parser.add_argument("--data", required=True)
    parser.add_argument("--dictionary", required=True)
    parser.add_argument("--duckdb", required=True)
    parser.add_argument("--duckdb-db", required=True)
    parser.add_argument("--expected-file", type=pathlib.Path, required=True)
    parser.add_argument("--helper", type=pathlib.Path, required=True)
    parser.add_argument("--rounds", type=int, default=51)
    parser.add_argument("--degree", type=int, default=16)
    parser.add_argument("--json", type=pathlib.Path, required=True)
    args = parser.parse_args()
    if args.rounds < 1 or args.degree < 1:
        parser.error("rounds and degree must be positive")

    expected = sorted(args.expected_file.read_text().splitlines())
    commands = {
        "sorted_projection_prototype": [
            args.projection, "scan", args.data, args.dictionary, str(args.degree)
        ],
        "duckdb": [
            args.duckdb, "-readonly", args.duckdb_db, "-noheader", "-csv", "-c", SQL
        ],
    }
    records = []
    with tempfile.TemporaryDirectory(prefix="q9-projection-") as directory:
        resource = pathlib.Path(directory) / "resource.json"
        for trial in range(args.rounds):
            names = list(commands)
            if trial % 2:
                names.reverse()
            for name in names:
                output = subprocess.run(
                    [str(args.helper), str(resource), *commands[name]],
                    capture_output=True,
                    text=True,
                    check=True,
                )
                rows = output.stdout.splitlines()
                counts = [int(row.rsplit(",", 1)[1]) for row in rows]
                if sorted(rows) != expected or counts != sorted(counts, reverse=True):
                    raise RuntimeError(f"{name} trial {trial}: output differs")
                reading = json.loads(resource.read_text())
                records.append({"engine": name, "trial": trial, **reading})

    args.json.parent.mkdir(parents=True, exist_ok=True)
    args.json.write_text(json.dumps(records, indent=2) + "\n")
    for name in commands:
        rows = [row for row in records if row["engine"] == name]
        print(
            name,
            f"wall={statistics.median(row['wall_s'] for row in rows) * 1000:.3f} ms",
            f"cpu={statistics.median(row['user_s'] + row['system_s'] for row in rows) * 1000:.3f} ms",
            f"rss={statistics.median(row['peak_rss_bytes'] for row in rows) / 2**20:.2f} MiB",
        )


if __name__ == "__main__":
    main()
