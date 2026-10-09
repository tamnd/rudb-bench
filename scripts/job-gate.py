"""The JOB gate of spec/bench/job/15-the-gate.md section 15.1, run on one machine.

  python3 -I job-gate.py load CONFIG DIR
  python3 -I job-gate.py run CONFIG DIR OUT.tsv [THREADS] [ONLY]
  python3 -I job-gate.py qerror CONFIG DIR OUT.tsv [THREADS] [ONLY]
  python3 -I job-gate.py truecounts CONFIG DIR OUT.tsv [THREADS] [ONLY]
  python3 -I job-gate.py perror CONFIG DIR OUT.tsv [THREADS] [ONLY]

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

Condition 7 reads three things off rudb. The EXPLAIN before the runs must hold no estimate taken from
a default. An EXPLAIN ANALYZE after the runs, outside the timer, must say every consistent reduction
kept its key sets as bitmaps, with no keys hashed. And every string column a query compares with a
constant must be stored on codes in every part, either against the table's dictionary or as
compressed text, which pragma_storage_info says.

`qerror` grades both engines' row estimates, which is section 12.8 of the JOB notes. Each query runs
once per engine, outside any timing, with rudb writing its metrics document and DuckDB writing its
JSON profile, and every operator with an estimate is compared with the rows it produced. The q-error
is the larger of the two over the smaller, with one row as the floor. The joins are summarized on
their own as well as with everything else, and the top join of each query on its own again. Both
engines filter a scan under a hash join by the keys the other side found, so an operator under a
join can produce fewer rows than the subplan it is the root of would, and its q-error is partly the
filter working and not only the estimate being wrong. Nothing filters the top join, so its q-error
is the estimate against the true size of the whole join.

`truecounts` counts the rows of every join a plan for a query could hold: the join of every
connected set of the query's aliases, filters and all, and each alias on its own after its filters.
A count is not taken by running the join, since some of these joins, three fact tables on the movie
and nothing else, run to trillions of rows. It is taken down a join tree instead, with each alias
counting its rows per join key and handing the counts to its parent, which multiplies them in, so no
join result is ever built. Every JOB query is acyclic, which is what makes that exact. rudb does the
counting, one process per query, and a query is written to OUT.tsv only when all of its counts are
in, so a run that stops can be started again and goes on from the first query not in the file.

`perror` grades the join order each engine picks against the order the true counts would pick. Each
query is planned once per engine outside any timing. rudb's join tree is read off its EXPLAIN, which
names every alias. DuckDB's is read off its profile, which names only tables, so its scans are
matched to the query's aliases by table, by every join in the tree joining aliases a condition
connects, and where that leaves a choice by the literals of each alias's filters the scan mentions.
A semi join an engine adds as a reduction, and an aggregation pushed under a join, are passed
through, so the tree graded is the tree of inner joins. The cost of a tree is the sum of the true
counts of its joins, C_out, and the best tree is found by dynamic programming over the same counts,
bushy and with no cross products, over the aliases the engine's tree reads. The P-error is the cost
of the engine's tree over the cost of the best one, with one row as the floor the way the q-error
has it, so 1 is the best order there is. The counts come from the `truecounts` file named by
JOB_COUNTS.

Environment: JOB_DUCKDB, JOB_RUDB (binaries), JOB_CSV (the CSV directory), JOB_QUERIES (the query
directory, default queries/job beside this script), JOB_COUNTS (the `truecounts` file `perror`
reads), JOB_RUDB_SET (statements sent to rudb only before every query, on top of what the
configuration sends), JOB_EVICT (set to `files` on a machine other people use, so the cold run
evicts only the two database files from the page cache rather than dropping all of it).
"""
import itertools, json, os, re, shutil, subprocess, statistics, sys, tempfile, time

