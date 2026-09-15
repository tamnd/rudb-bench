# q23 memory scaling

q23 is the 1 million row ClickBench suite's rudb peak at about 282 MiB. It scans `Title`, `URL`, `SearchPhrase`, and `UserID`, applies two text patterns and one string comparison, then groups the surviving rows and counts distinct users.

The exact query was run five times in fresh processes at each worker setting. The table reports median child wall and CPU time and the largest peak RSS.

| Workers | Wall time | CPU time | Peak RSS |
| ---: | ---: | ---: | ---: |
| 1 | 0.382 s | 0.380 s | 62.4 MiB |
| 2 | 0.213 s | 0.414 s | 90.9 MiB |
| 4 | 0.124 s | 0.451 s | 167.3 MiB |
| 8 | 0.073 s | 0.485 s | 279.8 MiB |
| 16 | 0.071 s | 0.487 s | 283.5 MiB |
| 32 | 0.071 s | 0.478 s | 283.1 MiB |

Memory scales with active Parquet readers until eight workers, while wall time stops improving after eight. One worker still needs 62.4 MiB, so lowering parallelism cannot reach the requested memory target and would make q23 more than five times slower.

The reader already reads one encoded page at a time. Each active row group keeps one decoded page for every projected column so it can assemble aligned vectors. q23 must decode three string predicate columns before the filter can discard rows. Eight workers therefore keep several large string pages live together. The distinct aggregate is not the primary source of the 282 MiB peak.

The next architectural step is staged predicate-aware scanning. It should decode the first selective predicate column, refine row ordinals, and read later predicate and payload columns only for surviving rows. A worker cap alone trades speed for memory and is not a fix.
