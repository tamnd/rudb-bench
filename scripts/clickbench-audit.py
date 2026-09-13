#!/usr/bin/env python3
"""Linux ClickBench size ladder with raw wait4 measurements and all 43 queries.

First means first execution in a fresh process, not a flushed OS page cache.
Every hot repetition also starts a fresh process. Results are fully rendered.
DuckDB loads its native table once; rudb decodes the source Parquet per query.
No engine-specific profiling is enabled inside the timed interval.
"""
import argparse
import csv
import hashlib
import io
import json
import math
import os
from pathlib import Path
import platform
import re
import signal
import statistics
import subprocess
import sys
import threading
import time

HELPER = Path(__file__).with_name('measure-child')

TIMER = re.compile(r"^Run Time \(s\): real ([0-9.]+).*$", re.M)


def measure(command, prefix, timeout):
    """Reap this exact child; never subtract cumulative RUSAGE_CHILDREN peaks."""
    prefix = Path(prefix)
    resource_path = prefix.with_suffix('.resource.json')
    resource_path.unlink(missing_ok=True)
    done = threading.Event()
    timed_out = threading.Event()
    with prefix.with_suffix('.stdout').open('wb') as out, prefix.with_suffix('.stderr').open('wb') as err:
        start = time.perf_counter_ns()
        child = subprocess.Popen([str(HELPER), str(resource_path), *command], stdout=out, stderr=err, start_new_session=True,
                                 env={**os.environ, 'LC_ALL': 'C', 'TZ': 'UTC'})

        def watchdog():
            if not done.wait(timeout):
                timed_out.set()
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass

        watcher = threading.Thread(target=watchdog, daemon=True)
        watcher.start()
        _, status = os.waitpid(child.pid, 0)
        wall = (time.perf_counter_ns() - start) / 1e9
        done.set()
        watcher.join()
        child.returncode = os.waitstatus_to_exitcode(status)
    stdout = prefix.with_suffix('.stdout').read_text(errors='replace')
    stderr = prefix.with_suffix('.stderr').read_text(errors='replace')
    clocks = TIMER.findall(stdout) or TIMER.findall(stderr)
    measured = json.loads(resource_path.read_text()) if resource_path.exists() else {}
    result = dict(command=command, status='timeout' if timed_out.is_set() else
                ('ok' if child.returncode == 0 else 'error'), exit_code=child.returncode,
                query_s=float(clocks[-1]) if clocks and child.returncode == 0 else None,
                stdout=prefix.with_suffix('.stdout').name,
                stderr=prefix.with_suffix('.stderr').name)
    # Wrapper counters are not engine counters. A killed wrapper has no reading.
    for key in ['wall_s', 'user_s', 'system_s', 'peak_rss_bytes', 'read_bytes',
                'write_bytes', 'major_faults', 'minor_faults',
                'voluntary_switches', 'involuntary_switches']:
        result[key] = measured.get(key)
    result['exit_code'] = measured.get('exit_code', child.returncode)
    result['orchestrator_wall_s'] = wall
    result['cpu_s'] = measured['user_s'] + measured['system_s'] if measured else None
    if not measured and result['status'] == 'ok':
        result['status'] = 'measurement_error'
    return result


def answer(path):
    text = TIMER.sub('', path.read_text()).strip()
    return list(csv.reader(io.StringIO(text)))


def equal(a, b):
    if len(a) != len(b):
        return False
    for ar, br in zip(a, b):
        if len(ar) != len(br):
            return False
        for av, bv in zip(ar, br):
            if av == bv:
                continue
            # Preserve exact integer comparisons, especially UserID near 2^63.
            if re.fullmatch(r'[+-]?\d+', av) and re.fullmatch(r'[+-]?\d+', bv):
                if int(av) == int(bv):
                    continue
                return False
            try:
                if math.isclose(float(av), float(bv), rel_tol=1e-9, abs_tol=1e-9):
                    continue
            except ValueError:
                pass
            return False
    return True


