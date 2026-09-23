# Q9 grouped distinct after removing the second pair copy

The corrected Q9 path counts `(RegionID, UserID)` pairs when the SQL runs. Its 10m scan sends
6,184,175 candidate pairs to the radix aggregate, and 5,659,348 remain distinct. The prior
finishing pass copied almost all of those pairs into a second vector for collision checks.
[rudb#1617](https://github.com/tamnd/rudb/pull/1617) keeps a compact reference to the pair
already held in the input run and checks the original record only when hash tags match. It
also carries an eight-byte group key through the counting pass instead of a twelve-byte key
plus hash. No query result is written to the native file.

The same Q9 SQL, rows, native files, and DuckDB files were used for 51 alternating
fresh-process trials per size. The runner checks every output against DuckDB's complete row
set and descending counts. A `wait4` helper measures each child from spawn to exit,
including startup, SQL execution, CSV output, CPU time, and peak RSS. All 612 invocations
passed. The two RuDB builds open the same native file; this change does not alter load time
or file size.

| Rows | Prior RuDB wall ms | New RuDB wall ms | DuckDB wall ms | DuckDB / new time | Prior RuDB CPU ms | New RuDB CPU ms | DuckDB CPU ms | Prior RuDB peak MiB | New RuDB peak MiB | DuckDB peak MiB | DuckDB / new RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 7.109 | 7.235 | 25.178 | 3.5x | 6.741 | 6.889 | 31.430 | 15.41 | 15.47 | 46.30 | 3.0x |
| 10k | 14.003 | 14.130 | 25.678 | 1.8x | 13.557 | 13.752 | 32.110 | 18.41 | 18.22 | 46.82 | 2.6x |
| 1m | 13.942 | 13.447 | 36.212 | 2.7x | 56.492 | 53.344 | 100.498 | 81.28 | 71.29 | 115.32 | 1.6x |
| 10m | 53.722 | 50.431 | 93.513 | 1.9x | 424.348 | 381.418 | 930.113 | 363.78 | 298.54 | 424.43 | 1.4x |

At 10m, the change cuts RuDB wall time by 6.1%, CPU time by 10.1%, and peak RSS by 17.9%.
The 1k and 10k wall medians are 0.126 and 0.127 ms slower. Q9 remains far from the target of
ten times lower wall time and peak RSS than DuckDB. The 5.66 million exact pairs still impose
substantial query-time work and memory under the current row order.

The prior-main build was at `f779cdcb6be64a8871f45ba00fdcf395171e8e64`; the change
merged as `fa775b7b2372a9d5fdd9d33c3f7f61c2a673b863`. The prior-main RuDB binary SHA-256 is
`8ab71d98daefebbff35461484bfc101f8a3a770bbd6f014c5925a99ab232ea2c`; the new
RuDB binary is `6f367fa78a3563d4ac204502ca92391f7f787b84915ea37aeecdd5f21296f3ef`.
DuckDB v2.0.0-dev84237 (`cc7e7bac7f`) is
`bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`.
The [runner](../../scripts/q9-fresh-process.py), [raw trials](q9-input-row-references/),
and [expected CSV files](../2026-09-23/q9-corrected-query-time/) reproduce the comparison.
Q10 and Q11 were also checked against DuckDB's complete grouped results at every size, with
ties at the TopN boundary accepted only when the selected groups are valid.