HERE = os.path.dirname(os.path.abspath(__file__))
QUERIES = os.environ.get('JOB_QUERIES', os.path.join(HERE, '..', 'queries', 'job'))
DUCKDB = os.environ.get('JOB_DUCKDB', 'duckdb')
RUDB = os.environ.get('JOB_RUDB', 'rudb')
CSV = os.environ.get('JOB_CSV', '')
COUNTS = os.environ.get('JOB_COUNTS', '')
EVICT = os.environ.get('JOB_EVICT', '')
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
            if os.path.isdir(p):
                shutil.rmtree(p)
            elif os.path.exists(p):
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
    # rudb is asked for EXPLAIN ANALYZE once more after the timed runs, for the counters of
    # condition 7, which reads them off the plan text.
    analyze = [f'EXPLAIN ANALYZE {sql};', mark] if engine == 'rudb' else []
    script = '\n'.join(pre + ['.mode list', '.separator |', '.nullvalue NULL', '.headers off',
                              mark, f'EXPLAIN {sql};', mark, '.timer on'] + [f'{sql};'] * RUNS
                       + ['.timer off', mark, f'EXPLAIN {sql};', mark] + analyze) + '\n'
    if EVICT == 'files':
        for path in (duck, rudb):
            fd = os.open(path, os.O_RDONLY)
            os.posix_fadvise(fd, 0, 0, os.POSIX_FADV_DONTNEED)
            os.close(fd)
    else:
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
    if len(parts) != 5 + bool(analyze) or len(times) != RUNS:
        return {'error': f'{len(times)} runs: ' + err.strip()[-300:].replace('\n', ' ')}
    plan_before, middle, plan_after = parts[1], parts[2], parts[3]
    rows = [line for line in middle.splitlines() if not line.startswith('Run Time')]
    n = len(rows) // RUNS if rows else 0
    answers = [tuple(rows[i * n:(i + 1) * n]) for i in range(RUNS)] if n else [()] * RUNS
    analyzed = parts[4] if analyze else ''
    return {'times': times, 'answer': answers[0], 'stable': len(set(answers)) == 1,
            'plan_stable': plan_before == plan_after, 'grown': grown,
            'defaults': plan_before.count('estimated from default'),
            'sets': 'key sets' in analyzed, 'hashed': 'keys hashed' in analyzed}


def filter_columns():
    """The (table, column) pairs the 113 queries compare with a string, by their aliases."""
    got = set()
    for q in names():
        sql = open(os.path.join(QUERIES, q + '.sql')).read()
        alias = {a: t for t, a in re.findall(r'\b(\w+)\s+AS\s+(\w+)', sql, re.I)}
        compared = (r"\b(\w+)\.(\w+)\s*(?:NOT\s+)?(?:=|<>|!=|<=|>=|<|>|LIKE\b|IN\b|BETWEEN\b)"
                    r"\s*\(?\s*'")
        for a, c in re.findall(compared, sql, re.I):
            if a in alias:
                got.add((alias[a], c))
    return sorted(got)


def coded(config, d):
    """Each string filter column's parts: (column, parts not answered on codes, parts on the
    table's dictionary, parts). A part is answered on codes when it is coded against the
    table's dictionary, whose ranks order its codes, when it is coded against a dictionary of
    its own, which the reader hands on as codes over the part's values, or when it is
    compressed text, whose pages answer a LIKE and an equality on their codes. The writer drops
    the dictionary of a column whose values are nearly all different and demotes one that
    outgrows its budget, so not every filter column has one, and condition 7 asks only that
    none is read as plain strings."""
    _, rudb = databases(config, d)
    lines = ['.mode list', '.separator |', '.headers off']
    for t, c in filter_columns():
        lines.append(f"SELECT '{t}.{c}', sum(CASE WHEN compression LIKE 'TABLE DICT%' OR "
                     f"compression LIKE 'DICT%' OR compression LIKE 'FSST%' THEN 0 ELSE 1 "
                     f"END), sum(CASE WHEN compression LIKE 'TABLE DICT%' THEN 1 ELSE 0 END), "
                     f"count(*) FROM "
                     f"pragma_storage_info('{t}') WHERE column_name = '{c}';")
    p = subprocess.run([RUDB, '-readonly', rudb], input='\n'.join(lines) + '\n',
                       capture_output=True, text=True, timeout=600)
    got = []
    for line in p.stdout.splitlines():
        cells = line.split('|')
        if len(cells) == 4 and all(x.isdigit() for x in cells[1:]):
            got.append((cells[0], int(cells[1]), int(cells[2]), int(cells[3])))
    return got


def names():
    got = [f[:-4] for f in os.listdir(QUERIES) if re.fullmatch(r'\d+[a-z]\.sql', f)]
    return sorted(got, key=lambda n: (int(n[:-1]), n[-1]))


