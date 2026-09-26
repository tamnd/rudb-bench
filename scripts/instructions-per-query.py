#!/usr/bin/env python3
"""Count the instructions one engine retires on every query of a suite.

`scripts/tpch-instructions-ab.py` compares two binaries and needs both databases on disk at once,
which a machine short of disk cannot always give it. This measures one side at a time with the same
protocol, so two runs of it against two engines give the same numbers the A/B script would: one
query per process, one thread, several rounds, the median taken, and the count of `SELECT 1` on the
same binary subtracted so what is left is the query rather than the process start up. It borrows the
query reader, the command lines and the perf parsing from that script rather than keeping a second
copy of them.

The count is of user space instructions only, `instructions:u`, so it is the engine's own work and
not the kernel's page faults and reads, which move with the page cache and with whatever else the
machine is doing. On a shared machine that makes it the one number here that does not move with
the load.

ClickHouse is the server row, started here from the config the harness wrote for it and kept with
`RUDB_BENCH_KEEP`, so it serves the table the harness loaded, sorting key and all. A new process a
query is not how ClickHouse is meant to be run and `clickhouse local` spends about five and a half
billion instructions starting, with a spread of a few hundred million between two starts, which is
more than a small query costs. So the server is started once, and each query is one `clickhouse
client` with `max_threads` set to the thread count while `perf stat -p` counts the server process
alone for as long as the client runs. The `SELECT 1` subtracted is then what a round trip costs
the server rather than what a process start costs.

Usage:

    cargo run --example export_queries -- clickbench duckdb /tmp/clickbench.sql
    scripts/instructions-per-query.py --engine duckdb --binary /usr/local/bin/duckdb \\
        --database /path/to/hits.duckdb --queries /tmp/clickbench.sql --rounds 3
    cargo run --example export_queries -- clickbench clickhouse-server /tmp/clickhouse.sql
    scripts/instructions-per-query.py --engine clickhouse --binary /usr/bin/clickhouse \\
        --database /path/to/scratch/clickhouse-server/config.xml --queries /tmp/clickhouse.sql
    cargo run --example export_queries -- clickbench rudb /tmp/rudb.sql
    scripts/instructions-per-query.py --engine rudb --binary rudb --database /path/to/hits.rudb \\
        --queries /tmp/rudb.sql --set stored_answers=false

rudb keeps summaries of each column and answers some ClickBench queries from them without reading
the rows. The harness turns that off for ClickBench with `RUDB_BENCH_STORED_ANSWERS=off`, and a count
taken here to go beside a harness run has to do the same with `--set stored_answers=false`, or those
queries come out at a million instructions instead of hundreds of millions.
"""

import argparse
import importlib.util
import os
import pathlib
import re
import statistics
import subprocess
import sys
import time

HERE = pathlib.Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location('ab', HERE / 'tpch-instructions-ab.py')
ab = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ab)

EVENT = 'instructions:u'


