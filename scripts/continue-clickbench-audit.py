#!/usr/bin/env python3
"""Complete missing ClickBench groups without reusing partial comparisons.

Every incomplete size/query group is rerun in all four cases and every round.
The source journal and its files remain intact. The combined journal records
which capacity limit applied to each selected group.
"""
import argparse
import importlib.util
import json
import math
import os
from pathlib import Path
import shutil
import time

spec = importlib.util.spec_from_file_location('audit', Path(__file__).with_name('clickbench-audit.py'))
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)
ENGINES = ['duckdb-native', 'rudb-native', 'duckdb-parquet', 'rudb-parquet']


def complete_group(rows, hot):
    return len(rows) == len(ENGINES) * (hot + 1) and all(sorted(r['run'] for r in rows if r['engine'] == e) == list(range(hot + 1))
               and all(r['status'] == 'ok' and r['query_s'] is not None
                       for r in rows if r['engine'] == e) for e in ENGINES)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('source', type=Path)
    p.add_argument('output', type=Path)
    p.add_argument('--max-load', type=float)
    p.add_argument('--idle-timeout', type=float, default=600)
    p.add_argument('--timeout', type=float, default=180)
    a = p.parse_args()
    if not math.isfinite(a.timeout) or a.timeout <= 0 or not math.isfinite(a.idle_timeout) or a.idle_timeout <= 0 or (a.max_load is not None and (not math.isfinite(a.max_load) or a.max_load <= 0)):
        p.error('capacity limits and timeouts must be finite and positive')
    source, root = a.source.resolve(), a.output.resolve()
    if root.exists():
        p.error('output must be a new directory')
    meta = json.loads((source / 'metadata.json').read_text())
    original = [json.loads(line) for line in (source / 'raw.jsonl').read_text().splitlines()]
    hot = meta['hot_runs']
    for b in meta['binaries'].values():
        if audit.digest(Path(b['path'])) != b['sha256']:
            raise RuntimeError('binary changed since the source run')
    for data in meta['datasets'].values():
        if audit.digest(Path(data['path'])) != data['sha256']:
            raise RuntimeError('input changed since the source run')
    for name, expected in meta['sql_sha256'].items():
        if audit.digest(source / 'sql' / name) != expected:
            raise RuntimeError('SQL changed since the source run')
    loads = [r for r in original if r['phase'] == 'load']
    if len(loads) != 2 * len(meta['datasets']) or any(r['status'] != 'ok' for r in loads):
        raise RuntimeError('every native load must have completed successfully')
    for size in meta['datasets']:
        for suffix in ['duckdb', 'rudb']:
            if not (source / f'{size}.{suffix}').is_file():
                raise RuntimeError('source native database missing')
    root.mkdir()
    shutil.copytree(source / 'sql', root / 'sql')
    selected = []
    protocols = []

    def retain(row):
        row = dict(row, capacity_limit=meta['max_load'], origin_root=str(source))
        selected.append(row)
        for key in ['stdout', 'stderr']:
            shutil.copy2(source / row[key], root / row[key])
        resource = row['stdout'].removesuffix('.stdout') + '.resource.json'
        shutil.copy2(source / resource, root / resource)

    for row in loads:
        retain(row)
    for size in meta['datasets']:
        for suffix in ['duckdb', 'rudb']:
            (root / f'{size}.{suffix}').symlink_to(source / f'{size}.{suffix}')
    projection = (source / 'sql' / 'projection.sql').read_text()
    settings = f"SET threads={meta['threads']}; SET memory_limit='{meta['memory_limit']}'"
    new_meta = dict(meta, source_run=str(source), source_metadata_sha256=audit.digest(source / 'metadata.json'),
                    continuation_start_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
                    continuation_max_load=a.max_load, continuation_script_sha256=audit.digest(Path(__file__)),
                    mixed_capacity_protocol=True, group_protocols=protocols)
    (root / 'metadata.json').write_text(json.dumps(new_meta, indent=2) + '\n')
    with (root / 'continuation.jsonl').open('x') as journal:
        for size, data in meta['datasets'].items():
            scan = f"SELECT {projection} FROM read_parquet('{data['path'].replace(chr(39), chr(39)*2)}', binary_as_string=True)"
            for q in range(1, 44):
                previous = [r for r in original if r['phase'] == 'query' and r['size'] == size and r['query'] == q]
                reuse = complete_group(previous, hot)
                reuse = reuse and not any(audit.SQL_ERROR.search((source / r['stderr']).read_text()) for r in previous)
                protocols.append(dict(size=size, query=q, retained_original=reuse,
                                      max_load=meta['max_load'] if reuse else a.max_load))
                if reuse:
                    for row in previous:
                        retain(row)
                    continue
                sql = (source / 'sql' / f'q{q}.sql').read_text()
                for run in range(hot + 1):
                    for e in audit.run_order(ENGINES, run + 2*q):
                        binary = 'rudb' if e.startswith('rudb') else 'duckdb'
                        command = [meta['binaries'][binary]['path'], '-batch', '-csv', '-noheader']
                        if e.endswith('native'):
                            suffix = 'rudb' if binary == 'rudb' else 'duckdb'
                            command += [str(source / f'{size}.{suffix}')]
                        command += ['-c', settings]
                        if binary == 'rudb':
                            command += ['-c', 'SET stored_answers=false']
                        command += ['-c', '.timer on']
                        if e.endswith('parquet'):
                            command += ['-c', 'CREATE VIEW hits AS ' + scan]
                        command += ['-c', sql]
                        waited = audit.wait_for_capacity(a.max_load, a.idle_timeout)
                        before = os.getloadavg()
                        row = audit.measure(command, root / f'{size}-{e}-q{q}-r{run}', a.timeout)
                        row.update(size=size, engine=e, query=q, run=run, phase='query',
                                   idle_wait_s=waited, load_before=before, load_after=os.getloadavg(),
                                   capacity_limit=a.max_load, origin_root=str(root))
                        journal.write(json.dumps(row) + '\n')
                        journal.flush()
                        selected.append(row)
                        if row['status'] != 'ok':
                            raise RuntimeError('continuation child failed; keep the incomplete journal')
                (root / 'metadata.json').write_text(json.dumps(new_meta, indent=2) + '\n')
                print(f'{size} q{q}/43 rerun complete', flush=True)
    new_meta.update(end_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
                    load_end=os.getloadavg(), meminfo_end=Path('/proc/meminfo').read_text())
    (root / 'metadata.json').write_text(json.dumps(new_meta, indent=2) + '\n')
    (root / 'raw.jsonl').write_text(''.join(json.dumps(r) + '\n' for r in selected))
    audit.render(root, selected, list(meta['datasets']), hot)


if __name__ == '__main__':
    main()