def run(config, d, out, threads, only):
    rows, worst = [], (0.0, '')
    total = {'duckdb': 0.0, 'rudb': 0.0}
    wrong, unstable, grew, slower, over = [], [], [], [], []
    defaulted, hashed, reported = [], [], 0
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
            if got['rudb']['defaults']:
                notes.append(f"default {got['rudb']['defaults']}")
                defaulted.append(q)
            if got['rudb']['hashed']:
                notes.append('hashed')
                hashed.append(q)
            reported += got['rudb']['sets']
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
    columns = coded(config, d)
    plain = [c for c, bad, _, parts in columns if bad or not parts]
    for c, bad, dictionary, parts in columns:
        print(f'column {c}: {parts} parts, {dictionary} on the table dictionary, '
              f'{bad} not on codes')
    counters = [f'default in {" ".join(defaulted)}' if defaulted else '',
                f'hashed in {" ".join(hashed)}' if hashed else '',
                'no key sets reported' if not reported else '',
                f'plain strings in {" ".join(plain)}' if plain else '',
                'no filter column measured' if not columns else '']
    counters = '; '.join(x for x in counters if x)
    verdict = [
        ('1 answers equal DuckDB', not wrong, ' '.join(wrong)),
        ('2 hot total at most a tenth', ratio <= 0.1, f'{ratio:.3f}'),
        ('3 every query at most a third', not over, ' '.join(over)),
        ('4 no query slower', not slower, ' '.join(slower)),
        ('5 plans and answers identical across runs', not unstable, ' '.join(unstable)),
        ('6 no file grew or appeared', not grew, ' '.join(grew)),
        ('7 counter assertions', not counters,
         counters or f'no default, {reported} reductions all dense, {len(columns)} filter '
                     'columns on codes'),
        ('8 load no slower than DuckDB', took and took['rudb'] <= took['duckdb'],
         f"rudb {took['rudb']:.1f} s, duckdb {took['duckdb']:.1f} s" if took else 'no load'),
    ]
    for name, ok, why in verdict:
        state = 'pass' if ok else ('n/a' if ok is None else 'FAIL')
        if config != 'plain' and name[0] in '23':
            state = 'reported'
        print(f'{state:8} {name}: {why}'[:300])


def q_error(estimated, produced):
    high, low = max(estimated, produced, 1), max(min(estimated, produced), 1)
    return high / low


def joins(kind):
    return 'JOIN' in kind.upper() or kind in ('Probe', 'NestedLoop')


def graded(engine, config, d, sql, threads):
    """What `graded_once` finds, asked up to three times. A document cut short does not parse, and
    on a shared machine with a nearly full disk that happens to a run now and then, so the query is
    run again rather than ending every query after it. `None` if no run left one that parses."""
    for _ in range(3):
        try:
            return graded_once(engine, config, d, sql, threads)
        except ValueError:
            continue
    return None


def rudb_settings(config, threads):
    pre = [f'SET threads={threads};', 'SET stored_answers = false;']
    if config == 'norule':
        pre.append("SET \"plan.consistent\" = false;")
    return pre + [s.strip() + ';' for s in os.environ.get('JOB_RUDB_SET', '').split(';')
                  if s.strip()]


def profiled(engine, config, d, sql, threads):
    """One run of a query outside any timing, and the profile it left: rudb's operators, or the
    root of DuckDB's JSON profile. `None` if it left none, and `ValueError` if it does not parse."""
    duck, rudb = databases(config, d)
    at = os.path.join(tempfile.gettempdir(), f'job-qerror-{os.getpid()}-{engine}.json')
    if os.path.exists(at):
        os.remove(at)
    pre = [f'SET threads={threads};']
    if engine == 'rudb':
        pre = rudb_settings(config, threads)
        command = [RUDB, '--metrics', at, '-readonly', rudb]
    else:
        pre += ["PRAGMA enable_profiling = 'json';", f"PRAGMA profiling_output = '{at}';"]
        command = [DUCKDB, '-readonly', duck]
    subprocess.run(command, input='\n'.join(pre + [f'{sql};']) + '\n', capture_output=True,
                   text=True, timeout=600)
    if not os.path.exists(at):
        return None
    try:
        if engine == 'rudb':
            lines = open(at).read().splitlines()
            return json.loads(lines[-1])['operators'] if lines else []
        return json.load(open(at))
    finally:
        os.remove(at)


