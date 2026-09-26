#!/usr/bin/env python3
"""Box 1 on gamingpc: per query median of the 3 hot tries (harness wall and engine clock), cold, load, and rudb against the best rival."""
import bisect
import pathlib
import re
import statistics
import sys

OUT = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else str(pathlib.Path(__file__).parent))
ENGINES = ['clickhouse-server', 'duckdb', 'rudb']
NAMES = [f'q{i}' for i in range(1, 44)]


def seconds(text):
    text = text.strip().rstrip(',')
    for unit, scale in (('us', 1e-6), ('µs', 1e-6), ('ms', 1e-3), ('s', 1.0)):
        if text.endswith(unit):
            return float(text[:-len(unit)]) * scale
    raise ValueError(text)


loads = []
for line in (OUT / 'load.log').read_text().splitlines():
    parts = line.split()
    if len(parts) >= 2:
        loads.append((int(parts[0]), float(parts[1])))
stamps = [t for t, _ in loads]


def window(start, end):
    i = bisect.bisect_left(stamps, start)
    j = bisect.bisect_right(stamps, end)
    inside = [l for _, l in loads[i:j]] or [loads[min(i, len(loads) - 1)][1]]
    return inside[0], max(inside)


LINE = re.compile(r'(\d+) (\S+): (\d+) of \d+, (q\d+), cold (\S+), hot (\S+), hot median (\S+?)'
                  r'(?:, own cold (\S+), own hot (\S+), own hot median (\S+))?$')


def rows_of(engine):
    rows, prev = {}, None
    for line in (OUT / f'{engine}.harness.log').read_text().splitlines():
        m = LINE.match(line)
        ts = int(line.split()[0]) if line[:1].isdigit() else None
        if m:
            first, top = window(prev or ts, ts)
            r = {'cold': seconds(m.group(5)), 'best': seconds(m.group(6)), 'median': seconds(m.group(7)),
                 'load_start': first, 'load_max': top}
            if m.group(8):
                r.update(own_cold=seconds(m.group(8)), own_best=seconds(m.group(9)), own_median=seconds(m.group(10)))
            rows[m.group(4)] = r
            prev = ts
        elif ts and ('waiting' in line or 'loading' in line or 'loaded' in line):
            prev = ts
    return rows


def fmt(v):
    return '' if v is None else (f'{v * 1000:.1f} ms' if v < 1 else f'{v:.2f} s')


data = {e: rows_of(e) for e in ENGINES if (OUT / f'{e}.harness.log').exists()}
present = list(data)
for key, title in (('median', 'harness wall, median of the 3 hot tries'), ('own_median', 'engine clock, median of the 3 hot tries')):
    print(f'\n### {title}\n')
    print('| query | ' + ' | '.join(present) + ' | best rival | rudb / best rival |')
    print('| --- |' + ' ---: |' * (len(present) + 2))
    ratios, totals = [], {e: 0.0 for e in present}
    for q in NAMES:
        vals = {e: data[e].get(q, {}).get(key) for e in present}
        for e, v in vals.items():
            totals[e] += v or 0
        rivals = [(v, e) for e, v in vals.items() if e != 'rudb' and v]
        best = min(rivals) if rivals else None
        ratio = vals.get('rudb') / best[0] if best and vals.get('rudb') else None
        if ratio:
            ratios.append(ratio)
        print(f'| {q} | ' + ' | '.join(fmt(vals[e]) for e in present)
              + f" | {'' if not best else best[1]} | {'' if ratio is None else f'{ratio:.2f}x'} |")
    print('| total | ' + ' | '.join(fmt(totals[e]) for e in present) + ' | | |')
    if ratios:
        print(f'\nrudb / best rival: geomean {statistics.geometric_mean(ratios):.2f}x, best {min(ratios):.2f}x, '
              f'worst {max(ratios):.2f}x, at or under 0.1x on {sum(r <= 0.1 for r in ratios)} of {len(ratios)}, '
              f'under 1x on {sum(r < 1 for r in ratios)}')
print('\n### cold, and load at start / max during each query\n')
print('| query | ' + ' | '.join(f'{e} cold' for e in present) + ' | ' + ' | '.join(f'{e} load' for e in present) + ' |')
print('| --- |' + ' ---: |' * (2 * len(present)))
for q in NAMES:
    rs = [data[e].get(q, {}) for e in present]
    print(f'| {q} | ' + ' | '.join(fmt(r.get('cold')) for r in rs) + ' | '
          + ' | '.join(f"{r['load_start']:.2f}/{r['load_max']:.2f}" if 'load_start' in r else '' for r in rs) + ' |')
for e in present:
    rs = data[e].values()
    ls = [r['load_start'] for r in rs]
    mx = [r['load_max'] for r in rs]
    print(f'{e}: {len(rs)} queries, cold total {sum(r["cold"] for r in rs):.1f}s, load at start median '
          f'{statistics.median(ls):.2f} max {max(ls):.2f}, max during {max(mx):.2f}')
