#!/usr/bin/env python3
"""Time rudb's frontend, parse, bind, rewrite and optimize, on every query of a suite.

The harness already prints planning next to each query, but that column is everything before the
first row moved, the physical build included, and it is read from runs that use every thread. The
compiler's frontend budget in tamnd/rudb#1828 is about parse, bind, rewrite and the logical
optimizer only, taken single threaded, so this reads those phases on their own out of the metrics
document rudb writes with `--metrics`.

The protocol is one thread and several new processes per query, with the query run several times in
each process. The first statement in a process is the cold one: it pays for first touch of the
catalog, the table's schema and the code pages. The statements after it are warm, which is what a
long lived process that has not seen this query before pays. Both are printed because the budget is
stated for a cache miss and a cache miss can happen in either kind of process.

rudb keeps the plan of the last whole table aggregate it ran, keyed on the statement's text, and a
statement that hits it skips the frontend altogether. That is a cache hit and not what the budget is
about, so every warm round gets one more space after its first keyword, which changes the text and
nothing else. A warm round that still reports no parse and no bind is treated as a cache hit and
stops the run rather than being counted as a fast frontend.

Per query the frontend is parse plus bind plus optimize, where optimize already holds the rewrites
(rudb prints the rewrites apart as `rewrite_ns`, part of `optimize_ns`). The suite summary is the
median and the max of the per query medians, which is how the budget is stated, once for cold and
once for warm. The fastest round is printed beside the median because these are wall clock spans,
and on a shared machine the fastest is the one least inflated by somebody else's work. The physical
build and the execute span are printed too so a reader can see how the frontend compares with the
rest of the statement.

`--gate` waits before every process until the one minute load average is below it, which is the
harness's rule for a shared machine. The default is the number of hardware threads. The reading a
process started at is kept, and each query's row carries the highest of them, so a query measured
next to somebody else's work says so.

`--queries` is either a file holding one query a line under a `-- name` comment, which is what
`cargo run --example export_queries` writes, or a directory of `NNx.sql` files, which is how the JOB
queries are kept in `queries/job`.

Usage:

    cargo run --example export_queries -- clickbench rudb /tmp/clickbench.sql
    scripts/frontend-latency.py --rudb /path/to/rudb --database /path/to/hits.rudb \\
        --queries /tmp/clickbench.sql --processes 5 --repeats 5 --set stored_answers=false \\
        --tsv /tmp/clickbench-frontend.tsv
    scripts/frontend-latency.py --rudb /path/to/rudb --database /path/to/imdb.rudb \\
        --queries queries/job --processes 5 --repeats 5
"""

import argparse
import json
import os
import pathlib
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import time

PHASES = ('parse', 'bind', 'rewrite', 'optimizer', 'physical', 'execute')


def queries(path):
    """The named queries of a query file or of a directory of JOB style files."""
    named = []
    if path.is_dir():
        files = [p for p in path.glob('*.sql') if re.fullmatch(r'\d+[a-z]', p.stem)]
        for file in sorted(files, key=lambda p: (int(p.stem[:-1]), p.stem[-1])):
            sql = ' '.join(file.read_text().split()).rstrip(';')
            named.append((file.stem, sql))
    else:
        name = None
        for line in path.read_text().splitlines():
            line = line.strip()
            marked = re.fullmatch(r'-- (q?\d+[a-z]?)', line)
            if marked:
                name = marked.group(1)
            elif line and not line.startswith('--') and name:
                named.append((name, line.rstrip(';')))
                name = None
    if not named:
        sys.exit(f'no queries found in {path}')
    return named


def spaced(sql, extra):
    """The same statement with `extra` more spaces after its first keyword."""
    keyword, _, rest = sql.partition(' ')
    return keyword + ' ' * (1 + extra) + rest