class Server:
    """A ClickHouse server over the directory the harness loaded, for as long as it is needed."""

    def __init__(self, binary, config):
        self.binary = binary
        self.port = re.search(r'<tcp_port>(\d+)</tcp_port>', config.read_text()).group(1)
        # No watchdog, so the process started here is the server and not a parent waiting on it,
        # which would count nothing.
        self.process = subprocess.Popen([str(binary), 'server', '--config-file', str(config)],
                                        env={**os.environ, 'CLICKHOUSE_WATCHDOG_ENABLE': '0'},
                                        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        for _ in range(120):
            if subprocess.run(self.client('SELECT 1'), capture_output=True,
                              check=False).returncode == 0:
                return
            if self.process.poll() is not None:
                sys.exit(f'the clickhouse server stopped, see the logs beside {config}')
            time.sleep(1)
        self.stop()
        sys.exit('the clickhouse server did not answer within two minutes')

    def client(self, sql, threads=None):
        command = [str(self.binary), 'client', '--host', '127.0.0.1', '--port', self.port,
                   '--format', 'CSV']
        if threads is not None:
            command.append(f'--max_threads={threads}')
        return command + ['--query', sql]

    def stop(self):
        self.process.terminate()
        self.process.wait()


def command(engine, binary, database, sql, threads, server, settings=()):
    """How to run one query and which process perf counts, `None` for the command itself.

    `settings` are rudb `--set` options, put after the thread count and before the database so they
    are in force for the query the way a `SET` in front of it would be.
    """
    if engine == 'clickhouse':
        return ['-p', str(server.process.pid), '--'] + server.client(sql, threads)
    made = ab.command(engine, binary, database, sql, threads)
    if engine == 'rudb':
        for setting in settings:
            made[3:3] = ['--set', setting]
    return made


def ran(engine, binary, database, sql, threads, server=None, settings=()):
    """One query, as the user space instructions of the process that ran it and its wall time."""
    started = time.monotonic()
    made = subprocess.run(['perf', 'stat', '-e', EVENT, '-x,']
                          + command(engine, binary, database, sql, threads, server, settings),
                          stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True, check=False)
    took = time.monotonic() - started
    if made.returncode:
        return None, took, made.stderr.strip()[-300:]
    for line in made.stderr.splitlines():
        parts = line.split(',')
        if len(parts) > 2 and parts[2].strip() == EVENT:
            try:
                return int(parts[0]), took, None
            except ValueError:
                break
    sys.exit(f'no instruction count from perf, is the counter readable here: {made.stderr[-600:]}')


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--engine', required=True, choices=('rudb', 'duckdb', 'clickhouse'),
                        help='how to invoke the binary')
    parser.add_argument('--binary', required=True, type=pathlib.Path, help='the engine binary')
    parser.add_argument('--database', required=True, type=pathlib.Path,
                        help='the database file holding the suite, or the server config for '
                             'ClickHouse')
    parser.add_argument('--queries', required=True, type=pathlib.Path,
                        help='the query file, one query a line under a -- qNN comment')
    parser.add_argument('--only', nargs='+', metavar='qNN', help='measure only these queries')
    parser.add_argument('--rounds', type=int, default=3, help='runs a query, the median taken')
    parser.add_argument('--threads', type=int, default=1, help='what to SET threads to')
    parser.add_argument('--set', action='append', default=[], metavar='NAME=VALUE',
                        help='a rudb setting for every query, such as stored_answers=false, which '
                             'the harness sets for a fair ClickBench run; may be given more than once')
    parser.add_argument('--tsv', type=pathlib.Path, help='also write the per query rows here')
    args = parser.parse_args()
    if args.set and args.engine != 'rudb':
        parser.error('--set is for rudb only')
    if args.rounds < 1:
        parser.error('--rounds must be positive')
    if sys.platform != 'linux':
        parser.error('perf is Linux only')

    named = ab.queries(args.queries, args.only)
    print(f'{args.engine}, {len(named)} queries, {args.rounds} rounds, threads={args.threads}, '
          f'settings {args.set or "none"}, {EVENT}')

    def rounds(sql):
        counts, seconds, loads = [], [], []
        for _ in range(args.rounds):
            loads.append(load_now())
            counted, took, why = ran(args.engine, args.binary, args.database, sql, args.threads,
                                     server, args.set)
            if counted is None:
                return None, why
            counts.append(counted)
            seconds.append(took)
        return (statistics.median(counts), min(seconds), max(loads)), None

    server = Server(args.binary, args.database) if args.engine == 'clickhouse' else None
    try:
        measure(args, named, rounds)
    finally:
        if server:
            server.stop()


def measure(args, named, rounds):
    """Every query of the suite, less `SELECT 1`, printed and written as it goes."""
    base, why = rounds('SELECT 1')
    if base is None:
        sys.exit(f'SELECT 1 failed: {why}')
    print(f'SELECT 1  {base[0] / 1e6:.1f}M, subtracted from every query below')
    rows = [('query', 'instructions', 'fastest_s', 'load')]
    total = 0.0
    for name, sql in named:
        measured, why = rounds(sql)
        if measured is None:
            print(f'{name}  failed: {why}', flush=True)
            rows.append((name, 'failed', '', ''))
            continue
        took = measured[0] - base[0]
        total += took
        rows.append((name, f'{took:.0f}', f'{measured[1]:.3f}', f'{measured[2]:.2f}'))
        print(f'{name}  {took / 1e6:.1f}M  fastest {measured[1]:.3f}s  load {measured[2]:.2f}',
              flush=True)
    print(f'\nsuite  {total / 1e9:.3f}G')
    if args.tsv:
        args.tsv.write_text(''.join('\t'.join(row) + '\n' for row in rows))


def load_now():
    """The one minute load average, which is kept beside each query's numbers."""
    with open('/proc/loadavg', encoding='ascii') as loadavg:
        return float(loadavg.read().split()[0])


if __name__ == '__main__':
    main()
