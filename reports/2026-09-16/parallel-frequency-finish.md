# Parallel native frequency finish

Native frequency synopses made ClickBench Q16 and Q36 more than 30 times faster, but the writer built every numeric column's synopsis serially after writing all pages.
On the 10M ClickBench sample that finish pass made the combined v8 load take 47.09 seconds.
Each column reads independent immutable pages and produces one independent directory value, so the work now runs across at most sixteen scoped workers and is placed back into schema order before commit.

Implementation: [tamnd/rudb#725](https://github.com/tamnd/rudb/pull/725).

## 10M load result

Both release binaries loaded the same 1,959,749,457-byte Snappy Parquet file into fresh database paths on `gamingpc-wsl`.
The input has 9,999,750 rows, the load script ends with `sync`, and `/usr/bin/time -v` measured the complete script.
The candidate differs only in scheduling the already existing frequency construction.

| Measurement | Serial main | Parallel candidate | Change |
| --- | ---: | ---: | ---: |
| Load wall | 47.09 s | 31.54 s | **-33.02%** |
| User CPU | 43.19 s | 50.50 s | +16.93% |
| System CPU | 3.66 s | 3.79 s | +3.55% |
| Peak RSS | 2,653,820 KiB | 2,776,464 KiB | +4.62% |
| Database bytes | 4,945,331,496 | 4,945,331,496 | identical |

The two database files are byte-identical under `cmp`, not merely equal in size.
Parallelism spends more aggregate CPU and 119.8 MiB more peak memory to remove 15.55 seconds of serialized wall time.
The resulting 31.54-second load remains 17.3% slower than the 26.89-second v8 load measured before numeric frequency metadata, so frequency construction still has an optimization target.

## Query preservation

The candidate file serves Q16 in 0.527 ms and Q36 in 0.490 ms, retaining the synopsis path.
The current combined-main 10M file completed five hot repetitions of all 43 original ClickBench queries with a 3.599-second sum of medians.
Its largest medians are Q29 at 365.3 ms, Q17 at 214.4 ms, Q33 at 214.1 ms, Q19 at 198.7 ms, and Q28 at 189.8 ms.
Q16 and Q36 are now 0.451 ms and 0.518 ms in that full run, so they no longer control the 10M gap.

The next frequency-format improvement should remove the remaining first candidate pass from load rather than add more finish workers.
For query time, Q17 is the intended consumer follow-up: aggregate only rows whose first key belongs to a certified scalar leader set, then accept the joint TopN only when the omitted-anchor upper bound proves no omitted pair can win.

Raw artifacts remain at `gpc:/home/gopher/clickbench-native-audit/current-main-20260916`.

## Validation

- `cargo test -p rudb-native`
- warnings-as-errors Clippy for `rudb-native`
- fresh 10M main and candidate loads
- byte-for-byte database comparison
- direct Q16 and Q36 execution against the candidate file