def phases(timing):
    """One statement's spans in microseconds, the optimizer proper apart from the rewrites."""
    rewrite = timing.get('rewrite_ns', 0)
    return {
        'parse': timing['parse_ns'] / 1e3,
        'bind': timing['bind_ns'] / 1e3,
        'rewrite': rewrite / 1e3,
        'optimizer': (timing['optimize_ns'] - rewrite) / 1e3,
        'physical': timing['physical_ns'] / 1e3,
        'execute': timing['execute_ns'] / 1e3,
        'frontend': (timing['parse_ns'] + timing['bind_ns'] + timing['optimize_ns']) / 1e3,
    }


def gate(limit, patience, scratch):
    """Waits for the one minute load average to fall below `limit` and returns the last reading.

    It also waits, with no limit, while the disk the metrics are written to has under 2 GB free.
    A full disk cuts a metrics document off part way, and a shared machine's disk does fill.
    """
    waited = 0
    while shutil.disk_usage(scratch).free < 2 << 30:
        if waited % 300 == 0:
            free = shutil.disk_usage(scratch).free >> 20
            print(f'waiting for disk, {free} MiB free', file=sys.stderr, flush=True)
        time.sleep(5)
        waited += 5
    waited = 0
    while True:
        load = os.getloadavg()[0]
        if load < limit or waited >= patience:
            return load
        time.sleep(5)
        waited += 5


def process(args, sql, scratch):
    """The spans of every statement one new process ran, or the error it stopped on."""
    metrics = scratch / 'metrics.jsonl'
    metrics.unlink(missing_ok=True)
    command = [str(args.rudb), '--set', f'threads={args.threads}']
    for setting in args.set:
        command += ['--set', setting]
    command += ['--metrics', str(metrics), '-readonly', str(args.database)]
    script = ''.join(f'{spaced(sql, round)};\n' for round in range(1 + args.repeats))
    made = subprocess.run(command, input=script, stdout=subprocess.DEVNULL,
                          stderr=subprocess.PIPE, text=True, check=False)
    documents = []
    for line in metrics.read_text().splitlines() if metrics.exists() else []:
        if not line.strip():
            continue
        try:
            documents.append(json.loads(line))
        except json.JSONDecodeError:
            # A document cut off part way, which is what a process killed while writing it
            # leaves. The exit status below says why.
            return None, f'exit status {made.returncode}, a metrics document cut off, ' \
                f'{made.stderr.strip()[-200:]}'
    failed = [d for d in documents if d.get('outcome', {}).get('state') != 'succeeded']
    if made.returncode or failed or len(documents) != 1 + args.repeats:
        why = failed[0]['outcome'].get('message') if failed else made.stderr.strip()[-300:]
        return None, why or f'{len(documents)} documents for {1 + args.repeats} statements'
    runs = [phases(document['timing']) for document in documents]
    for run in runs[1:]:
        if run['parse'] == 0 and run['bind'] == 0:
            sys.exit(f'a warm round of {sql[:70]} skipped the frontend, which is a plan cache hit')
    return runs, None


def middle(runs, phase):
    return statistics.median(run[phase] for run in runs)


