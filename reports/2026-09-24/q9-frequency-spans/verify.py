import csv
import io
import json
import subprocess
from pathlib import Path

root = Path('/root/q9-projection-bench')
rudb = root / 'rudb-q9-frequency-spans-current'
duck = Path('/root/.local/bin/duckdb')
outputs = []
for size in ('1k', '10k', '1m', '10m'):
    native = root / ('hits-1k-span-schema.rudb' if size == '1k' else f'hits-{size}-span.rudb')
    duckdb = root / (f'hits-{size}.duckdb' if size in ('1k', '10k') else f'q2-direct-audit-{size}/hits-{size}.duckdb')
    for query in ('SELECT count(*) FROM hits', (root / 'q2.sql').read_text(), (root / 'q8.sql').read_text()):
        results = []
        for binary, db in ((rudb, native), (duck, duckdb)):
            raw = subprocess.check_output([str(binary), '-csv', '-noheader', str(db), '-c', query], text=True)
            rows = list(csv.reader(io.StringIO(raw)))
            results.append(sorted(rows) if 'GROUP BY' in query else rows)
        if results[0] != results[1]:
            raise SystemExit(f'{size}: mismatch for {query}: {results}')
        outputs.append({'size': size, 'query': query, 'rows': len(results[0]), 'first': results[0][:3]})
print(json.dumps(outputs, indent=2))
