# Ten million rows under the official driver

rudb cannot load the full ClickBench file, which is [rudb#745](https://github.com/tamnd/rudb/issues/745), so ten million rows is the largest size it can be measured at today. This is the same fork of the official driver and the same machine as the other two reports from this week, over a fresh one in every ten stride of the real file, 9,999,750 rows. All three engines finished. The raw driver files are in [ten-million](ten-million) beside this one.

Ten million is a tenth of the benchmark and it is also the first size in this series where the engines are doing enough work per query to be using the machine, so it is the most informative number rudb has. It is still not a ClickBench result and nothing here should be quoted as one.

## What ran

| engine | version | load | size on disk | cold total | hot total |
| --- | --- | --- | --- | --- | --- |
| rudb | 0.3.31 | 32.918s | 4,945,075,143 | 62.772s | 3.423s |
| duckdb | v2.0.0-dev84237 (Development Version) cc7e7bac7f | 13.623s | 2,473,340,928 | 3.974s | 2.850s |
| clickhouse | 26.9.1.1162 | 8.209s | 1,977,178,878 | 3.416s | 1.642s |

Forty three queries each, no nulls, all three exit zero. ClickHouse ran against the same private instance on port 9001 as the other runs this week, for the same reason, with the same settings.

## The warm number

rudb finishes the suite in 1.20x DuckDB's hot time and 2.09x ClickHouse's, and wins eleven of the forty three against DuckDB and four against ClickHouse.

At a million rows yesterday it was 1.07x DuckDB. Ten times the data moved it to 1.20x. The fit off the sample ladder said rudb's per row cost was 1.8 times DuckDB's, which would have put it near 1.8x here once the fixed cost stopped mattering, and it did not. That is the second thing this week that the sample fit got wrong in the same direction, and the reason is the same one: below a million rows none of these engines is using the machine, so a constant fitted there is not the constant that governs a real run.

## The cold number, which is the finding

| engine | cold | hot | cold over hot |
| --- | --- | --- | --- |
| rudb | 62.772s | 3.423s | 18.3x |
| duckdb | 3.974s | 2.850s | 1.4x |
| clickhouse | 3.416s | 1.642s | 2.1x |

Same machine, same `drop_caches` between the same queries, same virtual disk. DuckDB pays 40% for a first touch, ClickHouse pays 108%, rudb pays 1734%.

As the full file report says, `drop_caches` inside WSL2 only empties the guest page cache and the Windows host still has the blocks, so this column is a floor and not a cold measurement. That makes the gap worse rather than better. What is left in the column after the host cache absorbs the disk is mostly the cost of issuing the reads, so a column that is eighteen times the warm number is saying rudb issues far more read calls for the same bytes, not that it waits on a slower disk. Against a disk that has to be asked, the same pattern costs more.

The per query shape agrees with that reading. The queries where rudb's cold and hot numbers are close are the ones that touch almost nothing: q16 and q36 at 1.0x, q37 at 1.1x, q38 at 1.6x. The ones that read the wide string columns are the worst: q11 at 67x, q25 at 70x, q32 at 65x, q27 at 65x, q23 at 39x with 4.834s cold against 0.123s hot. Cold cost tracks bytes touched, which is what it should do, but the constant in front of it is more than an order of magnitude too large.

The likely cause is in the shape of the file rather than in the read path, and it is the same shape that causes #745. A stripe in the native format is one appended chunk, and the loader appends a vector at a time, so ten million rows is about ten thousand stripes and a hundred million is about a hundred thousand. A four byte column in one stripe is four kilobytes on disk. `flush_pending` does group thirty two stripes into an extent and lays each column out contiguously inside it, which is what keeps this from being much worse, but thirty two vectors is still only thirty two thousand rows, so a column scan is issuing hundreds of small reads where it could be issuing a handful of large ones. Cutting stripes at their own row count rather than at the caller's chunk boundary is the fix suggested for the directory bound in #745, and if the reading above is right it fixes this at the same time.

That paragraph turned out to be wrong and the correction is worth leaving next to it rather than replacing it. rudb#752 made the change it describes, a stripe of sixty four parts with one page per column per stripe, and measured the same suite on the same machine: cold went from 62.772s to 61.817s and hot from 3.423s to 3.567s. The reasoning missed that the extent already grouped thirty two chunks and already laid each column out contiguously inside the extent, so sequential scans were already getting one run per thirty two chunks and the change only doubled that. The eighteen times is somewhere else, and [the full file report](the-full-file.md) has a stronger version of the same question on a database that does not fit in memory. That run is in [ten-million/rudb-10m-v10.csv](ten-million/rudb-10m-v10.csv) next to the original.

This is filed separately as [rudb#748](https://github.com/tamnd/rudb/issues/748) so that it does not get lost inside the directory bound.

## Load and size, which is the other half of #745

rudb loads ten million rows in 32.918s against DuckDB's 13.623s and ClickHouse's 8.209s, and at a million rows yesterday it was 3.039s, so its load is linear in rows with no sign of bending. Straight out that is about 330s at a hundred million against DuckDB's measured 40.834s.

The size is the interesting one. rudb's database is 4,945,075,143 bytes against DuckDB's 2,473,340,928, so exactly twice. At a million rows it was 561,185,011 against 525,611,008, which is 1.07x. DuckDB's file grew 4.7x for ten times the rows because its compression gets better with scale, and rudb's grew 8.8x because it barely does. Another factor of 8.8 puts rudb near 43.5 GB at a hundred million, and the load that died in #745 had written 42,218,416,686 bytes when it stopped, which is the same number. So the full rudb database would be a little over twice DuckDB's 20.46 GB, and that is worth knowing independently of whether it can be written at all.

## Where the warm time goes

The worst queries for rudb against DuckDB, on the hot number, are the cheap ones. q7, the `MIN(EventDate), MAX(EventDate)`, is 8.97x at 0.018s against 0.002s. Then q39 at 5.5x, q38 at 4.0x, q37 at 3.76x, q25 at 3.23x, q27 at 3.11x, q40 at 2.7x, and none of them is above 0.051s. Together the ten worst ratios account for 0.46s of the 0.573s total gap, which is real, but they are ratios on queries where DuckDB is already answering in single digit milliseconds.

The wins are larger in absolute terms and they come from one place. q16, the `COUNT(*)` grouped by UserID, is 0.0005s against DuckDB's 0.077s. q36, the same thing over ClientIP and three derived columns, is 0.0006s against 0.070s. q34 and q35, grouped by URL, are 0.012s against 0.205s and 0.221s. Those four are answered out of a per column frequency structure that the loader materialises, which is why they do not scale with rows and why the numbers look impossible. It is a real design choice with a real price, paid in the 32.918s load and in the file being twice the size, and it should be read as that rather than as a scan being fast.

It is worth seeing how much of the total those four carry. rudb is 1.128s behind over the thirty two queries it loses and 0.555s ahead over the eleven it wins, and 1.128 less 0.555 is the 0.573s that separates the two totals. Put the won queries back at DuckDB's cost and the same run reads 1.40x rather than 1.20x. The total is a difference of two much larger numbers and anyone quoting it should know that.

rudb's own expensive end is q29 at 0.399s, q22 at 0.313s, q33 at 0.213s, q19 at 0.198s and q28 at 0.183s, which is roughly the same five that the full file report found for DuckDB and ClickHouse. The suite concentrates in the same place for everybody.

## Correctness

The driver does not check answers, so the four queries that looked too fast were checked by hand against DuckDB over the same ten million rows. q16, q34, q35 and q36 come back byte identical.

q23 came back in a different order, which turned out to be ties. Its `ORDER BY c DESC LIMIT 10` cuts through a run of rows with the same count, so which of them appears is not determined by the query. Re-running it as `ORDER BY c DESC, SearchPhrase LIMIT 14` gives identical rows, counts and distinct counts from both engines, and the filtered set is 725 rows over 506 distinct phrases in both. The queries are the board's and they are not modified, so this is a property of the benchmark rather than something to fix.

## What this does not say

Ten million rows is not ClickBench. The cold column is a floor and not a cold measurement, for the reason above. This is one machine and rule seven holds. The load average on the way in was decaying from the job that generated the sample, between 4 and 8 on a machine with 32 threads, which is not the quiet box the other runs this week had, so the warm totals here are worth no more than two significant figures.

## Reproducing it

```
duckdb -c "COPY (SELECT * EXCLUDE (file_row_number) FROM read_parquet('hits.parquet', file_row_number = true) WHERE file_row_number % 10 = 0) TO 'hits-10m.parquet' (FORMAT parquet, COMPRESSION snappy, ROW_GROUP_SIZE 8192)"
export BENCH_SKIP_DOWNLOAD=yes
export BENCH_CONCURRENT_DURATION=0
cd rudb && cp ~/rudb-data/clickbench/hits-10m.parquet hits.parquet && ./benchmark.sh
```
