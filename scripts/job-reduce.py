"""Write the reduced form of every JOB query: a full semijoin reduction and a MIN per relation.

    python3 scripts/job-reduce.py queries/job queries/job-reduced

Each query becomes a chain of materialized CTEs. The first CTE of a relation is its scan with its
own filters and only the columns the rest needs. Then, over the join tree GYO finds, every edge is
a semijoin written as EXISTS, once from the leaves up to the root and once back down. The answer is
MIN of each output column over what is left of its relation, with no join executed. It is plain
SQL so that DuckDB can run it with its own operators, which makes it a check that the reduction
gives the same answers on the real data. It is not a prototype of the reducer's speed, because
every CTE is a copy and every semijoin is a hash join.

The corpus quotes the alias "at", which DuckDB reserves. The quotes are taken off while the query
is read and put back where the reduced form names the table, so the CTEs are called at_0 and so on.
"""
import collections
import glob
import os
import re
import sys


def split_and(s):
    """The top level conjuncts of a WHERE clause, keeping the AND inside a BETWEEN."""
    out, depth, cur, between = [], 0, '', False
    for t in re.split(r"(\(|\)|\bAND\b|\bBETWEEN\b|'[^']*')", s, flags=re.I):
        if t is None:
            continue
        u = t.upper()
        if t == '(':
            depth += 1
        elif t == ')':
            depth -= 1
        if u == 'BETWEEN' and depth == 0:
            between = True
        if u == 'AND' and depth == 0:
            if between:
                between = False
                cur += t
                continue
            out.append(cur.strip())
            cur = ''
            continue
        cur += t
    out.append(cur.strip())
    return [o for o in out if o]


def parse(path):
    sql = open(path).read().strip().rstrip(';')
    quoted = set(re.findall(r'"(\w+)"', sql))
    sql = re.sub(r'"(\w+)"', r'\1', sql)
    sel = re.search(r'SELECT(.*?)\bFROM\b', sql, re.S | re.I).group(1)
    frm = re.search(r'\bFROM\b(.*?)\bWHERE\b', sql, re.S | re.I).group(1)
    whr = re.search(r'\bWHERE\b(.*)$', sql, re.S | re.I).group(1)
    rels = {}
    for item in frm.split(','):
        words = item.split()
        rels[words[-1]] = words[0]
    aggs = re.findall(r'MIN\s*\(\s*(\w+)\.(\w+)\s*\)\s+AS\s+(\w+)', sel, re.I)
    joins, filters = [], collections.defaultdict(list)
    for c in split_and(whr):
        m = re.fullmatch(r'(\w+)\.(\w+)\s*=\s*(\w+)\.(\w+)', c.strip())
        if m and m.group(1) in rels and m.group(3) in rels and m.group(1) != m.group(3):
            joins.append(((m.group(1), m.group(2)), (m.group(3), m.group(4))))
            continue
        aliases = set(re.findall(r'\b(\w+)\.\w+', re.sub(r"'[^']*'", '', c)))
        if len(aliases) != 1:
            raise ValueError(f'{path}: a filter over more than one relation: {c}')
        filters[aliases.pop()].append(c)
    return rels, aggs, joins, filters, quoted


def plan(rels, joins):
    """The classes by union-find, then GYO: the join tree as (child, parent, classes) and the root."""
    parent = {}

    def find(x):
        parent.setdefault(x, x)
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x

    for a, b in joins:
        parent[find(a)] = find(b)
    classes = collections.defaultdict(set)
    for x in list(parent):
        classes[find(x)].add(x)
    column = collections.defaultdict(dict)
    for k, members in classes.items():
        for a, c in members:
            column[a].setdefault(k, c)
    edges = {r: set(column[r]) for r in rels}
    order = []
    while len(edges) > 1:
        count = collections.Counter(k for v in edges.values() for k in v)
        for r in edges:
            edges[r] = {k for k in edges[r] if count[k] > 1}
        # The ear with the fewest classes first, ties by name, so the output is the same every run.
        for r in sorted(edges, key=lambda r: (len(edges[r]), r)):
            over = [s for s in sorted(edges) if s != r and edges[r] <= edges[s]]
            if over:
                order.append((r, over[0], sorted(edges[r])))
                del edges[r]
                break
        else:
            raise ValueError('cyclic')
    return column, order, next(iter(edges))


def rewrite(path):
    rels, aggs, joins, filters, quoted = parse(path)
    column, order, _root = plan(rels, joins)
    need = collections.defaultdict(set)
    for a in rels:
        need[a] |= set(column[a].values())
    for a, c, _ in aggs:
        need[a].add(c)
    version = {a: 0 for a in rels}
    ctes = []
    for a, t in rels.items():
        cols = ', '.join(sorted(need[a])) or '1 AS one'
        where = (' WHERE ' + ' AND '.join(f'({c})' for c in filters[a])) if filters[a] else ''
        name = f'"{a}"' if a in quoted else a
        ctes.append(f'{a}_0 AS MATERIALIZED (SELECT {cols} FROM {t} AS {name}{where})')

    def semi(target, source, keys):
        cond = ' AND '.join(
            f'{source}_s.{column[source][k]} = {target}_t.{column[target][k]}' for k in keys
        )
        v = version[target]
        version[target] += 1
        ctes.append(
            f'{target}_{v + 1} AS MATERIALIZED (SELECT * FROM {target}_{v} AS {target}_t WHERE '
            f'EXISTS (SELECT 1 FROM {source}_{version[source]} AS {source}_s WHERE {cond}))'
        )

    for child, parent, keys in order:
        semi(parent, child, keys)
    for child, parent, keys in reversed(order):
        semi(child, parent, keys)
    outs = ', '.join(f'(SELECT MIN({c}) FROM {a}_{version[a]}) AS {n}' for a, c, n in aggs)
    return 'WITH ' + ',\n'.join(ctes) + f'\nSELECT {outs}\n'


def main():
    source, target = sys.argv[1:3]
    os.makedirs(target, exist_ok=True)
    paths = [p for p in glob.glob(os.path.join(source, '*.sql')) if re.fullmatch(r'\d+[a-z]\.sql', os.path.basename(p))]
    for path in paths:
        with open(os.path.join(target, os.path.basename(path)), 'w') as f:
            f.write(rewrite(path))
    print('wrote', len(paths))


if __name__ == '__main__':
    main()
