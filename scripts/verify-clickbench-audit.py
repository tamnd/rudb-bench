#!/usr/bin/env python3
"""Untimed deterministic retests of differing ClickBench answers.

Original benchmark SQL is never changed. This diagnostic adds all output columns
as tie breakers, so LIMIT chooses the same rows on these small sampled datasets.
It records a separate result, not an excuse to suppress original differences.
"""
import importlib.util
import json
from pathlib import Path
import re
import sys

spec = importlib.util.spec_from_file_location('audit', Path(__file__).with_name('clickbench-audit.py'))
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)
root = Path(sys.argv[1]).resolve()
raw = [json.loads(line) for line in (root/'raw.jsonl').read_text().splitlines()]
meta = json.loads((root/'metadata.json').read_text())
checks = []
for size, data in meta['datasets'].items():
    projection = (root/'sql/projection.sql').read_text()
    source = data['path'].replace("'", "''")
    view = f"CREATE VIEW hits AS SELECT {projection} FROM read_parquet('{source}', binary_as_string=True)"
    for q in range(1, 44):
        pair = [next(r for r in raw if r.get('size') == size and r.get('query') == q and r.get('run') == 0 and r['engine'] == e) for e in ['duckdb', 'rudb']]
        if any(r['status'] != 'ok' for r in pair):
            checks.append(dict(size=size, query=q, result='execution failed'))
            continue
        answers = [audit.answer(root/r['stdout']) for r in pair]
        if audit.equal(*answers):
            checks.append(dict(size=size, query=q, result='original match'))
            continue
        if audit.equal(*[sorted(rows) for rows in answers]):
            checks.append(dict(size=size, query=q, result='same rows; different order'))
            continue
        sql = (root/'sql'/f'q{q}.sql').read_text().strip().rstrip(';')
        if not answers[0] or 'GROUP BY' not in sql.upper():
            checks.append(dict(size=size, query=q, result='unresolved difference'))
            continue
        # ClickBench's outer clauses are flat and LIMIT is its last clause.
        limit = re.search(r'\s+LIMIT\s+\d+(?:\s+OFFSET\s+\d+)?\s*$', sql, re.I)
        body = sql[:limit.start()] if limit else sql
        suffix = limit.group() if limit else ''
        keys = ', '.join(str(i+1) for i in range(len(answers[0][0])))
        deterministic = body + (', ' if 'ORDER BY' in body.upper() else ' ORDER BY ') + keys + suffix
        (root/f'{size}-q{q}-deterministic.sql').write_text(deterministic+';\n')
        outputs = []
        statuses = []
        for e in ['duckdb', 'rudb']:
            command = [meta['binaries'][e]['path'], '-batch', '-csv', '-noheader', '-c', view, '-c', deterministic]
            r = audit.measure(command, root/f'{size}-{e}-q{q}-verify', 120)
            statuses.append(r['status'])
            outputs.append(audit.answer(root/r['stdout']))
        result = 'deterministic retest match' if statuses == ['ok','ok'] and audit.equal(*outputs) else 'UNRESOLVED deterministic retest'
        checks.append(dict(size=size, query=q, result=result, statuses=statuses, sql=deterministic))
        print(size, q, result, flush=True)
(root/'correctness.json').write_text(json.dumps(checks, indent=2)+'\n')

audit.render(root, raw, list(meta['datasets']), meta['hot_runs'])
