"""The JOB gate of spec/bench/job/15-the-gate.md section 15.1, run on one machine.

  python3 -I job-gate.py load CONFIG DIR
  python3 -I job-gate.py run CONFIG DIR OUT.tsv [THREADS] [ONLY]

`load` builds both databases for one configuration in DIR from the 21 CSVs, the same SQL for both
engines: JOB's schema.sql, one COPY per table, and fkindexes.sql in the indexed configuration. rudb
is then handed the fifteen relationships and checkpointed, inside its timed region, the way the
DuckDB load ends with a checkpoint too. The two load times go in DIR/load-CONFIG.json.

`run` times the 113 queries the way section 14.3 says. For each query the two engines take turns,
the one going first alternating, and a turn is: drop the page cache, record every file in DIR with
its size, start the engine on its database, capture EXPLAIN, run the query six times with the timer
on, capture EXPLAIN again, and check that no file in DIR grew or appeared. The first run is the
cold one and the other five are the hot ones, reported as a median and an IQR. Every answer is
checked against DuckDB's, every run against the first.

The verdict lines at the end are the eight conditions of 15.1. The ones a configuration other than
plain is not gated on are printed and not judged.

Environment: JOB_DUCKDB, JOB_RUDB (binaries), JOB_CSV (the CSV directory), JOB_QUERIES (the query
directory, default queries/job beside this script), JOB_RUDB_SET (statements sent to rudb only
before every query, on top of what the configuration sends).
"""
import json, os, re, subprocess, statistics, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
QUERIES = os.environ.get('JOB_QUERIES', os.path.join(HERE, '..', 'queries', 'job'))
DUCKDB = os.environ.get('JOB_DUCKDB', 'duckdb')
RUDB = os.environ.get('JOB_RUDB', 'rudb')
CSV = os.environ.get('JOB_CSV', '')
CONFIGS = ('plain', 'indexed', 'clustered', 'norule')
LINKS = ("SET graph_links = 'cast_info(movie_id) -> title(id), cast_info(person_id) -> name(id), "
         "cast_info(person_role_id) -> char_name(id), movie_info(movie_id) -> title(id), "
         "movie_info_idx(movie_id) -> title(id), movie_companies(movie_id) -> title(id), "
         "movie_companies(company_id) -> company_name(id), movie_keyword(movie_id) -> title(id), "
         "movie_keyword(keyword_id) -> keyword(id), movie_link(movie_id) -> title(id), "
         "movie_link(linked_movie_id) -> title(id), complete_cast(movie_id) -> title(id), "
         "aka_name(person_id) -> name(id), aka_title(movie_id) -> title(id), "
         "person_info(person_id) -> name(id)'")
MARK = 'job-gate-mark'
RUNS = 6


def tables():
    schema = open(os.path.join(QUERIES, 'schema.sql')).read()
    return re.findall(r'CREATE TABLE (\w+)', schema)


def first_indexed():
    """The first indexed column of each child table, in fkindexes.sql order."""
    got = {}
    for line in open(os.path.join(QUERIES, 'fkindexes.sql')):
        m = re.search(r'on (\w+)\s*\((\w+)\)', line, re.I)
        if m and m.group(1) not in got:
            got[m.group(1)] = m.group(2)
    return got


def load_sql(config):
    """The statements that build one configuration, the same for both engines."""
    out = [open(os.path.join(QUERIES, 'schema.sql')).read()]
    order = first_indexed() if config == 'clustered' else {}
    for t in tables():
        path = os.path.join(CSV, t + '.csv')
        copy = f"COPY {t} FROM '{path}' (FORMAT csv, HEADER false, ESCAPE '\\', QUOTE '\"', NULL '')"
        if t in order:
            # A child table goes in sorted by its first indexed column, through a table of the
            # same shape, which is the price of file order section 9.3 asks to see.
            out.append(f"CREATE TABLE {t}_in AS SELECT * FROM {t} LIMIT 0;")
            out.append(copy.replace(f'COPY {t} ', f'COPY {t}_in ') + ';')
            out.append(f"INSERT INTO {t} SELECT * FROM {t}_in ORDER BY {order[t]};")
            out.append(f"DROP TABLE {t}_in;")
        else:
            out.append(copy + ';')
    if config == 'indexed':
        out.append(open(os.path.join(QUERIES, 'fkindexes.sql')).read())
    return '\n'.join(out) + '\n'


