# Parquet schema-index measurements at main c3830c12

The pending schema-index change reduces name-resolution work for every Parquet footer. At 10m, Q1 whole-process wall time falls by 4.8% and CPU time by 4.2%. Median RSS does not fall. This is a focused measurement of the generic parser change, not proof of the 10x target or a result for all queries. The change remains unmerged.

Clean main is `c3830c12e583ed72468a7d5519da42a406f5db50`, version 0.8.5. The candidate is `2984cfc8d45c0e793fb1340636f233324e1cbcb4` on that main. DuckDB is released v1.5.5. Both RuDB builds used separate empty Cargo target directories. The archive retains their build logs and executable hashes.

Each sample has 60 rounds across six cases: clean main native and Parquet, candidate native and Parquet, and DuckDB native and Parquet. Every case uses a fresh process. The first execution is kept separately; the tables use medians of the other 59. The full 60 rounds balance positions using reversed pairs and rotations; dropping the first round leaves a small imbalance. The page cache is not flushed. Native files were loaded during the earlier d0ff4ee6 audit and reused unchanged. This focused comparison does not measure load time.

Both engines receive six threads and a 4GB memory limit. RuDB uses its default first engine with stored answers disabled and Parquet mirroring disabled. Ordinary footer row counts remain available. The same SQL, normalization projection, source Parquet, and native files are used across the paired builds. No SQL-specific path or precomputed query answer is added.

Wall time covers startup, binding, execution, output, and exit. User plus system CPU and peak RSS come from wait4 for the exact child. The RSS columns are medians of process peaks, not allocation counts. A load gate of 8 is applied before each child on the shared 32-logical-CPU Hyper-V host. This does not reserve CPUs or stabilize clocks.

## Fresh-process Q1 comparison

| Sample | Format | DuckDB wall (ms) | Main wall (ms) | Candidate wall (ms) | DuckDB CPU (ms) | Main CPU (ms) | Candidate CPU (ms) | DuckDB median RSS (MiB) | Main median RSS (MiB) | Candidate median RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | native | 10.154 | 1.762 | 1.748 | 11.645 | 1.729 | 1.706 | 26.21 | 11.05 | 10.81 |
| 1k | parquet | 10.792 | 1.884 | 1.868 | 12.322 | 1.857 | 1.844 | 25.98 | 11.30 | 11.38 |
| 10k | native | 9.929 | 2.055 | 2.021 | 11.290 | 2.030 | 1.990 | 26.22 | 11.04 | 11.04 |
| 10k | parquet | 10.448 | 1.851 | 1.827 | 11.938 | 1.825 | 1.804 | 25.98 | 11.30 | 11.42 |
| 1m | native | 10.328 | 2.096 | 2.022 | 11.686 | 2.065 | 1.997 | 26.43 | 11.04 | 10.81 |
| 1m | parquet | 11.819 | 2.684 | 2.626 | 13.205 | 3.193 | 3.091 | 26.73 | 11.86 | 12.07 |
| 10m | native | 11.657 | 3.364 | 3.313 | 13.335 | 3.339 | 3.288 | 26.95 | 12.54 | 12.31 |
| 10m | parquet | 257.384 | 113.854 | 108.360 | 256.612 | 116.212 | 111.386 | 150.23 | 118.14 | 118.57 |

The native cases act as a control because the patch changes only Parquet metadata parsing. Small startup differences do not establish a native improvement. All 1,440 children returned the exact expected count: 1,000, 10,000, 999,975, or 9,999,750. Resource readings and reported medians were recomputed from the retained files.

## Instruction profile

Callgrind 3.26.0 records emulated instructions for the complete Parquet child. These counts diagnose work and are separate from native timing and RSS. DuckDB threaded initialization and waits can make emulated counts vary sharply; cross-engine count ratios are not hardware throughput ratios. Valgrind reports a brk segment limit warning for the small Rust runs. All twelve profiled children completed with the expected count.

| Sample | DuckDB emulated instructions | Main emulated instructions | Candidate emulated instructions | Main instruction reduction |
| --- | ---: | ---: | ---: | ---: |
| 1k | 804,453,218 | 7,773,103 | 7,746,710 | 0.34% |
| 10k | 1,507,919,431 | 7,804,916 | 7,773,862 | 0.40% |
| 1m | 309,291,160 | 16,646,412 | 15,943,209 | 4.22% |
| 10m | 5,048,194,754 | 1,365,633,328 | 1,263,516,789 | 7.48% |

In the 10m clean-main profile, Metadata::parse accounts for 27.82% of self instructions, Thrift varint decoding for 11.49%, and read_stats for 6.79%. Several allocation and free functions also account for substantial self work. The schema index removes 7.48% of total emulated instructions, leaving most footer parsing and allocation work intact. These are instruction fractions, not wall-time fractions.

The old lookup searches the schema linearly for every column chunk, costing O(G * C²) name-search work for G groups and C columns. The candidate builds one map, preserves the first duplicate name, and checks ordinal matches only when names are unique. Name resolution becomes expected O(C + G * C). Full metadata is still parsed and allocated. No storage format, statistics policy, or query reduction changes.

All 184 Parquet crate tests passed at this revision. Strict Clippy passed with all targets and warnings denied. The focused count checks do not replace complete engine regression coverage before merging.

## Evidence

The [evidence archive](parquet-schema-index-c3830c12/evidence.tar.gz) contains all 1,440 stdout, stderr and resource records, raw journals, command metadata, twelve Callgrind files and their outputs, profiler script, and clean build and test logs. The [provenance](parquet-schema-index-c3830c12/provenance.json) records revisions, archive hash, capacity ranges, and recomputed summaries. Binaries and datasets are excluded. The whole-suite main rerun is recorded separately.

The [candidate patch](parquet-schema-index-c3830c12/candidate.patch) is the measured diff from c3830c12 to 2984cfc8. Its SHA-256 is `7d06fc85173de08a0963ec6a9c32c52227d4414c557e12a37d2301c9b00f6049`.