def summary(label, frontends):
    """One line of the suite summary over per query numbers."""
    ordered = sorted(frontends, key=lambda pair: pair[1])
    values = [value for _, value in ordered]
    over = sum(1 for value in values if value > 300)
    over2 = sum(1 for value in values if value > 2000)
    return (f'{label}: median {statistics.median(values):.1f}us, max {values[-1]:.1f}us '
            f'({ordered[-1][0]}), {over} of {len(values)} over 300us, {over2} over 2000us')


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--rudb', required=True, type=pathlib.Path, help='the rudb binary')
    parser.add_argument('--database', required=True, type=pathlib.Path,
                        help='the rudb database file holding the suite')
    parser.add_argument('--queries', required=True, type=pathlib.Path,
                        help='a query file with one query a line under a -- name comment, or a '
                             'directory of NNx.sql files')
    parser.add_argument('--processes', type=int, default=5,
                        help='new processes per query, each giving one cold statement')
    parser.add_argument('--repeats', type=int, default=5,
                        help='warm statements per process after the cold one')
    parser.add_argument('--threads', type=int, default=1, help='what to SET threads to')
    parser.add_argument('--set', action='append', default=[], metavar='NAME=VALUE',
                        help='one more setting for every process, repeatable')
    parser.add_argument('--only', nargs='+', metavar='NAME', help='just these queries')
    parser.add_argument('--gate', type=float, default=float(os.cpu_count() or 1),
                        help='wait before each process for the 1 minute load to be below this')
    parser.add_argument('--patience', type=int, default=1800,
                        help='seconds to wait at the gate before going ahead anyway')
    parser.add_argument('--tsv', type=pathlib.Path, help='also write the per query rows here')
    args = parser.parse_args()
    if args.processes < 1 or args.repeats < 1:
        parser.error('--processes and --repeats must be positive')

    named = queries(args.queries)
    if args.only:
        named = [(name, sql) for name, sql in named if name in args.only]
    print(f'{len(named)} queries, {args.processes} processes of 1 cold and {args.repeats} warm '
          f'statements, threads={args.threads}, settings {args.set or "none"}, gate below a load of '
          f'{args.gate:g}, times in us')
    header = ['query'] + [f'cold_{p}' for p in PHASES] + ['cold_frontend', 'cold_fastest'] + \
        [f'warm_{p}' for p in PHASES] + ['warm_frontend', 'warm_fastest', 'load']
    print('\t'.join(header))
    rows = []
    cold = []
    warm = []
    cold_fastest = []
    warm_fastest = []
    skipped = []
    with tempfile.TemporaryDirectory() as scratch:
        scratch = pathlib.Path(scratch)
        for name, sql in named:
            colds = []
            warms = []
            why = None
            loads = []
            for _ in range(args.processes):
                loads.append(gate(args.gate, args.patience, scratch))
                runs, why = process(args, sql, scratch)
                if runs is None:
                    break
                colds.append(runs[0])
                warms.extend(runs[1:])
            if why:
                skipped.append((name, why))
                print(f'{name}\tfailed: {why}', flush=True)
                continue
            row = [name]
            for runs in (colds, warms):
                row += [f'{middle(runs, phase):.1f}' for phase in PHASES]
                row += [f'{middle(runs, "frontend"):.1f}',
                        f'{min(run["frontend"] for run in runs):.1f}']
            row.append(f'{max(loads):.2f}')
            cold.append((name, middle(colds, 'frontend')))
            warm.append((name, middle(warms, 'frontend')))
            cold_fastest.append((name, min(run['frontend'] for run in colds)))
            warm_fastest.append((name, min(run['frontend'] for run in warms)))
            rows.append(row)
            print('\t'.join(row), flush=True)
    if args.tsv:
        args.tsv.write_text('\n'.join('\t'.join(row) for row in [header] + rows) + '\n')
    if not rows:
        sys.exit('no query ran')
    print()
    print(summary('cold frontend over the per query medians', cold))
    print(summary('cold frontend over the per query fastest rounds', cold_fastest))
    print(summary('warm frontend over the per query medians', warm))
    print(summary('warm frontend over the per query fastest rounds', warm_fastest))
    for label, column in (('cold', 1), ('warm', 1 + len(PHASES) + 2)):
        totals = {phase: sum(float(row[column + i]) for row in rows)
                  for i, phase in enumerate(PHASES[:4])}
        whole = sum(totals.values()) or 1
        shares = ', '.join(f'{phase} {100 * value / whole:.0f}%' for phase, value in totals.items())
        print(f'{label} frontend by phase, summed over the per query medians: {shares}')
    loads = [float(row[-1]) for row in rows]
    print(f'load at the start of a process, highest per query: median {statistics.median(loads):.2f}, '
          f'max {max(loads):.2f}, {sum(1 for load in loads if load >= args.gate)} queries went '
          f'ahead past the gate')
    for name, why in skipped:
        print(f'{name} did not run: {why}')


if __name__ == '__main__':
    main()