def files(d):
    got = {}
    for root, _, names in os.walk(d):
        for n in names:
            p = os.path.join(root, n)
            try:
                got[p] = os.path.getsize(p)
            except OSError:
                pass
    return got


def databases(config, d):
    base = 'plain' if config == 'norule' else config
    return os.path.join(d, f'duck-{base}.duckdb'), os.path.join(d, f'rudb-{base}.rudb')


def load(config, d):
    os.makedirs(d, exist_ok=True)
    duck, rudb = databases(config, d)
    sql = load_sql(config)
    took = {}
    for name, binary, db, tail in (('duckdb', DUCKDB, duck, 'CHECKPOINT;\n'),
                                   ('rudb', RUDB, rudb, LINKS + ';\nCHECKPOINT;\n')):
        for p in (db, db + '.wal'):
            if os.path.exists(p):
                os.remove(p)
        subprocess.run(['sh', '-c', 'sync'])
        start = time.monotonic()
        p = subprocess.run([binary, db], input=sql + tail, capture_output=True, text=True)
        took[name] = time.monotonic() - start
        if p.returncode != 0 or 'Error' in p.stderr:
            sys.exit(f'{name} load failed: {p.stderr.strip()[-400:]}')
        print(f'{name} loaded {config} in {took[name]:.1f} s, {os.path.getsize(db) / 1e9:.2f} GB',
              flush=True)
    with open(os.path.join(d, f'load-{config}.json'), 'w') as f:
        json.dump(took, f)


def quart(xs):
    if len(xs) < 2:
        return float('nan')
    q = statistics.quantiles(xs, n=4)
    return q[2] - q[0]


def turn(engine, config, d, sql, threads):
    """One engine's turn at one query: the cold run, five hot runs and the checks around them."""
    duck, rudb = databases(config, d)
    binary, db = (DUCKDB, duck) if engine == 'duckdb' else (RUDB, rudb)
    pre = [f'SET threads={threads};']
    if engine == 'rudb':
        pre.append('SET stored_answers = false;')
        if config == 'norule':
            pre.append("SET \"plan.consistent\" = false;")
        pre += [s.strip() + ';' for s in os.environ.get('JOB_RUDB_SET', '').split(';') if s.strip()]
    mark = f"SELECT '{MARK}';"
    script = '\n'.join(pre + ['.mode list', '.separator |', '.nullvalue NULL', '.headers off',
                              mark, f'EXPLAIN {sql};', mark, '.timer on'] + [f'{sql};'] * RUNS
                       + ['.timer off', mark, f'EXPLAIN {sql};', mark]) + '\n'
    subprocess.run(['sh', '-c', 'sync; echo 3 > /proc/sys/vm/drop_caches'])
    before = files(d)
    try:
        p = subprocess.run([binary, '-readonly', db], input=script, capture_output=True, text=True,
                           timeout=600 * RUNS)
        out, err = p.stdout, p.stderr
    except subprocess.TimeoutExpired:
        return {'error': 'timeout'}
    after = files(d)
    grown = sorted(f for f in after if f not in before or after[f] > before[f])
    times = [float(m.group(1)) * 1e3 for line in (out + '\n' + err).splitlines()
             if (m := re.match(r'Run Time \(s\): real ([\d.]+)', line))]
    parts = out.split(MARK + '\n')
    if len(parts) != 5 or len(times) != RUNS:
        return {'error': f'{len(times)} runs: ' + err.strip()[-300:].replace('\n', ' ')}
    plan_before, middle, plan_after = parts[1], parts[2], parts[3]
    rows = [line for line in middle.splitlines() if not line.startswith('Run Time')]
    n = len(rows) // RUNS if rows else 0
    answers = [tuple(rows[i * n:(i + 1) * n]) for i in range(RUNS)] if n else [()] * RUNS
    return {'times': times, 'answer': answers[0], 'stable': len(set(answers)) == 1,
            'plan_stable': plan_before == plan_after, 'grown': grown}


def names():
    got = [f[:-4] for f in os.listdir(QUERIES) if re.fullmatch(r'\d+[a-z]\.sql', f)]
    return sorted(got, key=lambda n: (int(n[:-1]), n[-1]))