def digest(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def render(root, records, sizes, hot):
    lines = ['# ClickBench measurement audit', '',
             'All 43 DuckDB SQL queries are attempted on both engines. Five hot repetitions are required for a complete row. A failed repetition invalidates that query; successful fragments are never averaged into a result.', '',
             'First execution is not disk-cold: the page cache is not flushed. Each repetition uses a fresh process. Query seconds are the CLI timer, including result rendering; wall and CPU seconds and peak RSS cover the whole child process. CPU and RSS come from Linux wait4 for that child. RSS is the maximum resident set, not allocated bytes or an incremental memory delta. DuckDB uses a loaded native table; rudb reads and decodes Parquet on every query. These are sample results, not official ClickBench scores.', '',
             '| Size | Engine | Complete / 43 | Query median sum (s) | Process wall median sum (s) | CPU median sum (s) | Peak RSS (MiB) |',
             '| --- | --- | ---: | ---: | ---: | ---: | ---: |']
    checks_path = root / 'correctness.json'
    checks = {(c['size'], c['query']): c['result'] for c in json.loads(checks_path.read_text())} if checks_path.exists() else {}
    details = []
    for size in sizes:
        if not any(r.get('size') == size and r.get('phase') == 'query' for r in records):
            continue
        for engine in ['duckdb', 'rudb']:
            groups = []
            for q in range(1, 44):
                rows = [r for r in records if r.get('size') == size and r.get('engine') == engine and r.get('query') == q]
                complete = len(rows) == hot + 1 and all(r['status'] == 'ok' and r['query_s'] is not None for r in rows)
                if complete:
                    groups.append(rows)
                if rows:
                    h = rows[1:]
                    med = lambda key: statistics.median(r[key] for r in h) if complete else None
                    spread = (sorted(r['query_s'] for r in h)[math.ceil(.75 * hot)-1] - sorted(r['query_s'] for r in h)[math.ceil(.25 * hot)-1]) if complete else None
                    details.append(dict(size=size, engine=engine, query=q, complete=complete,
                                        first_query_s=rows[0]['query_s'], first_wall_s=rows[0]['wall_s'],
                                        query_median_s=med('query_s'), query_iqr_s=spread,
                                        wall_median_s=med('wall_s'), cpu_median_s=med('cpu_s'),
                                        peak_rss_bytes=max((r['peak_rss_bytes'] or 0) for r in rows)))
            sums = [sum(statistics.median(r[k] for r in rows[1:]) for rows in groups) for k in ['query_s', 'wall_s', 'cpu_s']]
            peaks = [r['peak_rss_bytes'] for r in records if r.get('size') == size and r.get('engine') == engine and r.get('phase') == 'query' and r['peak_rss_bytes'] is not None]
            peak = f'{max(peaks) / 2**20:.2f}' if peaks else 'unavailable'
            lines.append(f'| {size} | {engine} | {len(groups)} | {sums[0]:.6f} | {sums[1]:.6f} | {sums[2]:.6f} | {peak} |')
    lines += ['', 'Time totals cover complete queries only; peak RSS includes every measured attempt, including failures. Compare engines only on the same completed query set. Raw JSONL retains every repetition, exit status, CPU components, I/O, page faults, context switches, command, and output paths.', '',
              '| Size | Query | Answer check |', '| --- | --- | --- |']
    for size in sizes:
        for q in range(1, 44):
            pair = [[r for r in records if r.get('size') == size and r.get('engine') == e and r.get('query') == q and r.get('run') == 0] for e in ['duckdb', 'rudb']]
            if not all(pair):
                continue
            a, b = pair[0][0], pair[1][0]
            if a['status'] != 'ok' or b['status'] != 'ok':
                check = f"duckdb {a['status']}, rudb {b['status']}"
            elif equal(answer(root / a['stdout']), answer(root / b['stdout'])):
                check = 'match'
            else:
                caveat = root / 'sql' / f'q{q}.caveat'
                check = 'different; investigate (known tie/semantic caveat)' if caveat.exists() else '**MISMATCH**'
            check = checks.get((size, q), check)
            lines.append(f'| {size} | q{q} | {check} |')
    if checks:
        lines += ['', 'Answer checks retain original differences. Same rows with different ordering are reported separately; differing row selections are rerun with deterministic tie breakers in a separate untimed diagnostic. Those diagnostic queries are stored alongside the raw outputs and never replace the timed SQL.', '']
    lines += ['', '| Size | Engine | Query | First query s | Hot median s | Hot IQR s | Hot process wall s | Hot CPU s | Peak RSS MiB |', '| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |']
    for d in details:
        if d['complete']:
            lines.append(f"| {d['size']} | {d['engine']} | q{d['query']} | {d['first_query_s']:.6f} | {d['query_median_s']:.6f} | {d['query_iqr_s']:.6f} | {d['wall_median_s']:.6f} | {d['cpu_median_s']:.6f} | {d['peak_rss_bytes']/2**20:.2f} |")
    (root / 'report.md').write_text('\n'.join(lines) + '\n')
    (root / 'summary.json').write_text(json.dumps(details, indent=2) + '\n')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--duckdb', required=True)
    p.add_argument('--rudb', required=True)
    p.add_argument('--data', type=Path, required=True, help='directory containing hits-1k.parquet etc')
    p.add_argument('--output', type=Path, required=True, help='new output directory; export SQL into its sql/ directory first')
    p.add_argument('--sizes', nargs='+', default=['1k', '10k', '100k', '1m'])
    p.add_argument('--hot', type=int, default=5)
    p.add_argument('--timeout', type=float, default=120)
    a = p.parse_args()
    if sys.platform != 'linux' or a.hot < 5 or a.timeout <= 0:
        p.error('Linux, at least five hot runs, and a positive timeout are required')
    root = a.output.resolve()
    root.mkdir(parents=True, exist_ok=True)
    log = (root / 'raw.jsonl').open('x')
    records = []

    def save(r):
        records.append(r)
        log.write(json.dumps(r) + '\n')
        log.flush()

    binaries = {e: str(Path(getattr(a, e)).resolve()) for e in ['duckdb', 'rudb']}
    meta = dict(start_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
                platform=platform.platform(), cpu_count=os.cpu_count(),
                helper_sha256=digest(HELPER), cpuinfo=Path('/proc/cpuinfo').read_text(), meminfo=Path('/proc/meminfo').read_text(),
                load_start=os.getloadavg(), argv=sys.argv, binaries={e: dict(path=b, sha256=digest(b),
                version=subprocess.check_output([b, '--version'], text=True).strip()) for e, b in binaries.items()},
                cache='first/unflushed; fresh process for every repetition', hot_runs=a.hot,
                sql_sha256={f.name: digest(f) for f in (root / 'sql').glob('*.sql')}, datasets={})
    schema = (root / 'sql/schema.sql').read_text()
    projection = (root / 'sql/projection.sql').read_text()
    for size in a.sizes:
        source = (a.data / f'hits-{size}.parquet').resolve()
        source_sql = str(source).replace("'", "''")
        scan = f"SELECT {projection} FROM read_parquet('{source_sql}', binary_as_string=True)"
        count = int(subprocess.check_output([binaries['duckdb'], '-csv', '-noheader', '-c', f"SELECT count(*) FROM read_parquet('{source_sql}')"], text=True))
        meta['datasets'][size] = dict(path=str(source), rows=count, bytes=source.stat().st_size, sha256=digest(source))
        (root / 'metadata.json').write_text(json.dumps(meta, indent=2) + '\n')
        database = root / f'{size}.duckdb'
        load = measure([binaries['duckdb'], '-batch', str(database), '-c', f'CREATE TABLE hits ({schema}); INSERT INTO hits {scan}; CHECKPOINT;'], root / f'{size}-load', a.timeout)
        load.update(size=size, engine='duckdb', phase='load', database_bytes=database.stat().st_size if database.exists() else None)
        save(load)
        if load['status'] != 'ok':
            raise RuntimeError(f'load failed: {load}')
        settings = subprocess.check_output([binaries['duckdb'], str(database), '-csv', '-c', "SELECT name,value FROM duckdb_settings() WHERE name IN ('threads','memory_limit','temp_directory','preserve_insertion_order')"], text=True)
        meta['datasets'][size]['duckdb_settings'] = settings
        for q in range(1, 44):
            sql = (root / 'sql' / f'q{q}.sql').read_text()
            for engine in (['duckdb', 'rudb'] if q % 2 else ['rudb', 'duckdb']):
                for run in range(a.hot + 1):
                    command = [binaries[engine], '-batch', '-csv', '-noheader']
                    if engine == 'duckdb':
                        command += [str(database)]
                    command += ['-c', '.timer on']
                    if engine == 'rudb':
                        command += ['-c', f'CREATE VIEW hits AS {scan}']
                    command += ['-c', sql]
                    r = measure(command, root / f'{size}-{engine}-q{q}-r{run}', a.timeout)
                    r.update(size=size, engine=engine, query=q, run=run, phase='query', load=os.getloadavg())
                    save(r)
                    if r['status'] != 'ok':
                        break
            print(f'{size} q{q}/43 done', flush=True)
        render(root, records, a.sizes, a.hot)
        # The native database is scratch; its size and load metrics are retained.
        database.unlink()
    meta.update(end_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()), load_end=os.getloadavg(), meminfo_end=Path('/proc/meminfo').read_text())
    (root / 'metadata.json').write_text(json.dumps(meta, indent=2) + '\n')
    log.close()


if __name__ == '__main__':
    main()
