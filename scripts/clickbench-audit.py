#!/usr/bin/env python3
"""ClickBench size ladder with raw wait4 measurements and all 43 queries.

First means first execution in a fresh process, not a flushed OS page cache.
Every hot repetition also starts a fresh process. Results are fully rendered.
DuckDB and rudb load native tables once; both also read the source Parquet per query.
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
SQL_ERROR = re.compile(r'^(?:[A-Za-z][A-Za-z ]{0,79} )?Error:', re.M)


def run_order(cases, run):
    """Reverse neighboring rounds, rotating after each pair of rounds."""
    offset = (run // 2) % len(cases)
    ordered = cases[offset:] + cases[:offset]
    return ordered if run % 2 == 0 else ordered[::-1]


def wait_for_capacity(max_load, timeout):
    """Wait outside the measured interval; preserve incomplete runs on timeout."""
    start = time.monotonic()
    last_notice = start - 30
    while max_load is not None and os.getloadavg()[0] > max_load:
        now = time.monotonic()
        if now - start > timeout:
            raise RuntimeError('host never reached the required capacity; this run is incomplete')
        if now - last_notice >= 30:
            print(f'Waiting: load {os.getloadavg()[0]:.2f}, limit {max_load:.2f}', flush=True)
            last_notice = now
        time.sleep(1)
    return time.monotonic() - start


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
                                 env={**os.environ, 'LC_ALL': 'C', 'TZ': 'UTC',
                                      'RUDB_PARQUET_MIRROR': '0'})

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
    if result['status'] == 'ok' and SQL_ERROR.search(stderr):
        result['status'] = 'query_error'
        result['query_s'] = None
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
             f'All 43 SQL queries are attempted on DuckDB native, rudb native, DuckDB Parquet, and rudb Parquet. {hot} hot repetitions are required for a complete row. A failed repetition invalidates that query; successful fragments are never averaged into a result.', '',
             'First execution is not disk-cold: the page cache is not flushed. Each repetition uses a fresh process. Query seconds are the CLI timer, including result rendering; wall and CPU seconds and peak RSS cover the whole child process. CPU and RSS come from wait4 for that child. RSS is the maximum resident set, not allocated bytes or an incremental memory delta. Both native rows open a loaded single-file database. Both Parquet rows query the same source file with native mirroring disabled; metadata-only paths remain available. These are sample results, not official ClickBench scores.', '',
             '| Size | Engine | Load wall (s) | Load CPU (s) | Load peak RSS (MiB) | Native bytes |',
             '| --- | --- | ---: | ---: | ---: | ---: |']
    for size in sizes:
        for engine in ['duckdb-native', 'rudb-native']:
            load = next((r for r in records if r.get('size') == size and r.get('engine') == engine and r.get('phase') == 'load'), None)
            if load:
                rss = load['peak_rss_bytes'] / 2**20 if load['peak_rss_bytes'] is not None else float('nan')
                lines.append(f"| {size} | {engine} | {load['wall_s']:.6f} | {load['cpu_s']:.6f} | {rss:.2f} | {load['database_bytes']} |")
    lines += ['',
             '| Size | Engine | Complete / 43 | Query median sum (s) | Process wall median sum (s) | CPU median sum (s) | Peak RSS (MiB) |',
             '| --- | --- | ---: | ---: | ---: | ---: | ---: |']
    checks_path = root / 'correctness.json'
    checks = {(c['size'], c['query']): c['result'] for c in json.loads(checks_path.read_text())} if checks_path.exists() else {}
    details = []
    for size in sizes:
        if not any(r.get('size') == size and r.get('phase') == 'query' for r in records):
            continue
        for engine in ['duckdb-native', 'rudb-native', 'duckdb-parquet', 'rudb-parquet']:
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
            group = [[r for r in records if r.get('size') == size and r.get('engine') == e and r.get('query') == q and r.get('run') == 0] for e in ['duckdb-native', 'rudb-native', 'duckdb-parquet', 'rudb-parquet']]
            if not all(group):
                continue
            rows = [part[0] for part in group]
            if any(row['status'] != 'ok' for row in rows):
                check = ', '.join(f"{row['engine']} {row['status']}" for row in rows)
            elif all(equal(answer(root / rows[0]['stdout']), answer(root / row['stdout'])) for row in rows[1:]):
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
    p.add_argument('--threads', type=int, default=6)
    p.add_argument('--memory-limit', default='4GB')
    p.add_argument('--max-load', type=float, help='wait before each child until the one-minute load is at most this value')
    p.add_argument('--idle-timeout', type=float, default=600)
    a = p.parse_args()
    if sys.platform not in ('linux', 'darwin') or a.hot < 5 or a.timeout <= 0 or a.threads < 1:
        p.error('Linux or macOS, at least five hot runs, positive threads and timeout are required')
    if not math.isfinite(a.idle_timeout) or a.idle_timeout <= 0 or (a.max_load is not None and (not math.isfinite(a.max_load) or a.max_load <= 0)):
        p.error('capacity limits must be finite and positive')
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
                helper_sha256=digest(HELPER), cpuinfo=Path('/proc/cpuinfo').read_text() if sys.platform == 'linux' else None, meminfo=Path('/proc/meminfo').read_text() if sys.platform == 'linux' else None,
                load_start=os.getloadavg(), argv=sys.argv, binaries={e: dict(path=b, sha256=digest(b),
                version=subprocess.check_output([b, '--version'], text=True).strip()) for e, b in binaries.items()},
                cache='first/unflushed; fresh process for every repetition', hot_runs=a.hot,
                threads=a.threads, memory_limit=a.memory_limit, stored_answers=False,
                parquet_mirror=False,
                max_load=a.max_load, idle_timeout=a.idle_timeout,
                order='rotate after each pair of rounds; reverse the second round',
                sql_sha256={f.name: digest(f) for f in (root / 'sql').glob('*.sql')}, datasets={})
    settings_sql = f"SET threads={a.threads}; SET memory_limit='{a.memory_limit.replace(chr(39), chr(39)*2)}'"
    schema = (root / 'sql/schema.sql').read_text()
    projection = (root / 'sql/projection.sql').read_text()
    for size in a.sizes:
        source = (a.data / f'hits-{size}.parquet').resolve()
        source_sql = str(source).replace("'", "''")
        scan = f"SELECT {projection} FROM read_parquet('{source_sql}', binary_as_string=True)"
        count = int(subprocess.check_output([binaries['duckdb'], '-csv', '-noheader', '-c', f"SELECT count(*) FROM read_parquet('{source_sql}')"], text=True))
        meta['datasets'][size] = dict(path=str(source), rows=count, bytes=source.stat().st_size, sha256=digest(source))
        (root / 'metadata.json').write_text(json.dumps(meta, indent=2) + '\n')
        databases = {'duckdb-native': root / f'{size}.duckdb', 'rudb-native': root / f'{size}.rudb'}
        load_sql = f'CREATE TABLE hits ({schema}); INSERT INTO hits {scan}; CHECKPOINT;'
        for engine, database in databases.items():
            binary = binaries['duckdb'] if engine == 'duckdb-native' else binaries['rudb']
            idle_wait = wait_for_capacity(a.max_load, a.idle_timeout)
            load_before = os.getloadavg()
            load = measure([binary, '-batch', str(database), '-c', settings_sql, '-c', load_sql], root / f'{size}-{engine}-load', a.timeout)
            load.update(size=size, engine=engine, phase='load', database_bytes=database.stat().st_size if database.exists() else None,
                        load_sql=load_sql, idle_wait_s=idle_wait, load_before=load_before, load_after=os.getloadavg())
            save(load)
            if load['status'] != 'ok':
                raise RuntimeError(f'{engine} load failed: {load}')
        settings = subprocess.check_output([binaries['duckdb'], str(databases['duckdb-native']), '-csv', '-c', settings_sql, '-c', "SELECT name,value FROM duckdb_settings() WHERE name IN ('threads','memory_limit','temp_directory','preserve_insertion_order')"], text=True)
        meta['datasets'][size]['duckdb_settings'] = settings
        meta['datasets'][size]['rudb_settings'] = subprocess.check_output(
            [binaries['rudb'], str(databases['rudb-native']), '-csv', '-c', settings_sql,
             '-c', 'SET stored_answers=false', '-c',
             "SELECT current_setting('threads'), current_setting('memory_limit'), current_setting('stored_answers')"],
            text=True)
        for q in range(1, 44):
            sql = (root / 'sql' / f'q{q}.sql').read_text()
            engines = ['duckdb-native', 'rudb-native', 'duckdb-parquet', 'rudb-parquet']
            failed = set()
            for run in range(a.hot + 1):
                for engine in run_order(engines, run + 2*q):
                    if engine in failed:
                        continue
                    binary = binaries['rudb'] if engine.startswith('rudb') else binaries['duckdb']
                    command = [binary, '-batch', '-csv', '-noheader']
                    if engine in databases:
                        command += [str(databases[engine])]
                    command += ['-c', settings_sql]
                    if engine.startswith('rudb'):
                        command += ['-c', 'SET stored_answers=false']
                    command += ['-c', '.timer on']
                    if engine.endswith('parquet'):
                        command += ['-c', f'CREATE VIEW hits AS {scan}']
                    command += ['-c', sql]
                    idle_wait = wait_for_capacity(a.max_load, a.idle_timeout)
                    load_before = os.getloadavg()
                    r = measure(command, root / f'{size}-{engine}-q{q}-r{run}', a.timeout)
                    r.update(size=size, engine=engine, query=q, run=run, phase='query', load=os.getloadavg(),
                             idle_wait_s=idle_wait, load_before=load_before, load_after=os.getloadavg())
                    save(r)
                    if r['status'] != 'ok':
                        failed.add(engine)
            print(f'{size} q{q}/43 done', flush=True)
        render(root, records, a.sizes, a.hot)
        # Native files stay beside the audit so deterministic retests open the exact timed files.
    meta.update(end_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()), load_end=os.getloadavg(), meminfo_end=Path('/proc/meminfo').read_text() if sys.platform == 'linux' else None)
    (root / 'metadata.json').write_text(json.dumps(meta, indent=2) + '\n')
    log.close()


if __name__ == '__main__':
    main()
