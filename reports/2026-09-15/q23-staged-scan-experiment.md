# q23 staged scan experiment

The q23 worker sweep showed that concurrent Parquet readers cause the 282 MiB peak. This experiment tested whether the existing ordinal Fetch operator could stage its three string predicates without adding a new execution operator.

The first predicate is highly selective.

| Predicate | Rows kept from 1,000,000 |
| --- | ---: |
| `Title LIKE '%Google%'` | 360 |
| `URL NOT LIKE '%.google.%'` | 999,941 |
| `SearchPhrase <> ''` | 131,316 |
| All three | 61 |

The first implementation scanned `Title` and `file_row_number`, evaluated the title predicate, and fetched the original columns for surviving ordinals before evaluating the remaining predicates. Fetch ran once per streaming input chunk and repeatedly opened overlapping pages.

| Version | q23 median wall | Median CPU | Peak RSS |
| --- | ---: | ---: | ---: |
| Current rudb | 0.074 s | 0.486 s | 286.8 MiB |
| Streaming staged fetch | 1.676 s | 9.022 s | 433.0 MiB |

A second implementation sorted the 360 surviving ordinals before Fetch. This coalesced them into one small run and removed repeated fetches between input chunks.

| Version | q23 median wall | Median CPU | Peak RSS |
| --- | ---: | ---: | ---: |
| Current rudb | 0.074 s | 0.484 s | 283.3 MiB |
| Coalesced staged fetch | 0.346 s | 0.727 s | 282.3 MiB |

Both implementations were rejected. The unchanged peak after scanning only `Title` shows that the dominant live allocation is the decompressed title page held by each concurrent row-group reader. Fetching sparse rows later adds page reads without avoiding that allocation.

The next implementation boundary is below the relational operators. It requires smaller independently compressed pages in the source data, or predicate execution inside the page decoder with an encoded representation that can reject rows without materializing the full string page. An optimizer-only rewrite cannot reach the requested memory target on this Parquet layout.