def graded_once(engine, config, d, sql, threads):
    """(kind, estimated, produced, top) for every operator one engine had an estimate for, where
    top says it is a join with no join above it."""
    profile = profiled(engine, config, d, sql, threads)
    if profile is None:
        return None
    got = []
    if engine == 'rudb':
        operators = profile
        by_id = {o['id']: o for o in operators}

        def under_join(o):
            seen = set()
            while o.get('parent') is not None and o['parent'] not in seen:
                seen.add(o['parent'])
                o = by_id.get(o['parent'], {})
                if joins(o.get('kind', '')):
                    return True
            return False
        for o in operators:
            if o.get('estimated_rows') is not None:
                top = joins(o['kind']) and not under_join(o)
                got.append((o['kind'], int(o['estimated_rows']), int(o['rows_out']), top))
    else:
        def walk(node, below):
            extra = node.get('extra_info')
            estimate = extra.get('Estimated Cardinality') if isinstance(extra, dict) else None
            if estimate is not None and str(estimate).isdigit():
                top = joins(node['type']) and not below
                got.append((node['type'], int(estimate), int(node.get('intermediate_rows', 0)),
                            top))
            for child in node.get('children', []):
                walk(child, below or joins(node['type']))
        for node in profile.get('operator', []):
            walk(node, False)
    return got


def quantiles(xs):
    xs = sorted(xs)
    if not xs:
        return 'none'
    at = lambda share: xs[min(len(xs) - 1, int(round((len(xs) - 1) * share)))]
    return (f'{len(xs)} estimates, median {at(0.5):.1f}, 90th {at(0.9):.1f}, 99th {at(0.99):.1f}, '
            f'max {xs[-1]:.1f}')


