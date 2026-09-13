#!/usr/bin/env python3
"""Collect rudb operator metrics for the ClickBench memory audit."""

import argparse
import json
from pathlib import Path
import subprocess


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("audit", type=Path)
    parser.add_argument("--rudb")
    parser.add_argument("--measure-helper", type=Path)
    parser.add_argument("--case", action="append", required=True, help="SIZE:QUERY")
    args = parser.parse_args()
    root = args.audit.resolve()
    meta = json.loads((root / "metadata.json").read_text())
    rudb = args.rudb or meta["binaries"]["rudb"]["path"]
    projection = (root / "sql/projection.sql").read_text()
    for case in args.case:
        size, query_text = case.split(":", 1)
        query = int(query_text.removeprefix("q"))
        source = meta["datasets"][size]["path"].replace("'", "''")
        view = (
            f"CREATE VIEW hits AS SELECT {projection} "
            f"FROM read_parquet('{source}', binary_as_string=True)"
        )
        metrics = root / f"deep-{size}-q{query}-metrics.jsonl"
        command = [
                rudb,
                "-batch",
                "-csv",
                "-noheader",
                "--metrics",
                str(metrics),
                "-c",
                view,
                "-c",
                (root / "sql" / f"q{query}.sql").read_text(),
            ]
        resource = root / f"deep-{size}-q{query}-resource.json"
        if args.measure_helper:
            command = [str(args.measure_helper.resolve()), str(resource), *command]
        completed = subprocess.run(
            command,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
        measured = json.loads(resource.read_text()) if resource.exists() else None
        print(size, query, completed.returncode, measured, completed.stderr.strip())


if __name__ == "__main__":
    main()
