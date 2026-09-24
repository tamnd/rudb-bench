#!/usr/bin/env python3
"""Compare two engine binaries over a suite by instructions retired.

Wall clock on a shared machine measures the machine. Instructions retired does not: the count a
query takes is the same whether the box is idle or carrying a load average of twenty, so a change
worth a percent can be told apart from noise on a box that is also building something. That is the
only instrument this script uses, and it is why the numbers it prints are comparable across runs
taken days apart. `reports/2026-09-24/tpch-instructions-retired.md` is the argument for it at
length, and this is the script that report asked for.

Two rudb binaries is the common use, which is whether a change made the engine do less work. rudb
against DuckDB is the other one, and it is the same measurement with a different command line, so
`--before-engine` and `--after-engine` pick how each side is invoked rather than there being a
second script.

The protocol is one query per process, one thread, several alternating rounds with the median of
each side taken, and the instruction count of `SELECT 1` on the same binary subtracted so what is
left is the query rather than the process start up. Alternating rather than all of one side and then
all of the other, because anything that drifts over the run drifts across both sides equally that
way.

Answers are compared before anything is measured, and a single query whose output changed stops the
run. A faster wrong answer is not a result. Two different engines format their output differently
and will not agree on a single row, so `--skip-answers` is there for that case and the harness
proper is what checks two engines agree.

Needs `perf` and the permission to read the instructions counter, which on most boxes means running
as root or setting `perf_event_paranoid` below two.
"""

import argparse
import pathlib
import re
import statistics
import subprocess
import sys


def queries(path, only):
    """The named queries of a file holding one query a line under a `-- qNN` comment.

    The file is `spec/graph/tpch.sql` in the rudb tree, or any file written the same way. A query is
    one line there on purpose, so this does not have to count semicolons to find where one ends.
    """
    named = []
    name = None
    for line in path.read_text().splitlines():
        line = line.strip()
        marked = re.fullmatch(r'-- (q\d+)', line)
        if marked:
            name = marked.group(1)
        elif line and not line.startswith('--') and name:
            named.append((name, line))
            name = None
    if only:
        wanted = set(only)
        named = [pair for pair in named if pair[0] in wanted]
        missing = wanted - {name for name, _ in named}
        if missing:
            sys.exit(f'no such query in {path}: {" ".join(sorted(missing))}')
    if not named:
        sys.exit(f'no queries found in {path}')
    return named


def instructions(stderr):
    """The count out of `perf stat -x,` output, or `None` when there is not one.

    The line is `<count>,,instructions,<enabled>,<ran>,,` and the count can be `<not counted>` when
    the counter was not available, which is a different failure from the command failing and is
    worth telling apart.
    """
    for line in stderr.splitlines():
        parts = line.split(',')
        if len(parts) > 2 and parts[2].strip() == 'instructions':
            try:
                return int(parts[0])
            except ValueError:
                return None
    return None


def command(engine, binary, database, sql, threads):
    """The command line one engine wants for one query in one process.

    rudb takes the thread count as an option so that it is set before the database is opened. DuckDB
    has no such option and takes a `SET` in front of the query instead, which is the same thing for
    a single statement since there is nothing before it to be affected.
    """
    if engine == 'rudb':
        return [str(binary), '--set', f'threads={threads}', '-readonly', str(database), '-c', sql]
    return [str(binary), '-readonly', str(database), '-c', f'SET threads={threads}; {sql}']


def ran(engine, binary, database, sql, threads):
    """One query in one new process, as a count of instructions and the answer it printed."""
    made = subprocess.run(
        ['perf', 'stat', '-e', 'instructions', '-x,']
        + command(engine, binary, database, sql, threads),
        capture_output=True, text=True, check=False)
    if made.returncode:
        sys.exit(f'{binary} failed on {sql[:70]}: {made.stderr[-600:]}')
    counted = instructions(made.stderr)
    if counted is None:
        sys.exit(f'no instruction count from perf, is the counter readable here: {made.stderr[-600:]}')
    return counted, made.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--before', required=True, type=pathlib.Path,
                        help='the baseline binary')
    parser.add_argument('--after', required=True, type=pathlib.Path,
                        help='the binary under test')
    parser.add_argument('--before-engine', default='rudb', choices=('rudb', 'duckdb'),
                        help='how to invoke the baseline binary')
    parser.add_argument('--after-engine', default='rudb', choices=('rudb', 'duckdb'),
                        help='how to invoke the binary under test')
    parser.add_argument('--database', required=True, type=pathlib.Path,
                        help='the database file the baseline reads, holding the suite at some scale')
    parser.add_argument('--after-database', type=pathlib.Path,
                        help='a second file for the side under test, when the format or engine differs')
    parser.add_argument('--queries', required=True, type=pathlib.Path,
                        help='the query file, one query a line under a -- qNN comment')
    parser.add_argument('--only', nargs='+', metavar='qNN',
                        help='measure only these queries rather than all of them')
    parser.add_argument('--rounds', type=int, default=3,
                        help='alternating rounds a query, the median of each side taken')
    parser.add_argument('--threads', type=int, default=1,
                        help='what to SET threads to, one by default so the count is one core')
    parser.add_argument('--moved', type=float, default=0.005,
                        help='how far from 1.000x a query has to be to get marked')
    parser.add_argument('--skip-answers', action='store_true',
                        help='measure without comparing the answers first, for a known rewrite')
    args = parser.parse_args()
    if args.rounds < 1:
        parser.error('--rounds must be positive')
    if sys.platform != 'linux':
        parser.error('perf is Linux only')

    named = queries(args.queries, args.only)
    after_database = args.after_database or args.database
    sides = [('before', args.before_engine, args.before, args.database),
             ('after', args.after_engine, args.after, after_database)]

    print(f'{len(named)} queries, {args.rounds} rounds, threads={args.threads}')
    base = {}
    for side, engine, binary, database in sides:
        base[side] = statistics.median(
            ran(engine, binary, database, 'SELECT 1', args.threads)[0]
            for _ in range(args.rounds))
    print(f'SELECT 1  before {base["before"] / 1e6:.1f}M  after {base["after"] / 1e6:.1f}M')

    if args.skip_answers:
        print('not comparing answers, --skip-answers was given')
    else:
        changed = []
        for name, sql in named:
            answers = {side: ran(engine, binary, database, sql, args.threads)[1]
                       for side, engine, binary, database in sides}
            if answers['before'] != answers['after']:
                changed.append(name)
        if changed:
            sys.exit(f'answers changed on {" ".join(changed)}, not measuring')
        print(f'all {len(named)} answers unchanged')

    print()
    total = {'before': 0.0, 'after': 0.0}
    for name, sql in named:
        counted = {'before': [], 'after': []}
        for _ in range(args.rounds):
            for side, engine, binary, database in sides:
                counted[side].append(ran(engine, binary, database, sql, args.threads)[0])
        took = {side: statistics.median(counted[side]) - base[side] for side in counted}
        for side in total:
            total[side] += took[side]
        ratio = took['after'] / took['before'] if took['before'] else 0.0
        mark = '  <<<' if abs(ratio - 1.0) > args.moved else ''
        print(f'{name}  {took["before"] / 1e6:9.1f}M  {took["after"] / 1e6:9.1f}M  {ratio:.3f}x{mark}')
    ratio = total['after'] / total['before'] if total['before'] else 0.0
    print(f'\nsuite  {total["before"] / 1e9:.2f}G  {total["after"] / 1e9:.2f}G  {ratio:.3f}x')


if __name__ == '__main__':
    main()
