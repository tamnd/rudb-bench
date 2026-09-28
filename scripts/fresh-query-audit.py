#!/usr/bin/env python3
"""Rotate arbitrary SQL commands in fresh processes and retain every reading.

Cases are a JSON list with unique names, command argument lists, and expected
CSV strings. Waiting for host capacity happens before the measured child starts.
"""
import argparse
import importlib.util
import json
import math
import os
from pathlib import Path
import re
import statistics

spec = importlib.util.spec_from_file_location('audit', Path(__file__).with_name('clickbench-audit.py'))
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


def run_order(cases, run):
    """Every neighboring pair of rounds uses the same rotation in opposite orders."""
    return audit.run_order(cases, run)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--cases', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--hot', type=int, default=51)
    p.add_argument('--max-load', type=float, default=max(1, (os.cpu_count() or 1) / 4))
    p.add_argument('--idle-timeout', type=float, default=600)
    p.add_argument('--timeout', type=float, default=60)
    a = p.parse_args()
    if a.hot < 5 or any(not math.isfinite(x) or x <= 0 for x in [a.max_load, a.idle_timeout, a.timeout]):
        p.error('at least five hot repetitions and finite positive limits are required')
    cases = json.loads(a.cases.read_text())
    if not cases or len({c['name'] for c in cases}) != len(cases):
        p.error('cases must have unique names')
    for case in cases:
        if not re.fullmatch('[a-zA-Z0-9-]+', case['name']) or not case['command'] or not isinstance(case['expected'], str):
            p.error('each case needs a safe name, a command, and expected CSV')
    a.output.mkdir(parents=True, exist_ok=False)
    meta = dict(cases=cases, argv=os.sys.argv, cpu_count=os.cpu_count(), max_load=a.max_load,
                parquet_mirror=False, first_cache='unflushed', hot=a.hot,
                binaries={c['command'][0]: audit.digest(c['command'][0]) for c in cases})
    (a.output/'metadata.json').write_text(json.dumps(meta, indent=2)+'\n')
    records = []
    with (a.output/'raw.jsonl').open('x') as log:
        for run in range(a.hot+1):
            for case in run_order(cases, run):
                idle_wait = audit.wait_for_capacity(a.max_load, a.idle_timeout)
                before = os.getloadavg()
                r = audit.measure(case['command'], a.output/f'{case["name"]}-r{run}', a.timeout)
                r.update(case=case['name'], run=run, load_before=before, load_after=os.getloadavg(),
                         idle_wait_s=idle_wait)
                records.append(r)
                log.write(json.dumps(r)+'\n')
                log.flush()
                output = (a.output/r['stdout']).read_text()
                if r['status'] != 'ok' or output != case['expected']:
                    raise RuntimeError(f'{case["name"]} r{run}: failed execution or unexpected CSV')
            print(f'Round {run}/{a.hot}', flush=True)
    summary = {}
    for case in cases:
        rows = [r for r in records if r['case'] == case['name'] and r['run'] > 0]
        summary[case['name']] = {k: statistics.median(r[k] for r in rows) for k in ['wall_s', 'cpu_s', 'peak_rss_bytes']}
    (a.output/'summary.json').write_text(json.dumps(summary, indent=2)+'\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
