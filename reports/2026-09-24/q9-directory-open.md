# Q9 directory opening and stored frequency summaries

Q9 reads row-valued native data, but opening its table also walks every stored column frequency summary. A profile of 40 fresh 10k runs put most sampled cycles in directory window reads, summary decoding, and varint decoding. The query does not request these summaries. The current reader validates and allocates their ordinal arrays on open, then drops them.

I tried validating the summaries in place and keeping their file spans for later decoding. This leaves the SQL and native data unchanged. It does not store a grouped answer. The first scan cut 10k startup cost substantially, but it slowed 10m. Reusing the current directory window and falling back to the existing decoder for lists over one million ordinals did not remove the large-file CPU regression. **No reader change from this experiment was merged.**

Each trial started a new process and checked the complete Q9 output against DuckDB. The runner alternated RuDB baseline, RuDB candidate, and DuckDB order on the same host. Wall time, user plus system CPU time, and peak RSS came from `wait4`. The native files use the row-preserving run projection, version 1. The baseline binary also understands the experimental version 2 format, but reads the same version 1 file as the candidate here. The host was shared and sometimes heavily contended, so paired deltas and win counts matter more than isolated wall medians. OS page cache was not cleared.

| Rows | Trials | RuDB baseline wall | Summary scan wall | DuckDB wall | DuckDB / scan wall | Baseline RSS | Scan RSS | DuckDB RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 11 | 16.161 ms | 13.906 ms | 114.063 ms | 8.20x | 6.125 MiB | 6.000 MiB | 37.863 MiB |
| 10k | 51 | 36.026 ms | 15.341 ms | 105.553 ms | 6.88x | 6.625 MiB | 6.125 MiB | 38.359 MiB |
| 1m | 11 | 86.475 ms | 61.015 ms | 525.251 ms | 8.61x | 10.500 MiB | 9.125 MiB | 68.758 MiB |
| 10m | 51 | 46.439 ms | 48.636 ms | 452.401 ms | 9.30x | 11.000 MiB | 11.125 MiB | 133.648 MiB |

| Rows | Baseline CPU | Summary scan CPU | DuckDB CPU | Paired scan minus baseline wall | Paired scan minus baseline CPU | Scan wall wins |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 10.547 ms | 8.512 ms | 91.681 ms | -1.995 ms | -2.037 ms | 10/11 |
| 10k | 30.076 ms | 12.621 ms | 87.236 ms | -20.840 ms | -17.455 ms | 51/51 |
| 1m | 65.317 ms | 52.070 ms | 452.838 ms | -12.454 ms | -4.314 ms | 8/11 |
| 10m | 67.756 ms | 69.506 ms | 711.921 ms | +3.495 ms | +2.715 ms | 16/51 |

The next revision consumed bytes already held in the directory window. At 10m its paired median was **+7.806 ms wall and +2.887 ms CPU**, with 16/51 wall wins. DuckDB's wall median in that run was 678.672 ms against 75.638 ms baseline and 80.087 ms candidate. A later size-adaptive revision decoded lists above one million ordinals through the old path. Its 10m paired median was -0.600 ms wall but **+3.000 ms CPU**, with only 15/51 CPU wins. Median peak RSS rose from 11.000 to 11.250 MiB. DuckDB's wall median was 429.458 ms against 47.077 ms baseline and 48.763 ms candidate. Those results are not a safe large-file improvement.

The profile and measurements point to a format cost. Without a stored byte length for each frequency synopsis, the reader must walk the ordinal stream to find the next column. Avoiding allocations alone does not remove that linear work. The next format should put a checked span length and entry count before each synopsis. A normal projection could then skip unused summaries at open; requesting one would decode and validate that summary. This is metadata about stored column values, not a precomputed SQL answer. Backward readers would still walk the older format.

Raw trials: [initial scan 1k](q9-directory-open/q9-summary-scan-1k.json), [10k](q9-directory-open/q9-summary-scan-10k.json), [1m](q9-directory-open/q9-summary-scan-1m.json), [10m](q9-directory-open/q9-summary-scan-10m.json), [window reuse 10m](q9-directory-open/q9-summary-scan-v2-10m.json), and [adaptive 10m](q9-directory-open/q9-summary-adaptive-10m.json). The [runner](../../scripts/q9-fresh-process.py) contains the process and output checks. The baseline SHA-256 is `5e39fe203b7ecd5162817975dd246bfa210d0039b1d10b08e014ea68e6832c87`; the first scan is `4da09c42b673c79ebad88963ef0328a79333561c86af4bf79043e8a2a2cd0ca1`, window reuse is `9eea4eae136af24365b53d4f10593ffec6c2adfa08c4a21ecf06e4717815662e`, and adaptive is `2539f9e23131cceff21109bd4ec52f5662f088650f86152656ac4644ce0627ac`. The scan code passed 204 native unit tests, the native crash test, and strict native Clippy after rebasing onto RuDB 0.4.30. The adaptive revision passed its focused test and strict Clippy.

Two other Q9 experiments remain unmerged: a version 2 row-preserving run projection saved file space and build memory, but its 10m wall improvement was too small relative to noise; an unsigned varint arithmetic change regressed 10m. Their raw trials are in the [same directory](q9-directory-open/).