def qerror(config, d, out, threads, only):
    every = {'duckdb': [], 'rudb': []}
    joined = {'duckdb': [], 'rudb': []}
    topmost = {'duckdb': [], 'rudb': []}
    better, worse = [], []
    with open(out, 'w') as f:
        f.write('query\tduck_joins\tduck_join_median\tduck_join_max\tduck_max\tduck_top'
                '\trudb_joins\trudb_join_median\trudb_join_max\trudb_max\trudb_top\n')
        for q in names():
            if only and q not in only:
                continue
            sql = open(os.path.join(QUERIES, q + '.sql')).read().strip().rstrip(';')
            got = {e: graded(e, config, d, sql, threads) for e in ('duckdb', 'rudb')}
            if any(g is None for g in got.values()):
                f.write(f'{q}\terror\n')
                print(f'{q} error', flush=True)
                continue
            cells = [q]
            worst = {}
            for e in ('duckdb', 'rudb'):
                all_q = [q_error(est, rows) for _, est, rows, _ in got[e]]
                join_q = sorted(q_error(est, rows) for kind, est, rows, _ in got[e] if joins(kind))
                top_q = [q_error(est, rows) for _, est, rows, top in got[e] if top]
                every[e] += all_q
                joined[e] += join_q
                topmost[e] += top_q
                worst[e] = join_q[-1] if join_q else 1.0
                median = join_q[(len(join_q) - 1) // 2] if join_q else 1.0
                cells += [str(len(join_q)), f'{median:.1f}', f'{worst[e]:.1f}',
                          f'{max(all_q, default=1.0):.1f}', f'{max(top_q, default=1.0):.1f}']
            (better if worst['rudb'] < worst['duckdb'] else worse).append(q)
            f.write('\t'.join(cells) + '\n')
            print('\t'.join(cells), flush=True)
    for e in ('duckdb', 'rudb'):
        print(f'{e} top joins: {quantiles(topmost[e])}')
        print(f'{e} joins: {quantiles(joined[e])}')
        print(f'{e} every operator: {quantiles(every[e])}')
    print(f'rudb worst join estimate closer than DuckDB in {len(better)} queries, not closer in '
          f'{len(worse)}')


def conjuncts(where):
    """The conditions joined by AND at the top of a WHERE clause, minding quotes, parentheses and
    the AND inside BETWEEN."""
    out, depth, start, quote, between, i = [], 0, 0, False, False, 0
    up = where.upper()
    word = lambda at, w: up.startswith(w, at) and (at == 0 or not (up[at - 1].isalnum() or
                                                                  up[at - 1] == '_')) and \
        (at + len(w) == len(up) or not (up[at + len(w)].isalnum() or up[at + len(w)] == '_'))
    while i < len(where):
        c = where[i]
        if quote:
            if c == "'" and where[i + 1:i + 2] == "'":
                i += 1
            elif c == "'":
                quote = False
        elif c == "'":
            quote = True
        elif c in '()':
            depth += 1 if c == '(' else -1
        elif depth == 0 and word(i, 'BETWEEN'):
            between = True
        elif depth == 0 and word(i, 'AND'):
            if between:
                between = False
            else:
                out.append(where[start:i].strip())
                start = i + 3
        i += 1
    out.append(where[start:].strip())
    return [x for x in out if x]


def query_graph(sql):
    """The aliases of a JOB query with their tables, its join conditions as pairs of
    (alias, column), and the filters on each alias."""
    sql = ' '.join(re.sub(r'--[^\n]*', ' ', sql).split()).rstrip(';')
    m = re.search(r'\bFROM\b(.*?)\bWHERE\b(.*)$', sql, re.I)
    aliases = {}
    for item in m.group(1).split(','):
        words = item.split()
        aliases[words[-1].strip('"')] = words[0]
    pairs, filters = [], {a: [] for a in aliases}
    for c in conjuncts(m.group(2)):
        j = re.fullmatch(r'\(?\s*"?(\w+)"?\.(\w+)\s*=\s*"?(\w+)"?\.(\w+)\s*\)?', c)
        if j and j.group(1) != j.group(3):
            pairs.append(((j.group(1), j.group(2)), (j.group(3), j.group(4))))
            continue
        used = set(re.findall(r'\b(\w+)"?\.\w+', re.sub(r"'(?:[^']|'')*'", "''", c))) & set(aliases)
        if len(used) != 1:
            raise ValueError(f'a condition on {len(used)} aliases: {c}')
        filters[used.pop()].append(c)
    return aliases, pairs, filters


def key_classes(pairs):
    """The columns the join conditions hold equal, as sets of (alias, column)."""
    parent = {}

    def find(x):
        parent.setdefault(x, x)
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x
    for a, b in pairs:
        parent[find(a)] = find(b)
    groups = {}
    for x in list(parent):
        groups.setdefault(find(x), set()).add(x)
    return list(groups.values())


def connected_sets(aliases, pairs):
    """Every set of aliases the join conditions connect, single aliases included, where two aliases
    are connected when any of their columns are held equal, written or implied."""
    near = {a: set() for a in aliases}
    for group in key_classes(pairs):
        members = {a for a, _ in group}
        for a in members:
            near[a] |= members - {a}
    seen = {frozenset([a]) for a in aliases}
    frontier = list(seen)
    while frontier:
        grown = []
        for s in frontier:
            for b in set().union(*(near[a] for a in s)) - s:
                t = s | {b}
                if t not in seen:
                    seen.add(t)
                    grown.append(t)
        frontier = grown
    return seen


def counting_sql(aliases, pairs, filters, s):
    """A statement counting the rows of the join of the aliases in `s`, with their filters and every
    condition between them. Each alias counts its rows per key of the join to its parent in a join
    tree, multiplied by the counts its children hand it, so no join result is built. None if the
    aliases in `s` are cyclic, where no such tree exists."""
    s = sorted(s)
    held = [g for g in key_classes(pairs) if len({a for a, _ in g if a in s}) > 1]
    keys = {a: {} for a in s}
    local = {a: list(filters[a]) for a in s}
    for i, group in enumerate(held):
        for a, column in sorted(group):
            if a not in s:
                continue
            if i in keys[a]:
                local[a].append(f'"{a}".{keys[a][i]} = "{a}".{column}')
            else:
                keys[a][i] = column
    # The GYO reduction leaves nothing exactly when the aliases are acyclic.
    left = {a: set(keys[a]) for a in s}
    while True:
        for a in left:
            left[a] = {k for k in left[a] if sum(k in e for e in left.values()) > 1}
        inside = next((a for a in left if any(b != a and left[a] <= left[b] for b in left)), None)
        if inside is None:
            break
        del left[inside]
    if len(left) > 1 or any(left.values()):
        return None
    # The heaviest spanning tree, weighed by the keys two aliases share, is a join tree.
    root = {a: a for a in s}

    def find(x):
        while root[x] != x:
            x = root[x]
        return x
    near = {a: [] for a in s}
    for _, a, b in sorted(((len(keys[a].keys() & keys[b].keys()), a, b)
                           for a, b in itertools.combinations(s, 2)), reverse=True):
        if keys[a].keys() & keys[b].keys() and find(a) != find(b):
            root[find(a)] = find(b)
            near[a].append(b)
            near[b].append(a)

    def node(a, parent):
        joined = f'{aliases[a]} AS "{a}"'
        weights = []
        for child in near[a]:
            if child != parent:
                on = ' AND '.join(f'"{a}".{keys[a][k]} = m_{child}.k{k}'
                                  for k in sorted(keys[a].keys() & keys[child].keys()))
                joined += f' JOIN ({node(child, a)}) AS m_{child} ON {on}'
                weights.append(f'm_{child}.w')
        where = f" WHERE {' AND '.join(local[a])}" if local[a] else ''
        if parent is None:
            counted = f"coalesce(sum({' * '.join(weights)}), 0)" if weights else 'count(*)'
            return f'SELECT {counted} FROM {joined}{where}'
        by = sorted(keys[a].keys() & keys[parent].keys())
        columns = ', '.join(f'"{a}".{keys[a][k]} AS k{k}' for k in by)
        group = ', '.join(f'"{a}".{keys[a][k]}' for k in by)
        counted = f"sum({' * '.join(weights)})" if weights else 'count(*)::DOUBLE'
        return f'SELECT {columns}, {counted} AS w FROM {joined}{where} GROUP BY {group}'
    return node(s[0], None)


def query_sql(q):
    return open(os.path.join(QUERIES, q + '.sql')).read().strip().rstrip(';')


def truecounts(config, d, out, threads, only):
    _, rudb = databases(config, d)
    done = set()
    if os.path.exists(out):
        done = {line.split('\t')[0] for line in open(out)}
    else:
        with open(out, 'w') as f:
            f.write('query\taliases\trows\n')
    for q in names():
        if (only and q not in only) or q in done:
            continue
        aliases, pairs, filters = query_graph(query_sql(q))
        sets = sorted(connected_sets(aliases, pairs), key=lambda s: (len(s), sorted(s)))
        statements = [counting_sql(aliases, pairs, filters, s) for s in sets]
        if None in statements:
            print(f'{q} cyclic, not counted', flush=True)
            continue
        began, got = time.time(), []
        for at in range(0, len(statements), 500):
            chunk = statements[at:at + 500]
            pre = [f'SET threads={threads};', '.mode csv', '.headers off']
            ran = subprocess.run([RUDB, '-readonly', rudb], capture_output=True, text=True,
                                 input='\n'.join(pre + [x + ';' for x in chunk]) + '\n',
                                 timeout=14400)
            rows = [line for line in ran.stdout.splitlines() if line.strip()]
            if len(rows) != len(chunk) or not all(re.fullmatch(r'[0-9.e+]+', r) for r in rows):
                got = None
                print(f'{q} counting failed: {ran.stderr.strip()[:300]}', flush=True)
                break
            got += [round(float(r)) for r in rows]
        if got is None:
            continue
        with open(out, 'a') as f:
            for s, n in zip(sets, got):
                f.write(f"{q}\t{','.join(sorted(s))}\t{n}\n")
        print(f'{q} {len(sets)} sets counted in {time.time() - began:.0f} s', flush=True)


def join_tree(engine, config, d, sql, threads):
    """The inner joins of the plan an engine picks, as a tree of ('join', left, right) over
    ('scan', table, alias, what). rudb's EXPLAIN names each alias. DuckDB's profile does not, so
    there the alias is None and `what` is the text of the scan with the filters between it and the
    join above, for `matched` to go on. A semi join is its kept side, a join with nothing read
    from a table on one side is its other side, and anything else with one input is that input, so
    the reductions an engine adds and an aggregation pushed under a join are passed through."""
    if engine == 'rudb':
        _, rudb = databases(config, d)
        ran = subprocess.run([RUDB, '-readonly', rudb], capture_output=True, text=True,
                             input='\n'.join(rudb_settings(config, threads) + [
                                 '.mode list', f'EXPLAIN {sql};']) + '\n', timeout=600)
        lines = ran.stdout.split('logical_plan|', 1)[-1].split('\n\n', 1)[0].splitlines()
        lines = [x for x in lines if x.strip()]

        def walk(at, depth):
            line = lines[at].strip()
            children, at = [], at + 1
            while at < len(lines) and len(lines[at]) - len(lines[at].lstrip()) == depth + 2:
                child, at = walk(at, depth + 2)
                children.append(child)
            get = re.match(r'Get \S+\.(\w+) AS "?(\w+)"? ', line)
            if get:
                return ('scan', get.group(1), get.group(2), None), at
            join = re.match(r'Join (\w+)', line)
            return reduce(join.group(0) if join else line.split()[0], children, ''), at
        if not lines:
            raise ValueError('no plan in EXPLAIN')
        return walk(0, 0)[0]
    profile = profiled(engine, config, d, sql, threads)

    def walk(n):
        kind = n.get('operator_type', n.get('type', ''))
        extra = n.get('extra_info') or {}
        if 'SCAN' in kind and isinstance(extra, dict) and extra.get('Table'):
            return ('scan', extra['Table'].replace('"', '').split('.')[-1], None,
                    json.dumps(extra))
        if 'JOIN' in kind and isinstance(extra, dict):
            kind = f"Join {extra.get('Join Type', 'INNER')}"
        return reduce(kind, [walk(c) for c in n.get('children', [])], json.dumps(extra))
    tops = [t for t in (walk(n) for n in profile.get('operator', [])) if t]
    if len(tops) != 1:
        raise ValueError(f'{len(tops)} trees in one plan')
    return tops[0]


def reduce(kind, children, text):
    if joins(kind) and len(children) == 2:
        kept = kind.split()[-1].upper()
        if 'SEMI' in kept or 'ANTI' in kept or 'MARK' in kept:
            return children[1] if kept.startswith('RIGHT') else children[0]
        if children[0] and children[1]:
            return ('join', children[0], children[1])
    children = [c for c in children if c]
    if len(children) > 1:
        raise ValueError(f'{kind} over {len(children)} inputs')
    if children and children[0][0] == 'scan' and children[0][3] is not None:
        return children[0][:3] + (children[0][3] + text,)
    return children[0] if children else None


def leaves(tree):
    return [tree] if tree[0] == 'scan' else leaves(tree[1]) + leaves(tree[2])


def matched(tree, aliases, filters, counted):
    """The aliases the scans of a tree read, as {id(scan): alias}, the C_out of the tree, and how
    many different costs the assignments that tie for the best have, so 1 means there was no doubt.
    An assignment has each scan read its table, or its alias where the plan names it, and every
    join join a set of aliases the conditions connect. The best is the one whose scans mention the
    most of the literals in their aliases' filters."""
    scans = leaves(tree)
    nodes = []

    def gather(t):
        if t[0] == 'join':
            gather(t[1])
            gather(t[2])
            nodes.append(t)
    gather(tree)

    def fits(scan, a):
        said = scan[3] or ''
        return sum(lit in said for c in filters[a] for lit in re.findall(r"'([^']+)'", c))
    found = []

    def place(i, given, used):
        if len(found) >= 5000:
            return
        if i == len(scans):
            sets = [frozenset(given[id(s)] for s in leaves(n)) for n in nodes]
            if all(s in counted for s in sets):
                found.append((sum(fits(s, given[id(s)]) for s in scans),
                              sum(counted[s] for s in sets), dict(given)))
            return
        for a in [scans[i][2]] if scans[i][2] else aliases:
            if a not in used and aliases.get(a) == scans[i][1]:
                given[id(scans[i])] = a
                place(i + 1, given, used | {a})
                del given[id(scans[i])]
    place(0, {}, frozenset())
    if not found:
        return None, 0, 0
    top = max(f[0] for f in found)
    best = [f for f in found if f[0] >= top - 1e-9]
    return best[0][2], best[0][1], len({f[1] for f in best})


def cheapest(reads, counted):
    """The least C_out of a bushy join tree with no cross products over the aliases `reads`."""
    bit = {a: 1 << i for i, a in enumerate(sorted(reads))}
    rows = {}
    for s, n in counted.items():
        if s <= reads:
            rows[sum(bit[a] for a in s)] = n
    best = {}
    for mask in sorted(rows, key=lambda m: bin(m).count('1')):
        if mask & (mask - 1) == 0:
            best[mask] = 0
            continue
        low, least = mask & -mask, None
        size = bin(mask).count('1')
        if 1 << size <= len(best):
            sub = (mask - 1) & mask
            candidates = []
            while sub:
                candidates.append(sub)
                sub = (sub - 1) & mask
        else:
            candidates = [m for m in best if m & mask == m and m != mask]
        for sub in candidates:
            if sub & low and sub in best and (mask ^ sub) in best:
                cost = best[sub] + best[mask ^ sub]
                if least is None or cost < least:
                    least = cost
        if least is not None:
            best[mask] = rows[mask] + least
    return best.get(sum(bit.values()))


def perror(config, d, out, threads, only):
    counts = {}
    for line in open(COUNTS):
        q, sets, n = line.rstrip('\n').split('\t')
        if q != 'query':
            counts.setdefault(q, {})[frozenset(sets.split(','))] = int(n)
    found = {'duckdb': [], 'rudb': []}
    better = same = worse = 0
    with open(out, 'w') as f:
        f.write('query\tduck_cost\tduck_best\tduck_perror\tduck_tied'
                '\trudb_cost\trudb_best\trudb_perror\trudb_tied\n')
        for q in names():
            if (only and q not in only) or q not in counts:
                continue
            aliases, _, filters = query_graph(query_sql(q))
            cells, graded_q = [q], {}
            for e in ('duckdb', 'rudb'):
                try:
                    tree = join_tree(e, config, d, query_sql(q), threads)
                    given, cost, tied = matched(tree, aliases, filters, counts[q])
                except (ValueError, TypeError, AttributeError) as why:
                    print(f'{q} {e}: {why}', flush=True)
                    given = None
                if given is None:
                    cells += ['error', '', '', '']
                    continue
                best = cheapest(frozenset(given.values()), counts[q])
                graded_q[e] = max(cost, 1) / max(best, 1)
                found[e].append(graded_q[e])
                cells += [str(cost), str(best), f'{graded_q[e]:.3f}', str(tied)]
            if len(graded_q) == 2:
                better += graded_q['rudb'] < graded_q['duckdb'] - 1e-9
                worse += graded_q['rudb'] > graded_q['duckdb'] + 1e-9
                same += abs(graded_q['rudb'] - graded_q['duckdb']) <= 1e-9
            f.write('\t'.join(cells) + '\n')
            print('\t'.join(cells), flush=True)
    for e in ('duckdb', 'rudb'):
        xs = sorted(found[e])
        at = lambda share: xs[min(len(xs) - 1, int(round((len(xs) - 1) * share)))] if xs else 0
        print(f'{e} P-error: {len(xs)} queries, median {at(0.5):.3f}, 90th {at(0.9):.3f}, '
              f'max {at(1.0):.3f}, at the best order {sum(x <= 1.0 + 1e-9 for x in xs)}, '
              f'within 2x {sum(x <= 2.0 for x in xs)}')
    print(f'rudb order closer to the best than DuckDB in {better} queries, as close in {same}, '
          f'further in {worse}')


def main():
    what = ('load', 'run', 'qerror', 'truecounts', 'perror')
    if len(sys.argv) < 4 or sys.argv[1] not in what or sys.argv[2] not in CONFIGS:
        sys.exit(__doc__)
    what, config, d = sys.argv[1:4]
    if what == 'load':
        load(config, d)
    else:
        threads = int(sys.argv[5]) if len(sys.argv) > 5 else os.cpu_count()
        only = set(sys.argv[6].split(',')) if len(sys.argv) > 6 else None
        {'run': run, 'qerror': qerror, 'truecounts': truecounts,
         'perror': perror}[what](config, d, sys.argv[4], threads, only)


if __name__ == '__main__':
    main()
