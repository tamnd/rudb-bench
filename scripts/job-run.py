"""Time the JOB queries on one engine through its CLI, one process per query.

  python3 -I run.py ENGINE_BINARY DB QUERYDIR OUT.tsv THREADS [RUNS] [ONLY]

Per query: drop the page cache, start the engine, run the query RUNS times in that process. The
first run is the cold one. The CLI's .timer gives each run's wall time. Output is one row per
query: cold ms, hot median ms, hot IQR ms, every sample, and the first answer row.
"""
import os, re, sys, glob, subprocess, statistics

binary, db, qdir, out, threads = sys.argv[1:6]
runs = int(sys.argv[6]) if len(sys.argv) > 6 else 6
only = set(sys.argv[7].split(',')) if len(sys.argv) > 7 else None
names = sorted((os.path.basename(p)[:-4] for p in glob.glob(qdir + '/*.sql') if re.fullmatch(r'\d+[a-z]', os.path.basename(p)[:-4])), key=lambda n: (int(n[:-1]), n[-1]))
if only:
    names = [n for n in names if n in only]

def quart(xs):
    if len(xs) < 2:
        return float('nan')
    q = statistics.quantiles(xs, n=4)
    return q[2] - q[0]

with open(out, 'w') as f:
    f.write('query\tcold_ms\thot_median_ms\thot_iqr_ms\tsamples_ms\tanswer\n')
    for n in names:
        sql = open(f'{qdir}/{n}.sql').read().strip().rstrip(';')
        script = f"SET threads={threads};\n.mode list\n.separator |\n.nullvalue NULL\n.headers off\n.timer on\n" + (sql + ';\n') * runs
        subprocess.run(['sh', '-c', 'sync; echo 3 > /proc/sys/vm/drop_caches'])
        try:
            p = subprocess.run([binary, '-readonly', db], input=script, capture_output=True, text=True, timeout=300 * runs)
            lines = p.stdout.splitlines()
            # DuckDB prints the timer on stdout and rudb 0.4.32 on stderr, so both are read.
            s = [float(m.group(1)) * 1e3 for l in lines + p.stderr.splitlines() if (m := re.match(r'Run Time \(s\): real ([\d.]+)', l))]
            rows = [l for l in lines if not l.startswith('Run Time')]
            answers = set(rows)
            ans = rows[0] if rows else '<none>'
            if len(answers) > 1:
                ans = 'UNSTABLE ' + ' / '.join(sorted(answers))
            if len(s) != runs:
                ans = f'<error {len(s)} runs> ' + p.stderr.strip()[-300:].replace('\n', ' ')
        except subprocess.TimeoutExpired:
            s, ans = [], '<timeout>'
        hot = s[1:]
        med = statistics.median(hot) if hot else float('nan')
        line = f"{n}\t{s[0] if s else float('nan'):.1f}\t{med:.1f}\t{quart(hot):.1f}\t{','.join(f'{x:.0f}' for x in s)}\t{ans}"
        print(line[:150], flush=True)
        f.write(line.replace('\n', ' ') + '\n')