def run(config, d, out, threads, only):
    rows, worst = [], (0.0, '')
    total = {'duckdb': 0.0, 'rudb': 0.0}
    wrong, unstable, grew, slower, over = [], [], [], [], []
    with open(out, 'w') as f:
        f.write('query\tduck_cold\tduck_hot\tduck_iqr\trudb_cold\trudb_hot\trudb_iqr\tratio\tcheck\n')
        for at, q in enumerate(names()):
            if only and q not in only:
                continue
            sql = open(os.path.join(QUERIES, q + '.sql')).read().strip().rstrip(';')
            order = ('duckdb', 'rudb') if at % 2 == 0 else ('rudb', 'duckdb')
            got = {e: turn(e, config, d, sql, threads) for e in order}
            if any('error' in g for g in got.values()):
                why = '; '.join(f"{e} {g['error']}" for e, g in got.items() if 'error' in g)
                f.write(f'{q}\t\t\t\t\t\t\t\terror {why}\n')
                print(f'{q} error {why}'[:200], flush=True)
                wrong.append(q)
                continue
            hot = {e: statistics.median(g['times'][1:]) for e, g in got.items()}
            ratio = hot['rudb'] / hot['duckdb']
            notes = []
            if got['rudb']['answer'] != got['duckdb']['answer']:
                notes.append('wrong')
                wrong.append(q)
            if not (got['rudb']['stable'] and got['rudb']['plan_stable']):
                notes.append('unstable')
                unstable.append(q)
            if got['rudb']['grown'] or got['duckdb']['grown']:
                notes.append('grew ' + ','.join(got['rudb']['grown'] + got['duckdb']['grown']))
                grew.append(q)
            if ratio > 1:
                slower.append(q)
            if ratio > 1 / 3:
                over.append(q)
            worst = max(worst, (ratio, q))
            for e in total:
                total[e] += hot[e]
            g = got
            line = (f"{q}\t{g['duckdb']['times'][0]:.1f}\t{hot['duckdb']:.1f}"
                    f"\t{quart(g['duckdb']['times'][1:]):.1f}\t{g['rudb']['times'][0]:.1f}"
                    f"\t{hot['rudb']:.1f}\t{quart(g['rudb']['times'][1:]):.1f}\t{ratio:.3f}"
                    f"\t{' '.join(notes) or 'ok'}")
            f.write(line + '\n')
            print(line, flush=True)
    ratio = total['rudb'] / total['duckdb'] if total['duckdb'] else float('nan')
    print(f"total {config} threads={threads}: duckdb {total['duckdb']:.0f} ms, rudb "
          f"{total['rudb']:.0f} ms, {1 / ratio:.2f}x, max per query ratio {worst[0]:.3f} "
          f"({worst[1]})")
    load_at = os.path.join(d, f'load-{config if config != "norule" else "plain"}.json')
    took = json.load(open(load_at)) if os.path.exists(load_at) else None
    verdict = [
        ('1 answers equal DuckDB', not wrong, ' '.join(wrong)),
        ('2 hot total at most a tenth', ratio <= 0.1, f'{ratio:.3f}'),
        ('3 every query at most a third', not over, ' '.join(over)),
        ('4 no query slower', not slower, ' '.join(slower)),
        ('5 plans and answers identical across runs', not unstable, ' '.join(unstable)),
        ('6 no file grew or appeared', not grew, ' '.join(grew)),
        ('7 counter assertions', None, 'not yet reported by the engine'),
        ('8 load no slower than DuckDB', took and took['rudb'] <= took['duckdb'],
         f"rudb {took['rudb']:.1f} s, duckdb {took['duckdb']:.1f} s" if took else 'no load'),
    ]
    for name, ok, why in verdict:
        state = 'pass' if ok else ('n/a' if ok is None else 'FAIL')
        if config != 'plain' and name[0] in '23':
            state = 'reported'
        print(f'{state:8} {name}: {why}'[:300])


def main():
    if len(sys.argv) < 4 or sys.argv[1] not in ('load', 'run') or sys.argv[2] not in CONFIGS:
        sys.exit(__doc__)
    what, config, d = sys.argv[1:4]
    if what == 'load':
        load(config, d)
    else:
        threads = int(sys.argv[5]) if len(sys.argv) > 5 else os.cpu_count()
        only = set(sys.argv[6].split(',')) if len(sys.argv) > 6 else None
        run(config, d, sys.argv[4], threads, only)


if __name__ == '__main__':
    main()
