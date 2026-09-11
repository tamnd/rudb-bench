# M1: the format experiment

This is the report that milestone M1 of [rudb](https://github.com/tamnd/rudb/issues/2) exists to produce. The milestone was a measurement rather than a feature, and its exit criterion was a written report with the numbers and a decision. The numbers are below and so is the decision.

The short version is that the answer is no on all three of the questions the milestone was built to answer, and that the one thing which did work was not in the specification at all.

## The question

`spec/02-the-goal.md` claims 10x less resource than DuckDB, and the disk half of that claim rested on a specific technical bet in `spec/06-compression.md`. Section 6.4 said real tables have groups of columns drawn from the same vocabulary, so one dictionary shared between them beats one dictionary each. Section 6.5 said the same for a shared FSST symbol table. Section 6.6 said a column that is a function of another column need not be stored at all, only the rule that recomputes it. Those three together were described as the techniques that have no equivalent in DuckDB, which is to say they were the part of the resource claim that could not be copied.

`spec/19-open-questions.md` asked three questions that are all questions about data rather than about code. Q1 is whether multi-column compression delivers the ratios on real data. Q4 is whether recomputation rules pay for themselves. Q5 is whether a global dictionary can be built during a load without an unacceptable cost. A question about data can be answered by a program that is slow, single threaded and disposable, which is much cheaper than answering it by building the storage engine and finding out.

The exit criterion named a number. ClickBench `hits` under 4 GB was the target, under 3 GB meant the resource axis was on track, and 6 GB or above meant the resource claim was wrong and documents 00, 02 and 03 were to be amended before another line of storage code was written. Naming the failure number in advance is the only part of this that makes the result worth reading.

## The apparatus

`experiments/format-lab` in the rudb repository, which lives outside the published workspace so that it can depend on arrow-rs for Parquet reading without putting a dependency into a crate we ship. It reads a Parquet file a row group at a time and never holds the whole thing, because the `URL` column of `hits` does not fit in the memory of the machine this ran on. It has three passes.

`stats` encodes every column of every chunk with the cascade chooser from `rudb-encoding`, decodes each result back and compares it to what went in, and prints per column the distinct count, the chosen shape, the size Parquet used, the size rudb used and the ratio. Every number in the size table below came out of a pass with verification on, so none of the ratios are for output that does not read back.

`pairs` estimates, for every ordered pair of columns, the Jaccard overlap of their value sets and a dependence score, using the bottom-k sketch in `rudb-encoding::sketch` with k of 4096. That is where shared dictionary candidates and functional dependency candidates come from.

`groups` turns those two answers into bytes, which is the whole point of having it as a separate pass. A Jaccard score is not a saving. For sharing, it takes the groups found in the first chunk, holds them fixed, and encodes each group three ways on every chunk: independently, with one shared symbol table, and with one shared dictionary. All three totals get printed along with which strategy won how many chunks. For recomputation it takes rules named on the command line, builds the mapping in first appearance order of the determining column, prices the column against the mapping alone and against the mapping plus its keys, and counts violations while it builds. A rule with any violations at all is not a rule.

The rules have to be named by hand rather than discovered, which is deliberate. `pairs` looks at 5,460 pairs on `hits` and at that many pairs a sketch cannot tell a dependency from a coincidence. The pass that spends real machine time on one wants a human to have looked at it first.

Chunk size is 122,880 rows throughout, which is DuckDB's row group, so that the comparison is between two formats and not between two chunk sizes. The runs were on server1 (4 cores, 5 GB) and server3 (8 cores, 23 GB), both Linux, and every wall clock number below carries its machine with it because the two are not comparable.

## The corpora

`hits` from ClickBench, 99,997,497 rows and 105 columns, because it is the dataset the whole resource claim is aimed at and the one every rival publishes a size for. It is a wide, string heavy web log.

TPC-H at scale factor 100, generated with DuckDB's `dbgen` in a hundred partitions and written to one Parquet file per table with Snappy. It is the opposite shape: narrow, mostly numeric and decimal, with generated text that is deliberately unlike real text.

Three real Parquet files picked to be unlike `hits` on purpose. NYC yellow taxi for January 2024 is narrow, numeric and timestamped. The NYC high volume for hire vehicle file for the same month is the same shape at twenty million rows. An Ookla fixed broadband performance tile file for the first quarter of 2024 is a handful of numeric columns next to one column of well known text geometry, which is the case no dictionary is going to help with and is in the list for exactly that reason.

The choice matters because a compression result on one dataset is an anecdote. Three of these came from the internet as somebody else wrote them, with whatever writer and whatever settings they used, which is the case rudb will actually meet.

## What the size came out at

Ratios are rudb over Parquet, so under one is a win.

| corpus | rows | columns | codec | Parquet | rudb | ratio |
| ------ | ----- | ------- | ----- | ------- | ---- | ----- |
| ClickBench hits | 99,997,497 | 105 | Snappy | 13.76 GB | 9.65 GB | 0.70 |
| TPC-H lineitem | 600,037,902 | 16 | Snappy | 21.16 GB | 13.62 GB | 0.64 |
| TPC-H orders | 150,000,000 | 9 | Snappy | 6.05 GB | 3.49 GB | 0.58 |
| TPC-H partsupp | 80,000,000 | 5 | Snappy | 4.09 GB | 2.87 GB | 0.70 |
| TPC-H customer | 15,000,000 | 8 | Snappy | 1.15 GB | 807.45 MB | 0.68 |
| TPC-H part | 20,000,000 | 9 | Snappy | 603.36 MB | 338.10 MB | 0.56 |
| TPC-H supplier | 1,000,000 | 7 | Snappy | 75.42 MB | 50.90 MB | 0.67 |
| NYC yellow taxi 2024-01 | 2,964,624 | 9 of 19 | ZSTD-1 | 30.89 MB | 18.37 MB | 0.59 |
| NYC for hire vehicle 2024-01 | 19,663,930 | 15 of 24 | ZSTD-1 | 287.70 MB | 205.63 MB | 0.71 |
| Ookla fixed Q1 2024 | 6,655,986 | 9 of 11 | Snappy | 327.04 MB | 525.96 MB | 1.61 |

The `lineitem` row was written in after the rest of the table, which is what the first version of this report said would happen. It is the largest single thing the experiment has encoded, 600,037,902 rows over 4,884 chunks in 11,446.8 seconds of wall clock on server1, three hours and eleven minutes rather than the nine hours this report estimated, which was an arithmetic slip rather than a measurement: 600 million rows at the 52,254 rows a second that had been measured is three hours and change. Peak resident was 169.67 MB for a 21 GB file, and every chunk decoded back to what went in. It came out at 0.64, between the 0.58 of `orders` and the 0.70 of `partsupp`, which is where the guess said it would be.

Two columns in it are worth naming. `l_orderkey` came out at 0.08 of what Parquet stores it in, 892.02 MB against 74.32 MB, because the file is sorted by it and the chooser picked plain run length encoding over a hundred and fifty million distinct values. That is the largest single column ratio anywhere in this experiment and it is entirely a property of the sort order rather than of the encoder. `l_linenumber` came out at 1.44, the only column in the table that is worse than Parquet by more than a rounding error, and the shape the chooser picked for it is `DELTA(RLE(DICT(FOR+BITPACK[4], FOR+BITPACK[3]), FOR+BITPACK[3]))` on a column with seven distinct values that cycles one to seven. Parquet stores it in 140.37 MB and rudb in 201.98 MB. A column that counts one to seven and starts over should be the easiest thing in the file, and the chooser is picking five levels and losing, which is a chooser defect rather than a format limit and is the smallest reproducing case of one anywhere here.

Summed over everything in that table, 47.53 GB of Parquet becomes 31.58 GB of rudb, which is 0.66, which is 1.51 times smaller. Against DuckDB, whose published `hits` is 20.46 GB against our 9.65 GB, it is 2.12 times smaller. The claim in document 02 was ten.

The three rows with "of" in the column count are the honest part. Floating point columns are not encoded yet and are skipped, and they are skipped on both sides of the ratio, so those three ratios describe part of a file rather than a file. The part that is missing is not small: ten float columns of yellow taxi are 16.78 MB of a 47.65 MB file, nine of the for hire vehicle file are 163.09 MB of 450.86 MB, and two of Ookla are 18.35 MB of 345.42 MB. If floats were stored raw at the size Parquet already achieves for them, which is the most pessimistic reading, the whole file ratios would be 0.74, 0.82 and 1.58 rather than 0.59, 0.71 and 1.61. The TPC-H rows have no floats in them, because `dbgen` emits decimals, and `hits` has none either, so six of the nine rows are whole file numbers and three are not.

## Where `hits` went

Three columns are more than half the file. `URL` is 2.44 GB, `Referer` 1.93 GB and `OriginalURL` 1.74 GB in the first whole file pass, which is 6.11 GB of 11.65 GB.

That first pass came out at 11.65 GB, 0.85 of Parquet. The reason was not a missing cascade, which is what everybody assumed before looking at the shapes. The chooser was already picking `DICT(FSST[255](...), ...)` on all three, the dictionary entries were already going back through the chooser, and they were already coming out FSST. A 255 symbol table still loses to Snappy on URLs, and the reason is structural rather than a tuning problem: FSST compresses each string on its own, and a block compressor has the previous few kilobytes of the page to point back into. On a column where each URL shares a host and a path prefix with the URL next to it, that back reference is most of the redundancy in the column, and a per string symbol table cannot reach it.

The shape that reaches it is a sorted dictionary that is front coded, which is not a multi-column technique and is not in section 6.4. Adding it took the whole file from 11.65 GB to 9.65 GB. `URL` went from 0.99 of Parquet to 0.70, `Referer` from 0.92 to 0.67, and `OriginalURL` from 1.26 to 0.95, so the one column that was larger than what Parquet stored it in stopped being larger. `Title` came out at 0.44 and `SearchPhrase` at 0.59.

The shape the chooser picked for `URL`, which nobody wrote down in advance, is `DICT(FRONT(DICT(DELTA(RLE(FOR+BITPACK, FOR+BITPACK))), FSST[255](DICT(DELTA(RLE(...))))), RLE(...))`. Six levels, assembled one candidate at a time by a chooser that knows nothing about URLs. All 814 chunks of all 105 columns decoded back to exactly what went in.

Two gigabytes from front coding is the largest single result in this milestone, and front coding is a technique any format could adopt tomorrow.

## Q1, shared dictionaries and shared symbol tables

The answer is no, and it is not close.

On `hits`, exactly one pair of string columns out of 5,460 overlaps enough to be worth one dictionary, which is `UTMSource` and `UTMCampaign`.

```
| group                   |   apart | shared table | shared dict | best saves | picked                                      |
| ----------------------- | ------- | ------------ | ----------- | ---------- | ------------------------------------------- |
| UTMSource + UTMCampaign | 4.17 MB |      9.42 MB |     4.17 MB |       0.1% | apart 388, shared table 12, shared dict 414 |
```

Four megabytes of a 9.65 GB file, and sharing saves a tenth of a percent of the four. The shared symbol table is more than twice the size of encoding the two columns apart, which is what a joint 255 symbol table looks like when two columns turn out not to share an alphabet after all.

An earlier run of this pass said sixteen columns wanted one dictionary. That was an artifact of the lab storing a null as an empty string, so every column that is mostly null looked like every other column that is mostly null. Fixing it took the answer from sixteen to two, and it is worth recording because it is the shape of mistake that would have kept section 6.4 alive for another three months.

The obvious candidate on a web log is `URL` and `Referer`, which are both columns of URLs, and they do not overlap. `Referer` holds the page somebody came from and `URL` holds the page they landed on, and on a log of one site those are different sets of strings.

On the eight other corpora the answer is the same or emptier. Seven of them print "no two columns overlap enough to share, so there is nothing to price" on every chunk: yellow taxi, Ookla, and all five TPC-H tables measured. The one exception is the for hire vehicle file, which finds two groups and prices both.

```
| group                                                                                            |   apart | shared table | shared dict | best saves | picked          |
| ------------------------------------------------------------------------------------------------ | ------- | ------------ | ----------- | ---------- | --------------- |
| dispatching_base_num + originating_base_num                                                      | 6.38 MB |     19.16 MB |     6.38 MB |       0.1% | shared dict 161 |
| shared_request_flag + shared_match_flag + access_a_ride_flag + wav_request_flag + wav_match_flag | 2.84 MB |      9.84 MB |     2.82 MB |       0.6% | shared dict 161 |
```

Eleven of its 105 pairs overlap at 0.05 or better, which sounds like eleven opportunities and is two: the one pair inside the first group and the ten pairs inside the second. Counting pairs overstates this mechanism everywhere, because overlap is transitive enough that a group of n columns contributes n choose 2 pairs and one decision.

The second group is five columns whose value sets are identical, a Jaccard of 1.000 across the board, which is as good as this mechanism can ever have it. They are five flag columns holding `Y` and `N`. Union them and you save half of a dictionary that is two strings long, which prices out at 0.6 percent of 2.84 MB, which is seventeen kilobytes. A mechanism that is handed the perfect case and returns seventeen kilobytes does not have a bad case, it has no case.

The synthetic numbers taken while this was being built said a third. A synthetic group where every column draws from one pool of URLs took 433,162 bytes to 295,585 with a shared dictionary. That number was real and it was also useless, because it measured a corpus written to make the mechanism work. That gap between the synthetic result and the real one is the single most useful thing this milestone produced about how to run the rest of the project.

## Q4, recomputation rules

The answer is also no, with one rule that is real and worth eighteen megabytes.

Eight rules were named on `hits` out of the 624 pairs that the sketch reported as a dependency at 0.98 or better, priced over every chunk, with violations counted while the mapping was built.

```
| determines  |      column |    stored | rule on codes | rule with keys | saves |       keys | violations |
| ----------- | ----------- | --------- | ------------- | -------------- | ----- | ---------- | ---------- |
| RefererHash |     Referer |   1.93 GB |       1.79 GB |        1.99 GB |  7.3% | 26,590,624 | 859,624    |
| Referer     | RefererHash | 355.71 MB |     189.65 MB |        1.96 GB | 46.7% | 24,830,152 | 5,940,018  |
| URL         |     URLHash | 381.90 MB |     209.09 MB |        2.49 GB | 45.3% | 27,374,884 | 11,586,966 |
| URLHash     |         URL |   2.44 GB |       2.14 GB |        2.37 GB | 12.3% | 30,106,942 | 3,562,541  |
| ClientIP    | IPNetworkID |  58.09 MB |      39.92 MB |      171.31 MB | 31.3% | 21,194,988 | 0          |
| UserID      |     FUniqID | 172.76 MB |     163.52 MB |      329.54 MB |  5.3% | 21,758,076 | 306,200    |
| URL         | OriginalURL |   1.74 GB |       1.60 GB |        3.89 GB |  8.2% | 27,374,884 | 4,021,299  |
| HID         |  WindowName |  32.71 MB |      29.58 MB |      435.17 MB |  9.6% | 84,728,013 | 38,640     |
```

One of the eight holds. `ClientIP` determines `IPNetworkID` with zero violations in a hundred million rows, which is a real functional dependency and which saves 31.3 percent of 58.09 MB, so eighteen megabytes of a 9.65 GB file.

The other seven have violations and a rule with violations is not a rule. The two that looked strongest on the sketch, `RefererHash` determining `Referer` at 0.993 and `URL` determining `OriginalURL` at 0.995, are wrong on 859,624 and 4,021,299 rows. `URL` determining `URLHash`, which sounds like it must be true by construction, is wrong on 11,586,966 rows.

The `with keys` column is the other half of the answer and it is worse than the first half. Storing the mapping as keys and values costs more than the column costs in all eight cases, and by a factor of thirteen on `HID` to `WindowName`. A recomputation rule is only ever cheaper than the column when it is expressed against the determining column's dictionary codes, which means it needs a dictionary that is stable across the chunk boundary, which is section 6.5 and is the same global dictionary machinery that Q1 just said is not worth building.

On the other corpora the detector found candidates and the candidates are worse than the ones on `hits`, in an instructive way. Twenty seven dependencies at 0.98 or better across 308 pairs, and reading them column by column shows that almost all of them are an artifact of the test rather than a property of the data.

```
| corpus            | pairs | reported | example                                             |
| ----------------- | ----- | -------- | --------------------------------------------------- |
| yellow taxi       |    36 |        1 | tpep_dropoff_datetime determines store_and_fwd_flag  |
| for hire vehicle  |   105 |       12 | pickup_datetime determines wav_request_flag          |
| Ookla             |    36 |        7 | tile determines devices                             |
| TPC-H customer    |    28 |        1 | c_address determines c_acctbal                      |
| TPC-H part        |    36 |        3 | p_name determines p_comment                         |
| TPC-H supplier    |    21 |        3 | s_comment determines s_acctbal                      |
| TPC-H orders      |    36 |        0 | none at 0.98 or better                              |
| TPC-H partsupp    |    10 |        0 | none at 0.98 or better                              |
```

Every example in that table has a near unique column on the left. A microsecond timestamp determines a two value flag because almost no two rows share a timestamp. `tile`, which is a WKT polygon with 6,214,187 distinct values in 6,655,986 rows, determines all seven numeric columns for the same reason. `c_address` is unique per customer, so it determines `c_acctbal` and it would equally determine anything else in the table. These are true statements and they are worth nothing, because the storage cost of a rule keyed on a near unique column is the near unique column.

Exactly one of the twenty six is a non trivial dependency: `p_brand` determines `p_mfgr` on TPC-H part, twenty five brands onto five manufacturers, which is true by construction of `dbgen`. It is worth nothing either, because `p_mfgr` is five distinct values over twenty million rows and a dictionary with RLE on top of it already stores that column in almost no space at all.

That is the finding to carry forward, and it is about the detector and not about the data. The dependence score cannot distinguish a real rule from a left side that is nearly unique, and any future version of this test has to require that the determining column has materially fewer distinct values than the table has rows before it reports anything. Without that filter the test reports 624 candidates on `hits` and 26 on everything else, and a human has to read all of them.

## Q5, memory during dictionary construction

This one is fine, and it is the only question in the milestone whose answer is yes.

Peak resident for the whole file pass over `hits`, streaming 105 columns at 122,880 rows a chunk, building every dictionary in the file and holding a sketch per column, was 1002 MB. On the first million rows it was 441 MB, and the difference is the per column sketches filling up rather than anything growing without bound.

On the eight smaller corpora it never went over 262.63 MB, and that peak was Ookla, whose 6.6 million rows carry a column of polygon strings. The rest sit between 72.39 MB for yellow taxi and 235.15 MB for TPC-H partsupp, with no relationship to the size of the file: TPC-H orders at 150 million rows and 6.05 GB peaked at 161.80 MB, lower than Ookla at 6.6 million rows.

The `groups` pass, which holds three encodings of a group at once, peaked at 813 MB on `hits` and never over 92.14 MB on anything else.

Q5 asked whether a global dictionary can be built during a load without an unacceptable cost, and on these files the memory is not the cost. The encode throughput is.

## Throughput, which is the real problem

Encode runs at 5 MB/s of values per core and decode at 186 MB/s. That is a write path 37 times slower than its read path, and no product ships that.

In row terms, the nine corpora ran between 49,067 and 119,061 rows a second on four cores.

| corpus | rows | wall clock | rows/s | peak resident |
| ------ | ---- | ---------- | ------ | ------------- |
| ClickBench hits | 99,997,497 | 105 min | 15,873 | 1002 MB |
| TPC-H orders | 150,000,000 | 2917.3s | 51,417 | 161.80 MB |
| TPC-H partsupp | 80,000,000 | 1446.9s | 55,290 | 235.15 MB |
| TPC-H customer | 15,000,000 | 305.7s | 49,067 | 190.95 MB |
| TPC-H part | 20,000,000 | 350.5s | 57,061 | 131.48 MB |
| TPC-H supplier | 1,000,000 | 18.3s | 54,645 | 158.81 MB |
| NYC yellow taxi | 2,964,624 | 24.9s | 119,061 | 72.39 MB |
| NYC for hire vehicle | 19,663,930 | 323.1s | 60,860 | 113.50 MB |
| Ookla | 6,655,986 | 100.3s | 66,361 | 262.63 MB |

`hits` is an order of magnitude below the rest per row because it is 105 columns wide and because three of them are long URLs, so a row of `hits` is much more work than a row of `orders`. The whole pass cost 5.6 CPU hours for 89 GB of values, which is 226 CPU seconds a gigabyte.

Where it goes is not a mystery and is not a micro optimisation problem. The chooser encodes every candidate cascade in full and then picks the smallest, which means the cost of choosing is the sum of the costs of every shape it considered rather than the cost of the one it kept. Adding front coding made the ratio better and the throughput worse, 5 MB/s against 7 MB/s before, for exactly that reason: one more candidate to encode in full on every chunk of every string column. Decode went the other way, 186 MB/s against 149 MB/s, because a front coded dictionary is smaller to read and a prefix copy is cheaper than the FSST expansion it replaced.

This is a finding of the milestone rather than an implementation detail of a throwaway lab, because the chooser is the piece that would have been carried into the storage engine unchanged.

## The one regression

Ookla is the only corpus where rudb is larger than Parquet, at 1.61, and it is one column.

`tile` is a well known text polygon string, 196.98 MB in Parquet with Snappy and 445.15 MB in rudb, which is 2.26 times larger. The shape chosen is `FRONT(DICT(FOR+BITPACK[6], FOR+BITPACK[4..5]), FSST[255](FOR+BITPACK[5..6]))`.

It is the same structural fact that `URL` showed, without the front coding rescue. Every polygon is a string of coordinate pairs that shares most of its text with the polygons around it, Snappy's LZ77 window reaches across row boundaries into that shared text, and FSST's per string symbol table cannot. Front coding gets the common prefix and stops, because the shared text here is not all prefix.

The conclusion is not that this column needs a better cascade. It is that a cascade with no general block compressor at the end of it has no floor, and can lose to a format that has one by any margin on data nobody anticipated. Parquet cannot lose to raw by more than its own header, because its last step is always a block compressor. rudb, as built in this experiment, can lose by 2.26x, and did, on the first file we pointed it at that was outside the three shapes we had in mind.

## The decision

The exit criterion said 6 GB or above means the resource claim is wrong. `hits` is 9.65 GB. The second clause applies.

First, the resource claim is amended rather than defended, and this has happened. Documents 00, 02 and 03 in the rudb repository now say 2.1x on disk instead of 10x, with the measurement written out in a new section 2.6.1 of the goal document and a new section 3.5.1 of the baselines document. The paragraphs that made the original prediction were left in place, because a specification that quietly replaces a prediction with its outcome is one nobody can check afterwards. Section 2.7, which lists four things that would make this project not worth doing, now records that the first of the four has happened.

Second, sections 6.4 and 6.5 are demoted. The shared dictionary and the shared symbol table stop being the techniques with no equivalent in DuckDB and become optional mechanisms that are not scheduled for any milestone. They are worth 4 MB out of 9.65 GB on `hits` and 17 KB on the one other corpus where they apply at all, and no amount of implementation quality moves a number that starts there. Document 06's ordering was backwards: the technique nobody else has was worth four kilobytes of saving, and the technique any format could adopt tomorrow was worth two gigabytes.

Third, section 6.6 is demoted with them, and the detector gets a condition. One rule out of eight held on `hits` and one out of twenty six held anywhere else, and both are worth less than a tenth of a percent of their file. Any future version of the dependence test must require the determining column to have materially fewer distinct values than the table has rows, because without that it reports hundreds of true and useless statements about near unique columns.

Fourth, front coding and the sorted dictionary are promoted out of the footnotes. They produced the entire difference between 11.65 GB and 9.65 GB, and they are single column techniques that cost nothing in cross column machinery. The next ratio work is more of that and not more of 6.4.

Fifth, float encoding is now the largest known unmeasured gap and it gets scheduled. Three of the nine corpora have between five and thirty six percent of their bytes in columns this encoder skips, and a storage engine that cannot compress a double is not a storage engine for anything with a measurement in it. Nothing about the disk claim is settled until that number exists.

Sixth, the cascade gets a floor. A general block compressor as the last resort in the chooser, so that the format can never be worse than a block compressed page by more than the cost of trying. The Ookla column is the argument and one column is enough of an argument, because the failure mode is unbounded.

Seventh, and this is the one that actually blocks the storage engine, the chooser has to stop encoding every candidate in full. At 5 MB/s of values per core the write path is 37 times slower than the read path, and that ratio gets worse every time a candidate is added. Sampling to rank candidates and encoding only the winner is the obvious shape, and the sampler is already known to be load bearing: during development, splitting a shared sample evenly across the columns of a group made sharing lose on a pair where allocating in proportion to column size made it win, on the same data with the same code. The sampler decided the outcome twice in this milestone, which means it is a component and not a detail.

## What this milestone was worth

It cost about two weeks of machine time and it prevented three months of storage engine work on top of an assumption that is false.

It also produced the number that the whole project now runs against, which is that rudb's format is 1.51 times smaller than Parquet and 2.12 times smaller than DuckDB on the data we have, and the ten in document 02 is not coming from the format. If a 10x resource claim survives at all it has to come from somewhere else, and saying that out loud now is worth more than any of the individual ratios above.

The measurements, pass by pass and with every number, are in the comments on [issue 2](https://github.com/tamnd/rudb/issues/2). The encoder is `experiments/format-lab` in the rudb repository, and it is meant to be thrown away.
