# Q2 fresh-process RSS below one million rows

The corrected Q2 measurements meet both 10x goals at 10 million rows. At 1k, 10k, and 1m, rudb still peaks at about 5.39 MiB while DuckDB peaks near 40 MiB. This follow-up tests whether a smaller release build can close the small-size RSS gap without losing the 10m time result. Each row below is a 51-pair fresh-process run with the same SQL, the same native data, verified answers, and the C `measure-child` helper. DuckDB is included beside every rudb variant.

| Build and rows | DuckDB wall | rudb wall | Wall ratio | DuckDB RSS | rudb RSS | RSS ratio |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Default, 1k | 22.229 ms | 1.219 ms | 18.24x | 40.07 MiB | 5.39 MiB | 7.44x |
| Panic abort, 1k | 23.105 ms | 1.245 ms | 18.56x | 40.08 MiB | 5.25 MiB | 7.64x |
| Size optimized and panic abort, 1k | 22.088 ms | 1.889 ms | 11.69x | 40.05 MiB | 5.02 MiB | 7.98x |
| Default, 10m | 31.943 ms | 2.046 ms | 15.61x | 64.83 MiB | 5.39 MiB | 12.03x |
| Panic abort, 10m | 32.789 ms | 2.129 ms | 15.40x | 65.00 MiB | 5.25 MiB | 12.38x |
| Size optimized and panic abort, 10m | 31.558 ms | 4.087 ms | 7.72x | 64.59 MiB | 5.01 MiB | 12.89x |

The default build uses the repository release profile. The panic-abort variant sets `CARGO_PROFILE_RELEASE_PANIC=abort`. The size variant also sets `CARGO_PROFILE_RELEASE_OPT_LEVEL=z`. The latter loses the 10x 10m time goal while still missing the 10x 1k RSS goal, so it should not replace the default build. Panic abort saves only about 0.14 MiB and changes the process's failure behavior for little gain.

A held 1k Q2 process had about 5.5 MiB resident. Its largest mapped parts were 1.92 MiB of rudb executable text, 1.28 MiB of libc text, 0.50 MiB of rudb read-only pages, and 0.29 MiB of anonymous allocator pages. The dominant cost is mapped code and runtime, not table metadata. Both experimental static-musl and static-glibc binaries exited with signal 11 before handling `--version`. They need a runtime investigation before they can be benchmarked as alternatives.

The 1k target would require less than about 4.01 MiB rudb peak RSS against this DuckDB build. More native synopsis work will not provide that reduction. The next useful experiment should isolate which executable and C library pages the Q2 path touches, then cut work from that path without changing SQL behavior. The [default measurements](q2-fresh-process-catalog-count.md) and the raw [1k panic-abort](q2-1k-panic-abort.json), [10m panic-abort](q2-10m-panic-abort.json), [1k size](q2-1k-size-release.json), and [10m size](q2-10m-size-release.json) results preserve the comparison.
