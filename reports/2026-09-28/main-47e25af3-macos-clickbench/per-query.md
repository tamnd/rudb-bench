# ClickBench measurement audit

All 43 SQL queries are attempted on DuckDB native, rudb native, DuckDB Parquet, and rudb Parquet. 7 hot repetitions are required for a complete row. A failed repetition invalidates that query; successful fragments are never averaged into a result.

First execution is not disk-cold: the page cache is not flushed. Each repetition uses a fresh process. Query seconds are the CLI timer, including result rendering; wall and CPU seconds and peak RSS cover the whole child process. CPU and RSS come from wait4 for that child. RSS is the maximum resident set, not allocated bytes or an incremental memory delta. Both native rows open a loaded single-file database. Both Parquet rows query the same source file with native mirroring disabled; metadata-only paths remain available. These are sample results, not official ClickBench scores.

| Size | Engine | Load wall (s) | Load CPU (s) | Load peak RSS (MiB) | Native bytes |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 0.067811 | 0.068186 | 62.25 | 1060864 |
| 1k | rudb-native | 0.143845 | 0.123861 | 38.22 | 1876635 |
| 10k | duckdb-native | 0.194626 | 0.282874 | 69.39 | 4730880 |
| 10k | rudb-native | 0.573614 | 0.614597 | 73.36 | 7732602 |
| 1m | duckdb-native | 5.968138 | 13.520913 | 980.28 | 454569984 |
| 1m | rudb-native | 3.757951 | 12.807664 | 1090.94 | 246771334 |
| 10m | duckdb-native | 56.258662 | 124.414475 | 2876.78 | 2359832576 |
| 10m | rudb-native | 18.825290 | 88.223290 | 2459.38 | 1469077072 |

| Size | Engine | Complete / 43 | Query median sum (s) | Process wall median sum (s) | CPU median sum (s) | Peak RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 43 | 0.084000 | 1.166738 | 1.159101 | 34.42 |
| 1k | rudb-native | 43 | 0.057230 | 0.364858 | 0.309942 | 13.45 |
| 1k | duckdb-parquet | 43 | 0.115000 | 1.183219 | 1.180317 | 32.92 |
| 1k | rudb-parquet | 43 | 0.065836 | 0.365188 | 0.312500 | 12.47 |
| 10k | duckdb-native | 43 | 0.105000 | 1.127270 | 1.146978 | 37.47 |
| 10k | rudb-native | 43 | 0.066803 | 0.377607 | 0.330246 | 15.72 |
| 10k | duckdb-parquet | 43 | 0.137000 | 1.155978 | 1.186434 | 38.53 |
| 10k | rudb-parquet | 43 | 0.098973 | 0.388588 | 0.333932 | 14.86 |
| 1m | duckdb-native | 43 | 2.039000 | 4.046149 | 7.677718 | 262.84 |
| 1m | rudb-native | 43 | 0.554430 | 1.245983 | 1.739419 | 75.44 |
| 1m | duckdb-parquet | 43 | 2.864000 | 4.827528 | 9.079889 | 389.38 |
| 1m | rudb-parquet | 43 | 2.838481 | 3.638602 | 7.487091 | 334.88 |
| 10m | duckdb-native | 43 | 5.380000 | 7.267671 | 30.168191 | 1146.69 |
| 10m | rudb-native | 43 | 1.156791 | 1.600102 | 5.109861 | 282.98 |
| 10m | duckdb-parquet | 43 | 7.708000 | 9.450556 | 39.685573 | 994.19 |
| 10m | rudb-parquet | 43 | 7.036105 | 7.559609 | 36.092951 | 819.11 |

Time totals cover complete queries only; peak RSS includes every measured attempt, including failures. Compare engines only on the same completed query set. Raw JSONL retains every repetition, exit status, CPU components, I/O, page faults, context switches, command, and output paths.

| Size | Query | Answer check |
| --- | --- | --- |
| 1k | q1 | original match |
| 1k | q2 | original match |
| 1k | q3 | original match |
| 1k | q4 | original match |
| 1k | q5 | original match |
| 1k | q6 | original match |
| 1k | q7 | original match |
| 1k | q8 | original match |
| 1k | q9 | same rows; different order |
| 1k | q10 | same rows; different order |
| 1k | q11 | same rows; different order |
| 1k | q12 | same rows; different order |
| 1k | q13 | original selections differ; deterministic retest match |
| 1k | q14 | original selections differ; deterministic retest match |
| 1k | q15 | original selections differ; deterministic retest match |
| 1k | q16 | original selections differ; deterministic retest match |
| 1k | q17 | original selections differ; deterministic retest match |
| 1k | q18 | original selections differ; deterministic retest match |
| 1k | q19 | original selections differ; deterministic retest match |
| 1k | q20 | original match |
| 1k | q21 | original match |
| 1k | q22 | original match |
| 1k | q23 | original match |
| 1k | q24 | original match |
| 1k | q25 | original match |
| 1k | q26 | original match |
| 1k | q27 | original match |
| 1k | q28 | original match |
| 1k | q29 | original match |
| 1k | q30 | original match |
| 1k | q31 | original selections differ; deterministic retest match |
| 1k | q32 | original selections differ; deterministic retest match |
| 1k | q33 | original selections differ; deterministic retest match |
| 1k | q34 | original selections differ; deterministic retest match |
| 1k | q35 | original selections differ; deterministic retest match |
| 1k | q36 | original selections differ; deterministic retest match |
| 1k | q37 | same rows; different order |
| 1k | q38 | same rows; different order |
| 1k | q39 | original match |
| 1k | q40 | original match |
| 1k | q41 | original match |
| 1k | q42 | original match |
| 1k | q43 | original match |
| 10k | q1 | original match |
| 10k | q2 | original match |
| 10k | q3 | original match |
| 10k | q4 | original match |
| 10k | q5 | original match |
| 10k | q6 | original match |
| 10k | q7 | original match |
| 10k | q8 | same rows; different order |
| 10k | q9 | original match |
| 10k | q10 | original match |
| 10k | q11 | original selections differ; deterministic retest match |
| 10k | q12 | original selections differ; deterministic retest match |
| 10k | q13 | original selections differ; deterministic retest match |
| 10k | q14 | original selections differ; deterministic retest match |
| 10k | q15 | original selections differ; deterministic retest match |
| 10k | q16 | original selections differ; deterministic retest match |
| 10k | q17 | original selections differ; deterministic retest match |
| 10k | q18 | original selections differ; deterministic retest match |
| 10k | q19 | original selections differ; deterministic retest match |
| 10k | q20 | original match |
| 10k | q21 | original match |
| 10k | q22 | original match |
| 10k | q23 | original match |
| 10k | q24 | original match |
| 10k | q25 | original match |
| 10k | q26 | original match |
| 10k | q27 | original match |
| 10k | q28 | original match |
| 10k | q29 | original match |
| 10k | q30 | original match |
| 10k | q31 | original selections differ; deterministic retest match |
| 10k | q32 | original selections differ; deterministic retest match |
| 10k | q33 | original selections differ; deterministic retest match |
| 10k | q34 | same rows; different order |
| 10k | q35 | same rows; different order |
| 10k | q36 | original selections differ; deterministic retest match |
| 10k | q37 | original selections differ; deterministic retest match |
| 10k | q38 | original selections differ; deterministic retest match |
| 10k | q39 | original match |
| 10k | q40 | original match |
| 10k | q41 | original match |
| 10k | q42 | original match |
| 10k | q43 | original match |
| 1m | q1 | original match |
| 1m | q2 | original match |
| 1m | q3 | original match |
| 1m | q4 | original match |
| 1m | q5 | original match |
| 1m | q6 | original match |
| 1m | q7 | original match |
| 1m | q8 | same rows; different order |
| 1m | q9 | original match |
| 1m | q10 | original match |
| 1m | q11 | original match |
| 1m | q12 | original match |
| 1m | q13 | same rows; different order |
| 1m | q14 | original match |
| 1m | q15 | original match |
| 1m | q16 | original selections differ; deterministic retest match |
| 1m | q17 | same rows; different order |
| 1m | q18 | original selections differ; deterministic retest match |
| 1m | q19 | original selections differ; deterministic retest match |
| 1m | q20 | original match |
| 1m | q21 | original match |
| 1m | q22 | same rows; different order |
| 1m | q23 | original selections differ; deterministic retest match |
| 1m | q24 | original match |
| 1m | q25 | original match |
| 1m | q26 | original match |
| 1m | q27 | original match |
| 1m | q28 | original match |
| 1m | q29 | original match |
| 1m | q30 | original match |
| 1m | q31 | same rows; different order |
| 1m | q32 | original selections differ; deterministic retest match |
| 1m | q33 | original selections differ; deterministic retest match |
| 1m | q34 | original match |
| 1m | q35 | original match |
| 1m | q36 | same rows; different order |
| 1m | q37 | same rows; different order |
| 1m | q38 | same rows; different order |
| 1m | q39 | original match |
| 1m | q40 | original selections differ; deterministic retest match |
| 1m | q41 | original selections differ; deterministic retest match |
| 1m | q42 | original match |
| 1m | q43 | original match |
| 10m | q1 | original match |
| 10m | q2 | original match |
| 10m | q3 | original match |
| 10m | q4 | original match |
| 10m | q5 | original match |
| 10m | q6 | original match |
| 10m | q7 | original match |
| 10m | q8 | original match |
| 10m | q9 | original match |
| 10m | q10 | original match |
| 10m | q11 | original match |
| 10m | q12 | original match |
| 10m | q13 | original match |
| 10m | q14 | original match |
| 10m | q15 | original match |
| 10m | q16 | original match |
| 10m | q17 | original match |
| 10m | q18 | original selections differ; deterministic retest match |
| 10m | q19 | same rows; different order |
| 10m | q20 | original match |
| 10m | q21 | original match |
| 10m | q22 | original selections differ; deterministic retest match |
| 10m | q23 | same rows; different order |
| 10m | q24 | original match |
| 10m | q25 | same rows; different order |
| 10m | q26 | original match |
| 10m | q27 | original match |
| 10m | q28 | original match |
| 10m | q29 | original match |
| 10m | q30 | original match |
| 10m | q31 | same rows; different order |
| 10m | q32 | original selections differ; deterministic retest match |
| 10m | q33 | original selections differ; deterministic retest match |
| 10m | q34 | original match |
| 10m | q35 | original match |
| 10m | q36 | original match |
| 10m | q37 | original match |
| 10m | q38 | original match |
| 10m | q39 | original selections differ; deterministic retest match |
| 10m | q40 | original selections differ; deterministic retest match |
| 10m | q41 | original selections differ; deterministic retest match |
| 10m | q42 | original match |
| 10m | q43 | original match |

Answer checks retain original differences. Same rows with different ordering are reported separately; differing row selections are rerun with deterministic tie breakers in a separate untimed diagnostic. Those diagnostic queries are stored alongside the raw outputs and never replace the timed SQL.


| Size | Engine | Query | First query s | Hot median s | Hot IQR s | Hot process wall s | Hot CPU s | Peak RSS MiB |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | q1 | 0.000000 | 0.001000 | 0.001000 | 0.023139 | 0.021874 | 18.77 |
| 1k | duckdb-native | q2 | 0.001000 | 0.001000 | 0.000000 | 0.024456 | 0.022172 | 19.73 |
| 1k | duckdb-native | q3 | 0.001000 | 0.001000 | 0.000000 | 0.024145 | 0.022868 | 20.47 |
| 1k | duckdb-native | q4 | 0.001000 | 0.001000 | 0.001000 | 0.026432 | 0.024765 | 20.50 |
| 1k | duckdb-native | q5 | 0.001000 | 0.001000 | 0.001000 | 0.025318 | 0.025425 | 24.31 |
| 1k | duckdb-native | q6 | 0.001000 | 0.001000 | 0.001000 | 0.025504 | 0.024789 | 24.50 |
| 1k | duckdb-native | q7 | 0.000000 | 0.001000 | 0.000000 | 0.025437 | 0.023659 | 20.05 |
| 1k | duckdb-native | q8 | 0.001000 | 0.001000 | 0.000000 | 0.024003 | 0.023342 | 20.58 |
| 1k | duckdb-native | q9 | 0.002000 | 0.002000 | 0.000000 | 0.026474 | 0.028214 | 32.27 |
| 1k | duckdb-native | q10 | 0.003000 | 0.002000 | 0.000000 | 0.029455 | 0.027723 | 33.33 |
| 1k | duckdb-native | q11 | 0.002000 | 0.003000 | 0.001000 | 0.027451 | 0.030497 | 30.88 |
| 1k | duckdb-native | q12 | 0.002000 | 0.002000 | 0.001000 | 0.026482 | 0.028587 | 33.48 |
| 1k | duckdb-native | q13 | 0.001000 | 0.001000 | 0.001000 | 0.023454 | 0.024127 | 26.22 |
| 1k | duckdb-native | q14 | 0.003000 | 0.002000 | 0.001000 | 0.026369 | 0.028340 | 33.22 |
| 1k | duckdb-native | q15 | 0.002000 | 0.002000 | 0.001000 | 0.027842 | 0.026430 | 26.86 |
| 1k | duckdb-native | q16 | 0.001000 | 0.002000 | 0.000000 | 0.025779 | 0.026401 | 26.44 |
| 1k | duckdb-native | q17 | 0.002000 | 0.002000 | 0.000000 | 0.026593 | 0.027950 | 27.52 |
| 1k | duckdb-native | q18 | 0.003000 | 0.002000 | 0.000000 | 0.028972 | 0.027828 | 26.28 |
| 1k | duckdb-native | q19 | 0.003000 | 0.002000 | 0.001000 | 0.026912 | 0.027568 | 28.45 |
| 1k | duckdb-native | q20 | 0.000000 | 0.001000 | 0.001000 | 0.025276 | 0.023442 | 20.30 |
| 1k | duckdb-native | q21 | 0.001000 | 0.001000 | 0.000000 | 0.024483 | 0.023029 | 20.95 |
| 1k | duckdb-native | q22 | 0.002000 | 0.001000 | 0.001000 | 0.026368 | 0.026082 | 21.92 |
| 1k | duckdb-native | q23 | 0.002000 | 0.002000 | 0.001000 | 0.026285 | 0.026372 | 21.91 |
| 1k | duckdb-native | q24 | 0.007000 | 0.006000 | 0.001000 | 0.030898 | 0.033316 | 34.42 |
| 1k | duckdb-native | q25 | 0.002000 | 0.002000 | 0.000000 | 0.026380 | 0.026175 | 23.33 |
| 1k | duckdb-native | q26 | 0.001000 | 0.001000 | 0.000000 | 0.025887 | 0.024444 | 19.55 |
| 1k | duckdb-native | q27 | 0.002000 | 0.001000 | 0.000000 | 0.026158 | 0.024783 | 21.33 |
| 1k | duckdb-native | q28 | 0.003000 | 0.002000 | 0.001000 | 0.027743 | 0.028319 | 26.44 |
| 1k | duckdb-native | q29 | 0.003000 | 0.003000 | 0.002000 | 0.029880 | 0.030721 | 27.61 |
| 1k | duckdb-native | q30 | 0.007000 | 0.006000 | 0.001000 | 0.030367 | 0.028903 | 30.31 |
| 1k | duckdb-native | q31 | 0.002000 | 0.003000 | 0.001000 | 0.031493 | 0.031064 | 28.12 |
| 1k | duckdb-native | q32 | 0.003000 | 0.002000 | 0.001000 | 0.029084 | 0.029568 | 27.67 |
| 1k | duckdb-native | q33 | 0.003000 | 0.002000 | 0.002000 | 0.032334 | 0.033386 | 28.34 |
| 1k | duckdb-native | q34 | 0.002000 | 0.002000 | 0.002000 | 0.027256 | 0.027617 | 26.25 |
| 1k | duckdb-native | q35 | 0.001000 | 0.002000 | 0.001000 | 0.026421 | 0.026567 | 27.06 |
| 1k | duckdb-native | q36 | 0.002000 | 0.002000 | 0.000000 | 0.028153 | 0.027982 | 27.70 |
| 1k | duckdb-native | q37 | 0.002000 | 0.002000 | 0.001000 | 0.027650 | 0.027815 | 26.52 |
| 1k | duckdb-native | q38 | 0.002000 | 0.002000 | 0.001000 | 0.028483 | 0.027998 | 25.67 |
| 1k | duckdb-native | q39 | 0.001000 | 0.002000 | 0.001000 | 0.026430 | 0.025801 | 21.44 |
| 1k | duckdb-native | q40 | 0.003000 | 0.003000 | 0.000000 | 0.034955 | 0.032293 | 28.30 |
| 1k | duckdb-native | q41 | 0.003000 | 0.002000 | 0.000000 | 0.026735 | 0.026540 | 26.19 |
| 1k | duckdb-native | q42 | 0.002000 | 0.002000 | 0.001000 | 0.026574 | 0.026701 | 25.19 |
| 1k | duckdb-native | q43 | 0.002000 | 0.002000 | 0.000000 | 0.027228 | 0.027624 | 25.34 |
| 1k | rudb-native | q1 | 0.000866 | 0.000762 | 0.000074 | 0.007080 | 0.006017 | 9.06 |
| 1k | rudb-native | q2 | 0.000903 | 0.000926 | 0.000093 | 0.007765 | 0.006605 | 9.36 |
| 1k | rudb-native | q3 | 0.000940 | 0.000904 | 0.000064 | 0.007533 | 0.006296 | 9.58 |
| 1k | rudb-native | q4 | 0.000759 | 0.000901 | 0.000487 | 0.008091 | 0.006876 | 10.27 |
| 1k | rudb-native | q5 | 0.000955 | 0.000820 | 0.000149 | 0.007822 | 0.006603 | 9.67 |
| 1k | rudb-native | q6 | 0.000981 | 0.000801 | 0.000234 | 0.007464 | 0.006274 | 9.70 |
| 1k | rudb-native | q7 | 0.000972 | 0.000907 | 0.000139 | 0.007489 | 0.006509 | 9.88 |
| 1k | rudb-native | q8 | 0.001303 | 0.001227 | 0.000092 | 0.007751 | 0.006545 | 11.06 |
| 1k | rudb-native | q9 | 0.001266 | 0.001272 | 0.000236 | 0.008500 | 0.007134 | 11.14 |
| 1k | rudb-native | q10 | 0.001695 | 0.001715 | 0.000377 | 0.008802 | 0.007357 | 11.20 |
| 1k | rudb-native | q11 | 0.001183 | 0.001354 | 0.000109 | 0.009027 | 0.007607 | 12.22 |
| 1k | rudb-native | q12 | 0.001253 | 0.001324 | 0.000316 | 0.008338 | 0.007141 | 11.28 |
| 1k | rudb-native | q13 | 0.001483 | 0.001474 | 0.000108 | 0.008238 | 0.007093 | 11.59 |
| 1k | rudb-native | q14 | 0.001365 | 0.001334 | 0.000104 | 0.008510 | 0.007158 | 11.12 |
| 1k | rudb-native | q15 | 0.001660 | 0.001597 | 0.000099 | 0.009270 | 0.007724 | 11.66 |
| 1k | rudb-native | q16 | 0.000911 | 0.000967 | 0.000096 | 0.007949 | 0.006766 | 9.72 |
| 1k | rudb-native | q17 | 0.000863 | 0.001027 | 0.000089 | 0.007767 | 0.006546 | 10.20 |
| 1k | rudb-native | q18 | 0.000983 | 0.001013 | 0.000218 | 0.008069 | 0.006739 | 9.81 |
| 1k | rudb-native | q19 | 0.001074 | 0.001088 | 0.000168 | 0.008370 | 0.006875 | 10.69 |
| 1k | rudb-native | q20 | 0.000964 | 0.000840 | 0.000080 | 0.007608 | 0.006385 | 9.53 |
| 1k | rudb-native | q21 | 0.001318 | 0.001181 | 0.000407 | 0.007817 | 0.006645 | 11.28 |
| 1k | rudb-native | q22 | 0.001250 | 0.001316 | 0.000082 | 0.008625 | 0.007178 | 11.25 |
| 1k | rudb-native | q23 | 0.001496 | 0.001472 | 0.000156 | 0.008571 | 0.007020 | 11.30 |
| 1k | rudb-native | q24 | 0.001106 | 0.001162 | 0.000228 | 0.007781 | 0.006848 | 10.66 |
| 1k | rudb-native | q25 | 0.001173 | 0.001172 | 0.000128 | 0.007886 | 0.006704 | 10.17 |
| 1k | rudb-native | q26 | 0.001072 | 0.001257 | 0.000225 | 0.008747 | 0.007480 | 10.56 |
| 1k | rudb-native | q27 | 0.001467 | 0.001164 | 0.000286 | 0.009270 | 0.007324 | 11.05 |
| 1k | rudb-native | q28 | 0.001706 | 0.001461 | 0.000135 | 0.008344 | 0.007207 | 11.42 |
| 1k | rudb-native | q29 | 0.002537 | 0.001727 | 0.000116 | 0.008394 | 0.007234 | 12.30 |
| 1k | rudb-native | q30 | 0.003655 | 0.003630 | 0.000346 | 0.010707 | 0.009572 | 10.89 |
| 1k | rudb-native | q31 | 0.002024 | 0.001667 | 0.000903 | 0.009496 | 0.008108 | 11.92 |
| 1k | rudb-native | q32 | 0.001177 | 0.001426 | 0.000394 | 0.008780 | 0.007366 | 11.06 |
| 1k | rudb-native | q33 | 0.001056 | 0.001143 | 0.000191 | 0.009054 | 0.007654 | 10.86 |
| 1k | rudb-native | q34 | 0.001074 | 0.001333 | 0.000327 | 0.008370 | 0.007342 | 11.25 |
| 1k | rudb-native | q35 | 0.001564 | 0.001622 | 0.000506 | 0.008967 | 0.008025 | 11.92 |
| 1k | rudb-native | q36 | 0.001505 | 0.001403 | 0.000212 | 0.008598 | 0.007346 | 11.23 |
| 1k | rudb-native | q37 | 0.001695 | 0.001823 | 0.000294 | 0.009233 | 0.007899 | 13.45 |
| 1k | rudb-native | q38 | 0.001961 | 0.001938 | 0.000316 | 0.008711 | 0.007616 | 12.27 |
| 1k | rudb-native | q39 | 0.001626 | 0.001235 | 0.000404 | 0.008104 | 0.006793 | 11.70 |
| 1k | rudb-native | q40 | 0.001990 | 0.001664 | 0.000547 | 0.010940 | 0.009552 | 11.88 |
| 1k | rudb-native | q41 | 0.002702 | 0.001454 | 0.000129 | 0.009545 | 0.007883 | 11.92 |
| 1k | rudb-native | q42 | 0.001369 | 0.001329 | 0.000476 | 0.008211 | 0.007045 | 11.25 |
| 1k | rudb-native | q43 | 0.001436 | 0.001397 | 0.000198 | 0.009264 | 0.007851 | 11.27 |
| 1k | duckdb-parquet | q1 | 0.001000 | 0.001000 | 0.000000 | 0.024025 | 0.022219 | 19.09 |
| 1k | duckdb-parquet | q2 | 0.002000 | 0.002000 | 0.001000 | 0.025664 | 0.023398 | 20.61 |
| 1k | duckdb-parquet | q3 | 0.001000 | 0.001000 | 0.001000 | 0.025476 | 0.024209 | 21.16 |
| 1k | duckdb-parquet | q4 | 0.001000 | 0.002000 | 0.001000 | 0.026675 | 0.025198 | 20.06 |
| 1k | duckdb-parquet | q5 | 0.005000 | 0.002000 | 0.001000 | 0.026952 | 0.025156 | 24.66 |
| 1k | duckdb-parquet | q6 | 0.002000 | 0.002000 | 0.001000 | 0.024938 | 0.024404 | 24.34 |
| 1k | duckdb-parquet | q7 | 0.002000 | 0.001000 | 0.001000 | 0.024972 | 0.023408 | 20.34 |
| 1k | duckdb-parquet | q8 | 0.002000 | 0.002000 | 0.001000 | 0.024864 | 0.023918 | 20.41 |
| 1k | duckdb-parquet | q9 | 0.003000 | 0.003000 | 0.001000 | 0.025828 | 0.029097 | 31.42 |
| 1k | duckdb-parquet | q10 | 0.002000 | 0.003000 | 0.001000 | 0.028357 | 0.030249 | 32.92 |
| 1k | duckdb-parquet | q11 | 0.002000 | 0.003000 | 0.000000 | 0.028935 | 0.030210 | 30.27 |
| 1k | duckdb-parquet | q12 | 0.002000 | 0.003000 | 0.001000 | 0.026382 | 0.027961 | 31.11 |
| 1k | duckdb-parquet | q13 | 0.002000 | 0.002000 | 0.000000 | 0.024559 | 0.024455 | 25.48 |
| 1k | duckdb-parquet | q14 | 0.003000 | 0.003000 | 0.000000 | 0.027279 | 0.029726 | 32.55 |
| 1k | duckdb-parquet | q15 | 0.002000 | 0.002000 | 0.001000 | 0.027067 | 0.027268 | 26.59 |
| 1k | duckdb-parquet | q16 | 0.002000 | 0.002000 | 0.000000 | 0.026182 | 0.026563 | 25.70 |
| 1k | duckdb-parquet | q17 | 0.003000 | 0.003000 | 0.001000 | 0.027324 | 0.028975 | 27.39 |
| 1k | duckdb-parquet | q18 | 0.002000 | 0.003000 | 0.001000 | 0.027310 | 0.027145 | 25.77 |
| 1k | duckdb-parquet | q19 | 0.003000 | 0.003000 | 0.001000 | 0.026245 | 0.026830 | 27.25 |
| 1k | duckdb-parquet | q20 | 0.001000 | 0.002000 | 0.001000 | 0.026413 | 0.024575 | 19.64 |
| 1k | duckdb-parquet | q21 | 0.001000 | 0.002000 | 0.001000 | 0.025820 | 0.024297 | 20.30 |
| 1k | duckdb-parquet | q22 | 0.002000 | 0.002000 | 0.000000 | 0.027266 | 0.025503 | 21.08 |
| 1k | duckdb-parquet | q23 | 0.003000 | 0.002000 | 0.001000 | 0.025231 | 0.024525 | 21.94 |
| 1k | duckdb-parquet | q24 | 0.007000 | 0.007000 | 0.001000 | 0.031601 | 0.035959 | 32.86 |
| 1k | duckdb-parquet | q25 | 0.003000 | 0.003000 | 0.000000 | 0.027848 | 0.028155 | 23.03 |
| 1k | duckdb-parquet | q26 | 0.002000 | 0.002000 | 0.000000 | 0.025844 | 0.024897 | 19.41 |
| 1k | duckdb-parquet | q27 | 0.002000 | 0.002000 | 0.001000 | 0.025864 | 0.024409 | 20.73 |
| 1k | duckdb-parquet | q28 | 0.003000 | 0.002000 | 0.001000 | 0.028870 | 0.029221 | 26.09 |
| 1k | duckdb-parquet | q29 | 0.004000 | 0.003000 | 0.002000 | 0.030796 | 0.030980 | 27.48 |
| 1k | duckdb-parquet | q30 | 0.008000 | 0.008000 | 0.002000 | 0.033467 | 0.031465 | 30.20 |
| 1k | duckdb-parquet | q31 | 0.002000 | 0.003000 | 0.001000 | 0.028297 | 0.028346 | 27.20 |
| 1k | duckdb-parquet | q32 | 0.003000 | 0.003000 | 0.001000 | 0.029608 | 0.029825 | 27.30 |
| 1k | duckdb-parquet | q33 | 0.003000 | 0.003000 | 0.001000 | 0.029378 | 0.030228 | 27.25 |
| 1k | duckdb-parquet | q34 | 0.002000 | 0.002000 | 0.001000 | 0.028888 | 0.028302 | 26.41 |
| 1k | duckdb-parquet | q35 | 0.003000 | 0.003000 | 0.001000 | 0.028899 | 0.029012 | 26.95 |
| 1k | duckdb-parquet | q36 | 0.003000 | 0.003000 | 0.001000 | 0.028580 | 0.028980 | 26.73 |
| 1k | duckdb-parquet | q37 | 0.003000 | 0.003000 | 0.001000 | 0.028329 | 0.028212 | 25.23 |
| 1k | duckdb-parquet | q38 | 0.003000 | 0.002000 | 0.001000 | 0.028118 | 0.028460 | 25.44 |
| 1k | duckdb-parquet | q39 | 0.002000 | 0.002000 | 0.001000 | 0.027693 | 0.027413 | 21.00 |
| 1k | duckdb-parquet | q40 | 0.004000 | 0.004000 | 0.003000 | 0.033943 | 0.033518 | 27.62 |
| 1k | duckdb-parquet | q41 | 0.004000 | 0.003000 | 0.001000 | 0.028295 | 0.028345 | 25.62 |
| 1k | duckdb-parquet | q42 | 0.002000 | 0.003000 | 0.001000 | 0.028228 | 0.028432 | 24.23 |
| 1k | duckdb-parquet | q43 | 0.002000 | 0.003000 | 0.001000 | 0.026909 | 0.026869 | 24.92 |
| 1k | rudb-parquet | q1 | 0.001332 | 0.001086 | 0.000127 | 0.007518 | 0.006261 | 10.52 |
| 1k | rudb-parquet | q2 | 0.001318 | 0.001180 | 0.000198 | 0.008281 | 0.006990 | 10.20 |
| 1k | rudb-parquet | q3 | 0.001203 | 0.001209 | 0.000120 | 0.007756 | 0.006571 | 10.30 |
| 1k | rudb-parquet | q4 | 0.001393 | 0.001121 | 0.000090 | 0.007784 | 0.006623 | 10.14 |
| 1k | rudb-parquet | q5 | 0.001907 | 0.001089 | 0.000292 | 0.007902 | 0.006672 | 10.50 |
| 1k | rudb-parquet | q6 | 0.001299 | 0.001285 | 0.000274 | 0.007811 | 0.006655 | 10.66 |
| 1k | rudb-parquet | q7 | 0.000787 | 0.000808 | 0.000111 | 0.007771 | 0.006757 | 10.19 |
| 1k | rudb-parquet | q8 | 0.001224 | 0.001330 | 0.000117 | 0.007744 | 0.006578 | 10.17 |
| 1k | rudb-parquet | q9 | 0.001410 | 0.001240 | 0.000072 | 0.008272 | 0.006891 | 10.59 |
| 1k | rudb-parquet | q10 | 0.001595 | 0.001756 | 0.000146 | 0.008306 | 0.007107 | 10.61 |
| 1k | rudb-parquet | q11 | 0.001161 | 0.001341 | 0.000246 | 0.007980 | 0.006727 | 10.39 |
| 1k | rudb-parquet | q12 | 0.001342 | 0.001358 | 0.000153 | 0.007755 | 0.006651 | 10.27 |
| 1k | rudb-parquet | q13 | 0.001345 | 0.001364 | 0.000268 | 0.007293 | 0.006234 | 10.80 |
| 1k | rudb-parquet | q14 | 0.001453 | 0.001344 | 0.000288 | 0.008132 | 0.006797 | 10.69 |
| 1k | rudb-parquet | q15 | 0.001437 | 0.001514 | 0.000137 | 0.008387 | 0.007122 | 10.55 |
| 1k | rudb-parquet | q16 | 0.001303 | 0.001328 | 0.000607 | 0.007846 | 0.007213 | 10.67 |
| 1k | rudb-parquet | q17 | 0.001500 | 0.001393 | 0.000096 | 0.008098 | 0.006891 | 10.17 |
| 1k | rudb-parquet | q18 | 0.001289 | 0.001388 | 0.000182 | 0.008195 | 0.006880 | 10.72 |
| 1k | rudb-parquet | q19 | 0.001698 | 0.001498 | 0.000133 | 0.008308 | 0.007031 | 10.88 |
| 1k | rudb-parquet | q20 | 0.001105 | 0.000956 | 0.000163 | 0.007731 | 0.006897 | 10.11 |
| 1k | rudb-parquet | q21 | 0.001453 | 0.001516 | 0.000439 | 0.008408 | 0.007135 | 10.53 |
| 1k | rudb-parquet | q22 | 0.001580 | 0.001667 | 0.000130 | 0.008800 | 0.007374 | 11.27 |
| 1k | rudb-parquet | q23 | 0.004452 | 0.002130 | 0.000425 | 0.009815 | 0.008415 | 12.16 |
| 1k | rudb-parquet | q24 | 0.001759 | 0.001539 | 0.000100 | 0.008701 | 0.007298 | 11.02 |
| 1k | rudb-parquet | q25 | 0.001348 | 0.001201 | 0.000096 | 0.007894 | 0.006984 | 9.62 |
| 1k | rudb-parquet | q26 | 0.001243 | 0.001178 | 0.000100 | 0.008037 | 0.006874 | 9.73 |
| 1k | rudb-parquet | q27 | 0.001239 | 0.001196 | 0.000037 | 0.007771 | 0.006642 | 9.42 |
| 1k | rudb-parquet | q28 | 0.001789 | 0.001546 | 0.000446 | 0.008079 | 0.007071 | 11.59 |
| 1k | rudb-parquet | q29 | 0.002520 | 0.001923 | 0.000211 | 0.008762 | 0.007558 | 12.47 |
| 1k | rudb-parquet | q30 | 0.004465 | 0.004093 | 0.000154 | 0.011162 | 0.009870 | 11.25 |
| 1k | rudb-parquet | q31 | 0.001711 | 0.001889 | 0.000702 | 0.009318 | 0.007771 | 12.03 |
| 1k | rudb-parquet | q32 | 0.002052 | 0.001771 | 0.000357 | 0.010067 | 0.008618 | 10.73 |
| 1k | rudb-parquet | q33 | 0.001811 | 0.001989 | 0.000262 | 0.008898 | 0.007595 | 10.84 |
| 1k | rudb-parquet | q34 | 0.001448 | 0.001609 | 0.000412 | 0.008020 | 0.006969 | 10.83 |
| 1k | rudb-parquet | q35 | 0.001502 | 0.001666 | 0.000351 | 0.008584 | 0.007286 | 10.59 |
| 1k | rudb-parquet | q36 | 0.001416 | 0.001509 | 0.000169 | 0.008313 | 0.007137 | 10.34 |
| 1k | rudb-parquet | q37 | 0.001761 | 0.001797 | 0.000375 | 0.009332 | 0.008166 | 10.78 |
| 1k | rudb-parquet | q38 | 0.002218 | 0.001767 | 0.000146 | 0.008870 | 0.007587 | 10.94 |
| 1k | rudb-parquet | q39 | 0.001505 | 0.001594 | 0.000210 | 0.008781 | 0.007470 | 10.28 |
| 1k | rudb-parquet | q40 | 0.002062 | 0.002008 | 0.000303 | 0.010773 | 0.009653 | 11.83 |
| 1k | rudb-parquet | q41 | 0.002138 | 0.001609 | 0.000167 | 0.008761 | 0.007414 | 10.97 |
| 1k | rudb-parquet | q42 | 0.001549 | 0.001545 | 0.000324 | 0.009568 | 0.007988 | 10.41 |
| 1k | rudb-parquet | q43 | 0.001416 | 0.001507 | 0.000347 | 0.009604 | 0.008077 | 11.00 |
| 10k | duckdb-native | q1 | 0.000000 | 0.000000 | 0.001000 | 0.025625 | 0.023650 | 18.92 |
| 10k | duckdb-native | q2 | 0.001000 | 0.001000 | 0.000000 | 0.027592 | 0.024637 | 20.30 |
| 10k | duckdb-native | q3 | 0.001000 | 0.001000 | 0.000000 | 0.027305 | 0.026815 | 21.38 |
| 10k | duckdb-native | q4 | 0.001000 | 0.001000 | 0.001000 | 0.025934 | 0.024517 | 19.36 |
| 10k | duckdb-native | q5 | 0.001000 | 0.002000 | 0.001000 | 0.029940 | 0.027965 | 25.03 |
| 10k | duckdb-native | q6 | 0.002000 | 0.002000 | 0.000000 | 0.028026 | 0.029090 | 25.08 |
| 10k | duckdb-native | q7 | 0.000000 | 0.001000 | 0.000000 | 0.030197 | 0.024839 | 20.81 |
| 10k | duckdb-native | q8 | 0.001000 | 0.001000 | 0.000000 | 0.028802 | 0.026520 | 21.73 |
| 10k | duckdb-native | q9 | 0.004000 | 0.003000 | 0.001000 | 0.027603 | 0.029785 | 31.88 |
| 10k | duckdb-native | q10 | 0.003000 | 0.003000 | 0.001000 | 0.027536 | 0.030807 | 34.39 |
| 10k | duckdb-native | q11 | 0.003000 | 0.003000 | 0.001000 | 0.031953 | 0.032628 | 32.69 |
| 10k | duckdb-native | q12 | 0.003000 | 0.002000 | 0.001000 | 0.028576 | 0.030531 | 33.66 |
| 10k | duckdb-native | q13 | 0.003000 | 0.002000 | 0.000000 | 0.030194 | 0.031215 | 27.22 |
| 10k | duckdb-native | q14 | 0.006000 | 0.003000 | 0.001000 | 0.030097 | 0.033301 | 33.89 |
| 10k | duckdb-native | q15 | 0.002000 | 0.003000 | 0.001000 | 0.031903 | 0.031130 | 27.39 |
| 10k | duckdb-native | q16 | 0.002000 | 0.002000 | 0.000000 | 0.026578 | 0.027517 | 26.47 |
| 10k | duckdb-native | q17 | 0.003000 | 0.003000 | 0.000000 | 0.027688 | 0.029243 | 29.33 |
| 10k | duckdb-native | q18 | 0.002000 | 0.003000 | 0.001000 | 0.027536 | 0.027533 | 27.58 |
| 10k | duckdb-native | q19 | 0.003000 | 0.003000 | 0.000000 | 0.028412 | 0.028764 | 29.69 |
| 10k | duckdb-native | q20 | 0.000000 | 0.001000 | 0.001000 | 0.022045 | 0.020986 | 19.50 |
| 10k | duckdb-native | q21 | 0.002000 | 0.002000 | 0.001000 | 0.022291 | 0.021136 | 20.17 |
| 10k | duckdb-native | q22 | 0.002000 | 0.002000 | 0.001000 | 0.022040 | 0.021944 | 21.34 |
| 10k | duckdb-native | q23 | 0.004000 | 0.003000 | 0.001000 | 0.023837 | 0.025702 | 29.88 |
| 10k | duckdb-native | q24 | 0.006000 | 0.006000 | 0.002000 | 0.029930 | 0.031132 | 37.47 |
| 10k | duckdb-native | q25 | 0.002000 | 0.002000 | 0.002000 | 0.022839 | 0.023207 | 23.98 |
| 10k | duckdb-native | q26 | 0.001000 | 0.001000 | 0.001000 | 0.021331 | 0.020658 | 19.14 |
| 10k | duckdb-native | q27 | 0.002000 | 0.001000 | 0.000000 | 0.023369 | 0.022491 | 21.66 |
| 10k | duckdb-native | q28 | 0.002000 | 0.003000 | 0.001000 | 0.024878 | 0.025842 | 27.08 |
| 10k | duckdb-native | q29 | 0.012000 | 0.009000 | 0.001000 | 0.031823 | 0.037753 | 29.05 |
| 10k | duckdb-native | q30 | 0.006000 | 0.006000 | 0.001000 | 0.027459 | 0.026096 | 30.25 |
| 10k | duckdb-native | q31 | 0.002000 | 0.002000 | 0.001000 | 0.024023 | 0.026194 | 28.19 |
| 10k | duckdb-native | q32 | 0.002000 | 0.002000 | 0.000000 | 0.023803 | 0.023887 | 28.30 |
| 10k | duckdb-native | q33 | 0.002000 | 0.002000 | 0.000000 | 0.023094 | 0.024674 | 29.52 |
| 10k | duckdb-native | q34 | 0.005000 | 0.003000 | 0.001000 | 0.024107 | 0.026801 | 29.50 |
| 10k | duckdb-native | q35 | 0.003000 | 0.003000 | 0.000000 | 0.024762 | 0.027156 | 30.00 |
| 10k | duckdb-native | q36 | 0.003000 | 0.003000 | 0.001000 | 0.025094 | 0.025865 | 27.89 |
| 10k | duckdb-native | q37 | 0.002000 | 0.002000 | 0.000000 | 0.024970 | 0.026503 | 27.50 |
| 10k | duckdb-native | q38 | 0.004000 | 0.002000 | 0.000000 | 0.023358 | 0.024064 | 27.27 |
| 10k | duckdb-native | q39 | 0.003000 | 0.002000 | 0.001000 | 0.023376 | 0.024245 | 26.89 |
| 10k | duckdb-native | q40 | 0.002000 | 0.003000 | 0.000000 | 0.023892 | 0.025327 | 29.80 |
| 10k | duckdb-native | q41 | 0.003000 | 0.002000 | 0.000000 | 0.024559 | 0.024406 | 27.70 |
| 10k | duckdb-native | q42 | 0.001000 | 0.002000 | 0.000000 | 0.023679 | 0.025033 | 26.34 |
| 10k | duckdb-native | q43 | 0.002000 | 0.002000 | 0.000000 | 0.025214 | 0.025389 | 26.95 |
| 10k | rudb-native | q1 | 0.000782 | 0.000818 | 0.000109 | 0.007421 | 0.006335 | 9.17 |
| 10k | rudb-native | q2 | 0.000839 | 0.000876 | 0.000219 | 0.008760 | 0.007485 | 9.73 |
| 10k | rudb-native | q3 | 0.001163 | 0.000925 | 0.000206 | 0.008290 | 0.006787 | 10.59 |
| 10k | rudb-native | q4 | 0.000908 | 0.000781 | 0.000203 | 0.007880 | 0.006444 | 9.42 |
| 10k | rudb-native | q5 | 0.000938 | 0.000793 | 0.000519 | 0.009162 | 0.007724 | 10.36 |
| 10k | rudb-native | q6 | 0.001010 | 0.000895 | 0.000116 | 0.008857 | 0.007698 | 10.38 |
| 10k | rudb-native | q7 | 0.000915 | 0.000947 | 0.000439 | 0.009273 | 0.008077 | 9.89 |
| 10k | rudb-native | q8 | 0.001278 | 0.001349 | 0.000217 | 0.009374 | 0.008339 | 11.48 |
| 10k | rudb-native | q9 | 0.001432 | 0.001494 | 0.000183 | 0.008902 | 0.007587 | 11.77 |
| 10k | rudb-native | q10 | 0.002548 | 0.002150 | 0.000320 | 0.009421 | 0.008198 | 12.27 |
| 10k | rudb-native | q11 | 0.001554 | 0.001396 | 0.000258 | 0.008559 | 0.007494 | 11.42 |
| 10k | rudb-native | q12 | 0.001530 | 0.001598 | 0.000225 | 0.009091 | 0.008360 | 11.41 |
| 10k | rudb-native | q13 | 0.001615 | 0.001922 | 0.000252 | 0.011219 | 0.010005 | 12.44 |
| 10k | rudb-native | q14 | 0.002327 | 0.001919 | 0.000251 | 0.011277 | 0.009790 | 12.88 |
| 10k | rudb-native | q15 | 0.002231 | 0.002513 | 0.000333 | 0.011234 | 0.009604 | 12.80 |
| 10k | rudb-native | q16 | 0.001639 | 0.001545 | 0.000078 | 0.008937 | 0.007544 | 11.52 |
| 10k | rudb-native | q17 | 0.002401 | 0.002095 | 0.000434 | 0.010920 | 0.009931 | 11.95 |
| 10k | rudb-native | q18 | 0.001333 | 0.001379 | 0.000120 | 0.008453 | 0.007269 | 10.72 |
| 10k | rudb-native | q19 | 0.002089 | 0.002030 | 0.000263 | 0.009094 | 0.008201 | 12.53 |
| 10k | rudb-native | q20 | 0.000781 | 0.000773 | 0.000061 | 0.007155 | 0.006110 | 9.78 |
| 10k | rudb-native | q21 | 0.001201 | 0.001233 | 0.000081 | 0.007378 | 0.006474 | 10.98 |
| 10k | rudb-native | q22 | 0.001462 | 0.001473 | 0.000311 | 0.007975 | 0.006838 | 11.25 |
| 10k | rudb-native | q23 | 0.003584 | 0.003644 | 0.000270 | 0.010401 | 0.009508 | 14.91 |
| 10k | rudb-native | q24 | 0.001202 | 0.001322 | 0.000338 | 0.007644 | 0.006529 | 11.52 |
| 10k | rudb-native | q25 | 0.001341 | 0.001379 | 0.000087 | 0.008040 | 0.006683 | 10.44 |
| 10k | rudb-native | q26 | 0.001291 | 0.001337 | 0.000152 | 0.007542 | 0.006537 | 10.11 |
| 10k | rudb-native | q27 | 0.001357 | 0.001638 | 0.001157 | 0.012060 | 0.009408 | 12.30 |
| 10k | rudb-native | q28 | 0.001784 | 0.001753 | 0.000240 | 0.008990 | 0.008119 | 12.44 |
| 10k | rudb-native | q29 | 0.003143 | 0.002983 | 0.000988 | 0.009959 | 0.010164 | 15.72 |
| 10k | rudb-native | q30 | 0.003283 | 0.003279 | 0.000316 | 0.009570 | 0.008407 | 11.66 |
| 10k | rudb-native | q31 | 0.001860 | 0.001834 | 0.000403 | 0.008143 | 0.007380 | 11.70 |
| 10k | rudb-native | q32 | 0.001257 | 0.001300 | 0.000101 | 0.007734 | 0.007025 | 11.11 |
| 10k | rudb-native | q33 | 0.001062 | 0.001009 | 0.000122 | 0.007310 | 0.006353 | 9.95 |
| 10k | rudb-native | q34 | 0.001158 | 0.001210 | 0.000261 | 0.008371 | 0.007110 | 10.53 |
| 10k | rudb-native | q35 | 0.001826 | 0.001811 | 0.000242 | 0.008424 | 0.007905 | 12.62 |
| 10k | rudb-native | q36 | 0.001198 | 0.001070 | 0.000201 | 0.007323 | 0.006329 | 10.20 |
| 10k | rudb-native | q37 | 0.001968 | 0.001728 | 0.000257 | 0.008570 | 0.007516 | 12.92 |
| 10k | rudb-native | q38 | 0.001778 | 0.001835 | 0.000241 | 0.008463 | 0.007583 | 12.41 |
| 10k | rudb-native | q39 | 0.001226 | 0.001285 | 0.000155 | 0.007763 | 0.006935 | 12.06 |
| 10k | rudb-native | q40 | 0.001442 | 0.001393 | 0.000224 | 0.007896 | 0.006835 | 11.70 |
| 10k | rudb-native | q41 | 0.001396 | 0.001341 | 0.000191 | 0.007927 | 0.006862 | 12.38 |
| 10k | rudb-native | q42 | 0.001374 | 0.001340 | 0.000184 | 0.008081 | 0.007019 | 11.94 |
| 10k | rudb-native | q43 | 0.001531 | 0.001409 | 0.000337 | 0.008764 | 0.007745 | 12.67 |
| 10k | duckdb-parquet | q1 | 0.001000 | 0.001000 | 0.000000 | 0.024683 | 0.023609 | 18.92 |
| 10k | duckdb-parquet | q2 | 0.001000 | 0.002000 | 0.001000 | 0.026842 | 0.025844 | 19.91 |
| 10k | duckdb-parquet | q3 | 0.002000 | 0.002000 | 0.002000 | 0.031150 | 0.026469 | 20.83 |
| 10k | duckdb-parquet | q4 | 0.001000 | 0.001000 | 0.001000 | 0.024575 | 0.023374 | 19.22 |
| 10k | duckdb-parquet | q5 | 0.003000 | 0.002000 | 0.001000 | 0.030636 | 0.030089 | 24.69 |
| 10k | duckdb-parquet | q6 | 0.002000 | 0.002000 | 0.001000 | 0.030169 | 0.029612 | 24.56 |
| 10k | duckdb-parquet | q7 | 0.002000 | 0.001000 | 0.000000 | 0.025996 | 0.024441 | 19.61 |
| 10k | duckdb-parquet | q8 | 0.002000 | 0.002000 | 0.000000 | 0.027509 | 0.026391 | 21.62 |
| 10k | duckdb-parquet | q9 | 0.005000 | 0.003000 | 0.001000 | 0.028812 | 0.031442 | 30.78 |
| 10k | duckdb-parquet | q10 | 0.003000 | 0.004000 | 0.001000 | 0.027757 | 0.030528 | 32.78 |
| 10k | duckdb-parquet | q11 | 0.005000 | 0.004000 | 0.001000 | 0.031192 | 0.033537 | 31.41 |
| 10k | duckdb-parquet | q12 | 0.004000 | 0.003000 | 0.000000 | 0.032958 | 0.033815 | 32.19 |
| 10k | duckdb-parquet | q13 | 0.003000 | 0.003000 | 0.001000 | 0.030685 | 0.031692 | 26.05 |
| 10k | duckdb-parquet | q14 | 0.004000 | 0.004000 | 0.001000 | 0.031372 | 0.033311 | 32.94 |
| 10k | duckdb-parquet | q15 | 0.003000 | 0.003000 | 0.001000 | 0.029839 | 0.030075 | 27.23 |
| 10k | duckdb-parquet | q16 | 0.003000 | 0.003000 | 0.001000 | 0.026290 | 0.027276 | 26.36 |
| 10k | duckdb-parquet | q17 | 0.004000 | 0.004000 | 0.002000 | 0.028216 | 0.030170 | 28.28 |
| 10k | duckdb-parquet | q18 | 0.003000 | 0.003000 | 0.000000 | 0.027173 | 0.027022 | 26.73 |
| 10k | duckdb-parquet | q19 | 0.003000 | 0.004000 | 0.001000 | 0.026499 | 0.028061 | 29.30 |
| 10k | duckdb-parquet | q20 | 0.001000 | 0.001000 | 0.000000 | 0.021992 | 0.020898 | 18.78 |
| 10k | duckdb-parquet | q21 | 0.002000 | 0.003000 | 0.001000 | 0.022772 | 0.021697 | 19.89 |
| 10k | duckdb-parquet | q22 | 0.002000 | 0.002000 | 0.001000 | 0.023024 | 0.023117 | 22.05 |
| 10k | duckdb-parquet | q23 | 0.004000 | 0.004000 | 0.000000 | 0.024520 | 0.027231 | 29.59 |
| 10k | duckdb-parquet | q24 | 0.009000 | 0.010000 | 0.003000 | 0.033111 | 0.035382 | 38.53 |
| 10k | duckdb-parquet | q25 | 0.003000 | 0.003000 | 0.001000 | 0.024456 | 0.024999 | 22.58 |
| 10k | duckdb-parquet | q26 | 0.002000 | 0.002000 | 0.001000 | 0.021819 | 0.020997 | 20.73 |
| 10k | duckdb-parquet | q27 | 0.002000 | 0.002000 | 0.000000 | 0.024314 | 0.022853 | 21.25 |
| 10k | duckdb-parquet | q28 | 0.005000 | 0.003000 | 0.000000 | 0.025293 | 0.026465 | 27.22 |
| 10k | duckdb-parquet | q29 | 0.010000 | 0.010000 | 0.001000 | 0.031710 | 0.039848 | 28.64 |
| 10k | duckdb-parquet | q30 | 0.005000 | 0.006000 | 0.003000 | 0.033842 | 0.033708 | 30.67 |
| 10k | duckdb-parquet | q31 | 0.003000 | 0.003000 | 0.000000 | 0.024109 | 0.025501 | 27.52 |
| 10k | duckdb-parquet | q32 | 0.002000 | 0.003000 | 0.001000 | 0.025498 | 0.026823 | 27.75 |
| 10k | duckdb-parquet | q33 | 0.003000 | 0.003000 | 0.001000 | 0.024992 | 0.026057 | 27.83 |
| 10k | duckdb-parquet | q34 | 0.004000 | 0.004000 | 0.001000 | 0.026275 | 0.029761 | 28.97 |
| 10k | duckdb-parquet | q35 | 0.003000 | 0.003000 | 0.001000 | 0.025470 | 0.028574 | 29.83 |
| 10k | duckdb-parquet | q36 | 0.003000 | 0.003000 | 0.000000 | 0.024657 | 0.025229 | 27.14 |
| 10k | duckdb-parquet | q37 | 0.004000 | 0.003000 | 0.001000 | 0.024369 | 0.025167 | 26.62 |
| 10k | duckdb-parquet | q38 | 0.004000 | 0.003000 | 0.001000 | 0.026878 | 0.027554 | 27.61 |
| 10k | duckdb-parquet | q39 | 0.003000 | 0.003000 | 0.001000 | 0.024756 | 0.025893 | 26.53 |
| 10k | duckdb-parquet | q40 | 0.004000 | 0.003000 | 0.001000 | 0.023620 | 0.025750 | 29.70 |
| 10k | duckdb-parquet | q41 | 0.003000 | 0.003000 | 0.000000 | 0.024476 | 0.024605 | 27.09 |
| 10k | duckdb-parquet | q42 | 0.002000 | 0.003000 | 0.001000 | 0.024998 | 0.024754 | 24.64 |
| 10k | duckdb-parquet | q43 | 0.003000 | 0.003000 | 0.001000 | 0.026674 | 0.026813 | 25.77 |
| 10k | rudb-parquet | q1 | 0.001090 | 0.001215 | 0.000222 | 0.007750 | 0.006575 | 9.86 |
| 10k | rudb-parquet | q2 | 0.002025 | 0.001339 | 0.000138 | 0.008063 | 0.006761 | 11.67 |
| 10k | rudb-parquet | q3 | 0.001184 | 0.001324 | 0.000353 | 0.009498 | 0.007729 | 10.59 |
| 10k | rudb-parquet | q4 | 0.001363 | 0.001182 | 0.000073 | 0.007981 | 0.006751 | 10.27 |
| 10k | rudb-parquet | q5 | 0.001700 | 0.001358 | 0.000125 | 0.008727 | 0.007563 | 10.91 |
| 10k | rudb-parquet | q6 | 0.001738 | 0.001986 | 0.000549 | 0.010632 | 0.009090 | 11.89 |
| 10k | rudb-parquet | q7 | 0.000790 | 0.000791 | 0.000100 | 0.007804 | 0.006539 | 9.75 |
| 10k | rudb-parquet | q8 | 0.001313 | 0.001334 | 0.000274 | 0.007803 | 0.006679 | 11.55 |
| 10k | rudb-parquet | q9 | 0.001531 | 0.001745 | 0.000375 | 0.009506 | 0.008017 | 12.53 |
| 10k | rudb-parquet | q10 | 0.002234 | 0.002389 | 0.000422 | 0.009480 | 0.008200 | 11.59 |
| 10k | rudb-parquet | q11 | 0.001919 | 0.001548 | 0.000270 | 0.008497 | 0.007252 | 11.38 |
| 10k | rudb-parquet | q12 | 0.002185 | 0.002233 | 0.000762 | 0.010970 | 0.009174 | 12.28 |
| 10k | rudb-parquet | q13 | 0.001776 | 0.001772 | 0.000391 | 0.009903 | 0.008420 | 11.11 |
| 10k | rudb-parquet | q14 | 0.003281 | 0.002170 | 0.000408 | 0.009860 | 0.008350 | 11.97 |
| 10k | rudb-parquet | q15 | 0.001801 | 0.002019 | 0.000316 | 0.010265 | 0.008909 | 11.19 |
| 10k | rudb-parquet | q16 | 0.001684 | 0.001641 | 0.000243 | 0.009004 | 0.007933 | 11.25 |
| 10k | rudb-parquet | q17 | 0.002749 | 0.002752 | 0.000730 | 0.010139 | 0.008493 | 12.72 |
| 10k | rudb-parquet | q18 | 0.001665 | 0.001814 | 0.000220 | 0.008822 | 0.007458 | 10.77 |
| 10k | rudb-parquet | q19 | 0.003039 | 0.003020 | 0.000498 | 0.010348 | 0.008942 | 12.55 |
| 10k | rudb-parquet | q20 | 0.000890 | 0.000913 | 0.000074 | 0.006782 | 0.005769 | 9.28 |
| 10k | rudb-parquet | q21 | 0.002767 | 0.002861 | 0.000184 | 0.008944 | 0.007672 | 12.56 |
| 10k | rudb-parquet | q22 | 0.003066 | 0.003212 | 0.000434 | 0.009070 | 0.008013 | 13.12 |
| 10k | rudb-parquet | q23 | 0.005505 | 0.005709 | 0.000430 | 0.011613 | 0.010456 | 14.86 |
| 10k | rudb-parquet | q24 | 0.002783 | 0.003054 | 0.000218 | 0.009224 | 0.007984 | 12.55 |
| 10k | rudb-parquet | q25 | 0.001426 | 0.001677 | 0.000444 | 0.007910 | 0.006649 | 11.16 |
| 10k | rudb-parquet | q26 | 0.001854 | 0.001495 | 0.000100 | 0.007453 | 0.006309 | 10.53 |
| 10k | rudb-parquet | q27 | 0.001612 | 0.001580 | 0.000213 | 0.008562 | 0.006836 | 11.17 |
| 10k | rudb-parquet | q28 | 0.003076 | 0.002897 | 0.000037 | 0.009476 | 0.008124 | 14.20 |
| 10k | rudb-parquet | q29 | 0.004270 | 0.004470 | 0.000722 | 0.010777 | 0.009489 | 13.84 |
| 10k | rudb-parquet | q30 | 0.003637 | 0.003797 | 0.000310 | 0.009666 | 0.008655 | 10.95 |
| 10k | rudb-parquet | q31 | 0.002359 | 0.002104 | 0.000202 | 0.008184 | 0.007049 | 11.62 |
| 10k | rudb-parquet | q32 | 0.002067 | 0.002029 | 0.000197 | 0.007919 | 0.006834 | 11.16 |
| 10k | rudb-parquet | q33 | 0.001793 | 0.001736 | 0.000175 | 0.007621 | 0.006816 | 11.16 |
| 10k | rudb-parquet | q34 | 0.003838 | 0.003743 | 0.000209 | 0.009966 | 0.008777 | 13.59 |
| 10k | rudb-parquet | q35 | 0.003340 | 0.003637 | 0.000140 | 0.009601 | 0.008364 | 12.73 |
| 10k | rudb-parquet | q36 | 0.001473 | 0.001562 | 0.000199 | 0.007991 | 0.006783 | 11.28 |
| 10k | rudb-parquet | q37 | 0.004697 | 0.002911 | 0.000211 | 0.009271 | 0.008127 | 13.55 |
| 10k | rudb-parquet | q38 | 0.004917 | 0.004047 | 0.000121 | 0.010269 | 0.009104 | 13.81 |
| 10k | rudb-parquet | q39 | 0.003299 | 0.002870 | 0.000057 | 0.008929 | 0.007782 | 12.62 |
| 10k | rudb-parquet | q40 | 0.004239 | 0.004243 | 0.000281 | 0.010795 | 0.009386 | 14.12 |
| 10k | rudb-parquet | q41 | 0.002053 | 0.001642 | 0.000136 | 0.007603 | 0.006520 | 12.11 |
| 10k | rudb-parquet | q42 | 0.001529 | 0.001574 | 0.000145 | 0.007652 | 0.006510 | 11.33 |
| 10k | rudb-parquet | q43 | 0.002438 | 0.001578 | 0.000887 | 0.008258 | 0.007089 | 12.28 |
| 1m | duckdb-native | q1 | 0.001000 | 0.001000 | 0.001000 | 0.020994 | 0.019911 | 18.78 |
| 1m | duckdb-native | q2 | 0.002000 | 0.001000 | 0.001000 | 0.026341 | 0.026207 | 23.22 |
| 1m | duckdb-native | q3 | 0.002000 | 0.002000 | 0.001000 | 0.026337 | 0.029473 | 26.30 |
| 1m | duckdb-native | q4 | 0.005000 | 0.003000 | 0.002000 | 0.031349 | 0.033982 | 29.88 |
| 1m | duckdb-native | q5 | 0.023000 | 0.018000 | 0.003000 | 0.046904 | 0.098681 | 66.95 |
| 1m | duckdb-native | q6 | 0.016000 | 0.013000 | 0.001000 | 0.039682 | 0.080965 | 53.56 |
| 1m | duckdb-native | q7 | 0.001000 | 0.001000 | 0.001000 | 0.028592 | 0.027462 | 21.14 |
| 1m | duckdb-native | q8 | 0.001000 | 0.002000 | 0.002000 | 0.027259 | 0.026464 | 24.80 |
| 1m | duckdb-native | q9 | 0.022000 | 0.023000 | 0.010000 | 0.050324 | 0.127903 | 77.23 |
| 1m | duckdb-native | q10 | 0.028000 | 0.035000 | 0.007000 | 0.067741 | 0.166442 | 88.09 |
| 1m | duckdb-native | q11 | 0.008000 | 0.007000 | 0.002000 | 0.033159 | 0.048879 | 50.08 |
| 1m | duckdb-native | q12 | 0.008000 | 0.007000 | 0.006000 | 0.038180 | 0.056436 | 52.00 |
| 1m | duckdb-native | q13 | 0.009000 | 0.011000 | 0.002000 | 0.036486 | 0.067341 | 56.92 |
| 1m | duckdb-native | q14 | 0.015000 | 0.022000 | 0.004000 | 0.053233 | 0.107939 | 83.75 |
| 1m | duckdb-native | q15 | 0.013000 | 0.011000 | 0.002000 | 0.033710 | 0.065367 | 61.55 |
| 1m | duckdb-native | q16 | 0.018000 | 0.018000 | 0.003000 | 0.045540 | 0.110534 | 76.80 |
| 1m | duckdb-native | q17 | 0.036000 | 0.039000 | 0.006000 | 0.069896 | 0.207077 | 153.92 |
| 1m | duckdb-native | q18 | 0.031000 | 0.039000 | 0.004000 | 0.074101 | 0.171725 | 138.55 |
| 1m | duckdb-native | q19 | 0.047000 | 0.060000 | 0.036000 | 0.093286 | 0.270361 | 173.23 |
| 1m | duckdb-native | q20 | 0.002000 | 0.002000 | 0.000000 | 0.027298 | 0.030079 | 29.11 |
| 1m | duckdb-native | q21 | 0.083000 | 0.053000 | 0.038000 | 0.109836 | 0.219123 | 90.42 |
| 1m | duckdb-native | q22 | 0.061000 | 0.066000 | 0.028000 | 0.130406 | 0.213698 | 98.45 |
| 1m | duckdb-native | q23 | 0.060000 | 0.062000 | 0.027000 | 0.124911 | 0.230060 | 120.34 |
| 1m | duckdb-native | q24 | 0.102000 | 0.097000 | 0.086000 | 0.209920 | 0.367600 | 229.12 |
| 1m | duckdb-native | q25 | 0.013000 | 0.018000 | 0.009000 | 0.072117 | 0.078623 | 34.98 |
| 1m | duckdb-native | q26 | 0.015000 | 0.013000 | 0.012000 | 0.062370 | 0.078781 | 28.81 |
| 1m | duckdb-native | q27 | 0.014000 | 0.013000 | 0.005000 | 0.066782 | 0.077888 | 32.66 |
| 1m | duckdb-native | q28 | 0.138000 | 0.067000 | 0.050000 | 0.131785 | 0.220786 | 97.52 |
| 1m | duckdb-native | q29 | 1.020000 | 0.646000 | 0.102000 | 0.711509 | 1.888765 | 142.52 |
| 1m | duckdb-native | q30 | 0.017000 | 0.018000 | 0.011000 | 0.083431 | 0.066084 | 33.39 |
| 1m | duckdb-native | q31 | 0.028000 | 0.043000 | 0.010000 | 0.119144 | 0.139769 | 60.70 |
| 1m | duckdb-native | q32 | 0.091000 | 0.041000 | 0.024000 | 0.097409 | 0.146127 | 69.33 |
| 1m | duckdb-native | q33 | 0.117000 | 0.120000 | 0.030000 | 0.179394 | 0.405951 | 127.30 |
| 1m | duckdb-native | q34 | 0.120000 | 0.197000 | 0.100000 | 0.279885 | 0.555129 | 255.73 |
| 1m | duckdb-native | q35 | 0.136000 | 0.157000 | 0.058000 | 0.249296 | 0.584742 | 262.84 |
| 1m | duckdb-native | q36 | 0.042000 | 0.065000 | 0.006000 | 0.119051 | 0.252051 | 74.64 |
| 1m | duckdb-native | q37 | 0.013000 | 0.009000 | 0.006000 | 0.072589 | 0.056933 | 30.55 |
| 1m | duckdb-native | q38 | 0.005000 | 0.006000 | 0.009000 | 0.058564 | 0.053893 | 28.91 |
| 1m | duckdb-native | q39 | 0.009000 | 0.005000 | 0.001000 | 0.055486 | 0.050794 | 30.78 |
| 1m | duckdb-native | q40 | 0.010000 | 0.011000 | 0.002000 | 0.060456 | 0.059961 | 37.66 |
| 1m | duckdb-native | q41 | 0.007000 | 0.005000 | 0.002000 | 0.054850 | 0.052766 | 28.98 |
| 1m | duckdb-native | q42 | 0.004000 | 0.006000 | 0.012000 | 0.066532 | 0.052536 | 28.50 |
| 1m | duckdb-native | q43 | 0.005000 | 0.006000 | 0.002000 | 0.060014 | 0.052430 | 27.56 |
| 1m | rudb-native | q1 | 0.000946 | 0.000702 | 0.000083 | 0.006772 | 0.005690 | 9.47 |
| 1m | rudb-native | q2 | 0.000960 | 0.000890 | 0.000247 | 0.008177 | 0.006737 | 10.30 |
| 1m | rudb-native | q3 | 0.001002 | 0.000890 | 0.000090 | 0.008328 | 0.006918 | 9.72 |
| 1m | rudb-native | q4 | 0.000769 | 0.000806 | 0.000068 | 0.007865 | 0.006390 | 9.91 |
| 1m | rudb-native | q5 | 0.001459 | 0.000786 | 0.000061 | 0.007491 | 0.006388 | 10.98 |
| 1m | rudb-native | q6 | 0.001089 | 0.000943 | 0.000103 | 0.009518 | 0.007924 | 10.66 |
| 1m | rudb-native | q7 | 0.000963 | 0.001033 | 0.000299 | 0.010940 | 0.009518 | 10.62 |
| 1m | rudb-native | q8 | 0.001839 | 0.001843 | 0.000538 | 0.010189 | 0.011179 | 15.42 |
| 1m | rudb-native | q9 | 0.008455 | 0.008389 | 0.001014 | 0.019187 | 0.042926 | 48.30 |
| 1m | rudb-native | q10 | 0.016423 | 0.013023 | 0.002905 | 0.024960 | 0.056833 | 52.78 |
| 1m | rudb-native | q11 | 0.003038 | 0.003600 | 0.001243 | 0.013797 | 0.018023 | 25.34 |
| 1m | rudb-native | q12 | 0.002996 | 0.004675 | 0.001674 | 0.015348 | 0.020225 | 26.81 |
| 1m | rudb-native | q13 | 0.003286 | 0.003298 | 0.001084 | 0.013030 | 0.016042 | 18.03 |
| 1m | rudb-native | q14 | 0.005621 | 0.008466 | 0.002678 | 0.020717 | 0.032294 | 33.72 |
| 1m | rudb-native | q15 | 0.006408 | 0.005559 | 0.000552 | 0.014325 | 0.027337 | 26.75 |
| 1m | rudb-native | q16 | 0.000973 | 0.001103 | 0.000188 | 0.008855 | 0.007396 | 10.73 |
| 1m | rudb-native | q17 | 0.009387 | 0.010201 | 0.000909 | 0.019619 | 0.053605 | 54.20 |
| 1m | rudb-native | q18 | 0.002313 | 0.002593 | 0.000359 | 0.013219 | 0.011496 | 14.50 |
| 1m | rudb-native | q19 | 0.014555 | 0.014380 | 0.003591 | 0.028385 | 0.067142 | 57.81 |
| 1m | rudb-native | q20 | 0.001405 | 0.001392 | 0.000069 | 0.009748 | 0.010152 | 13.39 |
| 1m | rudb-native | q21 | 0.043701 | 0.044611 | 0.021454 | 0.062810 | 0.108764 | 47.12 |
| 1m | rudb-native | q22 | 0.106482 | 0.051219 | 0.012988 | 0.072451 | 0.134365 | 46.55 |
| 1m | rudb-native | q23 | 0.045232 | 0.051029 | 0.026166 | 0.067376 | 0.102080 | 48.52 |
| 1m | rudb-native | q24 | 0.057874 | 0.060283 | 0.012468 | 0.084757 | 0.139800 | 67.06 |
| 1m | rudb-native | q25 | 0.005043 | 0.006669 | 0.002865 | 0.032170 | 0.024074 | 15.75 |
| 1m | rudb-native | q26 | 0.009151 | 0.009858 | 0.005570 | 0.038867 | 0.033957 | 18.08 |
| 1m | rudb-native | q27 | 0.006670 | 0.006090 | 0.005334 | 0.034308 | 0.026180 | 15.95 |
| 1m | rudb-native | q28 | 0.008559 | 0.011253 | 0.002342 | 0.030567 | 0.041444 | 31.61 |
| 1m | rudb-native | q29 | 0.159948 | 0.141133 | 0.058810 | 0.166152 | 0.387476 | 75.44 |
| 1m | rudb-native | q30 | 0.007943 | 0.007178 | 0.001173 | 0.040640 | 0.021897 | 11.81 |
| 1m | rudb-native | q31 | 0.012273 | 0.015403 | 0.007584 | 0.038674 | 0.047605 | 28.03 |
| 1m | rudb-native | q32 | 0.002289 | 0.003316 | 0.000609 | 0.022721 | 0.016820 | 11.69 |
| 1m | rudb-native | q33 | 0.002383 | 0.002097 | 0.000650 | 0.020297 | 0.015939 | 11.02 |
| 1m | rudb-native | q34 | 0.001805 | 0.002268 | 0.004259 | 0.023408 | 0.016680 | 10.88 |
| 1m | rudb-native | q35 | 0.018589 | 0.009861 | 0.001672 | 0.028460 | 0.031439 | 24.78 |
| 1m | rudb-native | q36 | 0.001939 | 0.002516 | 0.000397 | 0.021137 | 0.016343 | 10.62 |
| 1m | rudb-native | q37 | 0.009197 | 0.012204 | 0.015059 | 0.035877 | 0.022706 | 16.31 |
| 1m | rudb-native | q38 | 0.005315 | 0.006431 | 0.001728 | 0.028627 | 0.021433 | 15.69 |
| 1m | rudb-native | q39 | 0.005001 | 0.003999 | 0.001595 | 0.024969 | 0.020653 | 15.97 |
| 1m | rudb-native | q40 | 0.010971 | 0.008487 | 0.000947 | 0.026739 | 0.025251 | 18.50 |
| 1m | rudb-native | q41 | 0.004739 | 0.004120 | 0.000462 | 0.023716 | 0.020114 | 14.58 |
| 1m | rudb-native | q42 | 0.004772 | 0.004014 | 0.000899 | 0.022895 | 0.019313 | 15.39 |
| 1m | rudb-native | q43 | 0.005226 | 0.005821 | 0.002162 | 0.027895 | 0.020871 | 14.69 |
| 1m | duckdb-parquet | q1 | 0.003000 | 0.002000 | 0.000000 | 0.023471 | 0.022402 | 19.33 |
| 1m | duckdb-parquet | q2 | 0.003000 | 0.004000 | 0.001000 | 0.027333 | 0.029699 | 22.75 |
| 1m | duckdb-parquet | q3 | 0.004000 | 0.004000 | 0.002000 | 0.030376 | 0.034488 | 24.52 |
| 1m | duckdb-parquet | q4 | 0.007000 | 0.005000 | 0.000000 | 0.033819 | 0.037470 | 37.55 |
| 1m | duckdb-parquet | q5 | 0.030000 | 0.020000 | 0.002000 | 0.045130 | 0.099613 | 69.86 |
| 1m | duckdb-parquet | q6 | 0.019000 | 0.015000 | 0.003000 | 0.047163 | 0.084420 | 59.34 |
| 1m | duckdb-parquet | q7 | 0.003000 | 0.003000 | 0.000000 | 0.034606 | 0.029327 | 22.06 |
| 1m | duckdb-parquet | q8 | 0.004000 | 0.004000 | 0.001000 | 0.029430 | 0.030858 | 23.52 |
| 1m | duckdb-parquet | q9 | 0.025000 | 0.032000 | 0.012000 | 0.061234 | 0.136673 | 79.02 |
| 1m | duckdb-parquet | q10 | 0.031000 | 0.031000 | 0.010000 | 0.062434 | 0.153608 | 88.19 |
| 1m | duckdb-parquet | q11 | 0.008000 | 0.008000 | 0.001000 | 0.033866 | 0.051271 | 54.70 |
| 1m | duckdb-parquet | q12 | 0.007000 | 0.010000 | 0.006000 | 0.039554 | 0.058013 | 54.39 |
| 1m | duckdb-parquet | q13 | 0.011000 | 0.016000 | 0.003000 | 0.043443 | 0.082621 | 61.64 |
| 1m | duckdb-parquet | q14 | 0.023000 | 0.030000 | 0.007000 | 0.063974 | 0.130701 | 87.30 |
| 1m | duckdb-parquet | q15 | 0.020000 | 0.017000 | 0.003000 | 0.043657 | 0.088218 | 65.56 |
| 1m | duckdb-parquet | q16 | 0.018000 | 0.021000 | 0.004000 | 0.046180 | 0.110478 | 79.66 |
| 1m | duckdb-parquet | q17 | 0.048000 | 0.042000 | 0.003000 | 0.071340 | 0.215229 | 159.12 |
| 1m | duckdb-parquet | q18 | 0.043000 | 0.040000 | 0.015000 | 0.076799 | 0.180731 | 141.41 |
| 1m | duckdb-parquet | q19 | 0.056000 | 0.067000 | 0.021000 | 0.100968 | 0.271540 | 178.42 |
| 1m | duckdb-parquet | q20 | 0.004000 | 0.004000 | 0.001000 | 0.029910 | 0.033863 | 35.91 |
| 1m | duckdb-parquet | q21 | 0.108000 | 0.063000 | 0.011000 | 0.110256 | 0.259258 | 139.56 |
| 1m | duckdb-parquet | q22 | 0.087000 | 0.074000 | 0.015000 | 0.131937 | 0.243029 | 158.66 |
| 1m | duckdb-parquet | q23 | 0.141000 | 0.230000 | 0.169000 | 0.300143 | 0.509943 | 289.44 |
| 1m | duckdb-parquet | q24 | 0.246000 | 0.306000 | 0.258000 | 0.382132 | 0.752620 | 389.38 |
| 1m | duckdb-parquet | q25 | 0.058000 | 0.067000 | 0.017000 | 0.127492 | 0.158193 | 56.42 |
| 1m | duckdb-parquet | q26 | 0.029000 | 0.030000 | 0.009000 | 0.089148 | 0.101754 | 35.75 |
| 1m | duckdb-parquet | q27 | 0.045000 | 0.035000 | 0.012000 | 0.087605 | 0.128988 | 47.03 |
| 1m | duckdb-parquet | q28 | 0.097000 | 0.080000 | 0.068000 | 0.146054 | 0.273898 | 156.31 |
| 1m | duckdb-parquet | q29 | 0.649000 | 0.670000 | 0.374000 | 0.751631 | 1.870920 | 177.06 |
| 1m | duckdb-parquet | q30 | 0.018000 | 0.027000 | 0.016000 | 0.087933 | 0.074322 | 33.47 |
| 1m | duckdb-parquet | q31 | 0.052000 | 0.059000 | 0.018000 | 0.126221 | 0.157927 | 62.98 |
| 1m | duckdb-parquet | q32 | 0.070000 | 0.051000 | 0.033000 | 0.109862 | 0.165335 | 67.25 |
| 1m | duckdb-parquet | q33 | 0.141000 | 0.109000 | 0.066000 | 0.171154 | 0.417721 | 116.62 |
| 1m | duckdb-parquet | q34 | 0.228000 | 0.210000 | 0.039000 | 0.277216 | 0.589612 | 278.31 |
| 1m | duckdb-parquet | q35 | 0.152000 | 0.198000 | 0.112000 | 0.257507 | 0.562033 | 275.45 |
| 1m | duckdb-parquet | q36 | 0.062000 | 0.068000 | 0.018000 | 0.119869 | 0.259004 | 73.80 |
| 1m | duckdb-parquet | q37 | 0.049000 | 0.048000 | 0.018000 | 0.103928 | 0.111321 | 67.98 |
| 1m | duckdb-parquet | q38 | 0.029000 | 0.031000 | 0.007000 | 0.089985 | 0.099254 | 61.02 |
| 1m | duckdb-parquet | q39 | 0.040000 | 0.036000 | 0.014000 | 0.085922 | 0.108107 | 65.22 |
| 1m | duckdb-parquet | q40 | 0.077000 | 0.050000 | 0.007000 | 0.109622 | 0.160962 | 97.02 |
| 1m | duckdb-parquet | q41 | 0.014000 | 0.014000 | 0.009000 | 0.065710 | 0.067431 | 33.89 |
| 1m | duckdb-parquet | q42 | 0.014000 | 0.015000 | 0.009000 | 0.071501 | 0.059898 | 30.62 |
| 1m | duckdb-parquet | q43 | 0.014000 | 0.018000 | 0.004000 | 0.080013 | 0.067136 | 30.33 |
| 1m | rudb-parquet | q1 | 0.002208 | 0.002080 | 0.000211 | 0.008206 | 0.008829 | 12.06 |
| 1m | rudb-parquet | q2 | 0.003465 | 0.003149 | 0.000968 | 0.011089 | 0.014525 | 17.66 |
| 1m | rudb-parquet | q3 | 0.005499 | 0.004011 | 0.000378 | 0.011336 | 0.015993 | 22.58 |
| 1m | rudb-parquet | q4 | 0.004983 | 0.005401 | 0.001305 | 0.015767 | 0.021369 | 34.98 |
| 1m | rudb-parquet | q5 | 0.013365 | 0.010275 | 0.001194 | 0.019163 | 0.049225 | 45.94 |
| 1m | rudb-parquet | q6 | 0.031506 | 0.025151 | 0.003046 | 0.035355 | 0.094658 | 52.22 |
| 1m | rudb-parquet | q7 | 0.001255 | 0.001580 | 0.000954 | 0.012491 | 0.010415 | 11.12 |
| 1m | rudb-parquet | q8 | 0.003552 | 0.003663 | 0.000536 | 0.011601 | 0.015364 | 18.14 |
| 1m | rudb-parquet | q9 | 0.012745 | 0.011983 | 0.005031 | 0.022379 | 0.054344 | 58.94 |
| 1m | rudb-parquet | q10 | 0.015234 | 0.016315 | 0.006938 | 0.027144 | 0.070371 | 68.23 |
| 1m | rudb-parquet | q11 | 0.009935 | 0.008678 | 0.001903 | 0.019191 | 0.037917 | 42.11 |
| 1m | rudb-parquet | q12 | 0.011098 | 0.011034 | 0.005626 | 0.021572 | 0.045509 | 45.27 |
| 1m | rudb-parquet | q13 | 0.016204 | 0.018145 | 0.000796 | 0.026877 | 0.078478 | 50.94 |
| 1m | rudb-parquet | q14 | 0.022769 | 0.031558 | 0.002911 | 0.043809 | 0.112756 | 77.92 |
| 1m | rudb-parquet | q15 | 0.023428 | 0.020879 | 0.005736 | 0.029963 | 0.088043 | 55.98 |
| 1m | rudb-parquet | q16 | 0.007005 | 0.007398 | 0.001974 | 0.015775 | 0.033930 | 50.44 |
| 1m | rudb-parquet | q17 | 0.047805 | 0.054533 | 0.013291 | 0.066393 | 0.222485 | 124.53 |
| 1m | rudb-parquet | q18 | 0.014971 | 0.017861 | 0.001206 | 0.028459 | 0.072580 | 51.44 |
| 1m | rudb-parquet | q19 | 0.067120 | 0.086884 | 0.020759 | 0.105313 | 0.284885 | 157.20 |
| 1m | rudb-parquet | q20 | 0.003700 | 0.003747 | 0.001143 | 0.011479 | 0.016420 | 32.47 |
| 1m | rudb-parquet | q21 | 0.104588 | 0.116921 | 0.074158 | 0.141471 | 0.350616 | 187.39 |
| 1m | rudb-parquet | q22 | 0.216525 | 0.123505 | 0.033714 | 0.152678 | 0.355031 | 172.03 |
| 1m | rudb-parquet | q23 | 0.342448 | 0.262172 | 0.101343 | 0.292975 | 0.728233 | 280.23 |
| 1m | rudb-parquet | q24 | 0.186034 | 0.387376 | 0.207705 | 0.424550 | 1.017217 | 334.88 |
| 1m | rudb-parquet | q25 | 0.027015 | 0.031230 | 0.024702 | 0.051697 | 0.100297 | 46.47 |
| 1m | rudb-parquet | q26 | 0.022258 | 0.032753 | 0.010701 | 0.054481 | 0.076920 | 30.05 |
| 1m | rudb-parquet | q27 | 0.040019 | 0.037966 | 0.019842 | 0.061931 | 0.097326 | 45.55 |
| 1m | rudb-parquet | q28 | 0.093983 | 0.150127 | 0.061988 | 0.183199 | 0.319498 | 168.34 |
| 1m | rudb-parquet | q29 | 0.389987 | 0.262602 | 0.037660 | 0.296093 | 0.675742 | 186.92 |
| 1m | rudb-parquet | q30 | 0.015036 | 0.014980 | 0.013231 | 0.039091 | 0.034450 | 18.44 |
| 1m | rudb-parquet | q31 | 0.041527 | 0.053725 | 0.021193 | 0.076797 | 0.116093 | 52.33 |
| 1m | rudb-parquet | q32 | 0.135679 | 0.055021 | 0.022555 | 0.077075 | 0.120724 | 64.86 |
| 1m | rudb-parquet | q33 | 0.032849 | 0.034283 | 0.010353 | 0.059116 | 0.129974 | 66.39 |
| 1m | rudb-parquet | q34 | 0.254453 | 0.305484 | 0.104339 | 0.345761 | 0.668594 | 269.95 |
| 1m | rudb-parquet | q35 | 0.240537 | 0.258960 | 0.099644 | 0.298059 | 0.666395 | 281.92 |
| 1m | rudb-parquet | q36 | 0.015874 | 0.016578 | 0.005254 | 0.034978 | 0.060296 | 38.77 |
| 1m | rudb-parquet | q37 | 0.054322 | 0.069228 | 0.024642 | 0.090807 | 0.105076 | 63.91 |
| 1m | rudb-parquet | q38 | 0.157260 | 0.080890 | 0.020853 | 0.105049 | 0.135091 | 59.28 |
| 1m | rudb-parquet | q39 | 0.082065 | 0.063198 | 0.014100 | 0.085740 | 0.101625 | 64.09 |
| 1m | rudb-parquet | q40 | 0.102761 | 0.097959 | 0.034198 | 0.125785 | 0.184888 | 104.92 |
| 1m | rudb-parquet | q41 | 0.015644 | 0.015202 | 0.005648 | 0.034574 | 0.033683 | 28.08 |
| 1m | rudb-parquet | q42 | 0.012574 | 0.010476 | 0.002711 | 0.030248 | 0.028385 | 26.81 |
| 1m | rudb-parquet | q43 | 0.012228 | 0.013521 | 0.002322 | 0.033085 | 0.032841 | 23.22 |
| 10m | duckdb-native | q1 | 0.001000 | 0.001000 | 0.001000 | 0.018422 | 0.017844 | 21.08 |
| 10m | duckdb-native | q2 | 0.011000 | 0.004000 | 0.001000 | 0.022874 | 0.027404 | 27.69 |
| 10m | duckdb-native | q3 | 0.017000 | 0.010000 | 0.001000 | 0.030632 | 0.066296 | 35.95 |
| 10m | duckdb-native | q4 | 0.017000 | 0.010000 | 0.001000 | 0.030333 | 0.064746 | 41.66 |
| 10m | duckdb-native | q5 | 0.044000 | 0.045000 | 0.005000 | 0.070264 | 0.263902 | 106.70 |
| 10m | duckdb-native | q6 | 0.095000 | 0.088000 | 0.003000 | 0.118130 | 0.513772 | 246.77 |
| 10m | duckdb-native | q7 | 0.003000 | 0.003000 | 0.001000 | 0.021437 | 0.020884 | 22.42 |
| 10m | duckdb-native | q8 | 0.005000 | 0.005000 | 0.001000 | 0.023774 | 0.032561 | 29.52 |
| 10m | duckdb-native | q9 | 0.062000 | 0.058000 | 0.004000 | 0.084080 | 0.345826 | 127.22 |
| 10m | duckdb-native | q10 | 0.107000 | 0.107000 | 0.006000 | 0.137031 | 0.632891 | 147.67 |
| 10m | duckdb-native | q11 | 0.023000 | 0.020000 | 0.002000 | 0.042941 | 0.116834 | 72.41 |
| 10m | duckdb-native | q12 | 0.022000 | 0.022000 | 0.002000 | 0.044549 | 0.127440 | 78.53 |
| 10m | duckdb-native | q13 | 0.070000 | 0.077000 | 0.009000 | 0.106652 | 0.438004 | 243.31 |
| 10m | duckdb-native | q14 | 0.138000 | 0.143000 | 0.022000 | 0.183588 | 0.755300 | 351.19 |
| 10m | duckdb-native | q15 | 0.092000 | 0.082000 | 0.013000 | 0.113141 | 0.465809 | 262.86 |
| 10m | duckdb-native | q16 | 0.059000 | 0.055000 | 0.004000 | 0.080649 | 0.321418 | 124.41 |
| 10m | duckdb-native | q17 | 0.135000 | 0.141000 | 0.015000 | 0.178425 | 0.792515 | 334.44 |
| 10m | duckdb-native | q18 | 0.119000 | 0.115000 | 0.016000 | 0.146075 | 0.624793 | 302.56 |
| 10m | duckdb-native | q19 | 0.297000 | 0.261000 | 0.010000 | 0.327045 | 1.511740 | 702.00 |
| 10m | duckdb-native | q20 | 0.004000 | 0.004000 | 0.001000 | 0.024477 | 0.029063 | 33.81 |
| 10m | duckdb-native | q21 | 0.236000 | 0.137000 | 0.014000 | 0.193618 | 0.812192 | 446.56 |
| 10m | duckdb-native | q22 | 0.112000 | 0.127000 | 0.020000 | 0.214681 | 0.677723 | 497.30 |
| 10m | duckdb-native | q23 | 0.427000 | 0.191000 | 0.041000 | 0.324422 | 1.147597 | 859.56 |
| 10m | duckdb-native | q24 | 0.157000 | 0.135000 | 0.066000 | 0.204171 | 0.693669 | 513.12 |
| 10m | duckdb-native | q25 | 0.011000 | 0.010000 | 0.001000 | 0.034979 | 0.058233 | 40.73 |
| 10m | duckdb-native | q26 | 0.040000 | 0.038000 | 0.002000 | 0.063615 | 0.220916 | 75.67 |
| 10m | duckdb-native | q27 | 0.010000 | 0.009000 | 0.002000 | 0.033604 | 0.057795 | 39.62 |
| 10m | duckdb-native | q28 | 0.150000 | 0.101000 | 0.047000 | 0.191640 | 0.597699 | 457.16 |
| 10m | duckdb-native | q29 | 2.025000 | 1.894000 | 0.146000 | 1.975570 | 10.531611 | 682.61 |
| 10m | duckdb-native | q30 | 0.016000 | 0.015000 | 0.000000 | 0.039846 | 0.071275 | 41.17 |
| 10m | duckdb-native | q31 | 0.078000 | 0.082000 | 0.018000 | 0.120170 | 0.453691 | 193.31 |
| 10m | duckdb-native | q32 | 0.121000 | 0.101000 | 0.008000 | 0.151339 | 0.553165 | 319.30 |
| 10m | duckdb-native | q33 | 0.375000 | 0.338000 | 0.026000 | 0.440058 | 1.902392 | 832.67 |
| 10m | duckdb-native | q34 | 0.373000 | 0.365000 | 0.036000 | 0.508912 | 1.964264 | 1134.09 |
| 10m | duckdb-native | q35 | 0.397000 | 0.389000 | 0.036000 | 0.572112 | 2.137983 | 1146.69 |
| 10m | duckdb-native | q36 | 0.081000 | 0.078000 | 0.003000 | 0.106167 | 0.448944 | 116.92 |
| 10m | duckdb-native | q37 | 0.031000 | 0.029000 | 0.004000 | 0.058567 | 0.152061 | 132.31 |
| 10m | duckdb-native | q38 | 0.008000 | 0.007000 | 0.002000 | 0.029404 | 0.048462 | 47.83 |
| 10m | duckdb-native | q39 | 0.010000 | 0.009000 | 0.001000 | 0.030348 | 0.056439 | 69.95 |
| 10m | duckdb-native | q40 | 0.046000 | 0.055000 | 0.006000 | 0.085845 | 0.283316 | 237.88 |
| 10m | duckdb-native | q41 | 0.009000 | 0.006000 | 0.001000 | 0.028388 | 0.043859 | 46.64 |
| 10m | duckdb-native | q42 | 0.007000 | 0.006000 | 0.001000 | 0.028034 | 0.040681 | 40.55 |
| 10m | duckdb-native | q43 | 0.007000 | 0.007000 | 0.001000 | 0.027662 | 0.045182 | 37.19 |
| 10m | rudb-native | q1 | 0.001473 | 0.000614 | 0.000073 | 0.006888 | 0.006028 | 11.55 |
| 10m | rudb-native | q2 | 0.003587 | 0.000756 | 0.000131 | 0.007767 | 0.006925 | 11.42 |
| 10m | rudb-native | q3 | 0.001044 | 0.000792 | 0.000089 | 0.008216 | 0.007298 | 11.47 |
| 10m | rudb-native | q4 | 0.000745 | 0.000777 | 0.000131 | 0.008514 | 0.007390 | 11.31 |
| 10m | rudb-native | q5 | 0.000760 | 0.000766 | 0.000046 | 0.008688 | 0.007600 | 11.17 |
| 10m | rudb-native | q6 | 0.000821 | 0.000824 | 0.000123 | 0.009331 | 0.008073 | 11.55 |
| 10m | rudb-native | q7 | 0.000916 | 0.000772 | 0.000079 | 0.007675 | 0.006763 | 10.97 |
| 10m | rudb-native | q8 | 0.003997 | 0.002782 | 0.000299 | 0.010843 | 0.017678 | 27.55 |
| 10m | rudb-native | q9 | 0.019909 | 0.021333 | 0.000928 | 0.031470 | 0.118754 | 73.05 |
| 10m | rudb-native | q10 | 0.031123 | 0.031961 | 0.002437 | 0.043930 | 0.173526 | 82.73 |
| 10m | rudb-native | q11 | 0.007866 | 0.007174 | 0.000492 | 0.016285 | 0.042134 | 37.06 |
| 10m | rudb-native | q12 | 0.008453 | 0.008259 | 0.000791 | 0.017682 | 0.048866 | 38.55 |
| 10m | rudb-native | q13 | 0.008566 | 0.009178 | 0.000961 | 0.019064 | 0.048183 | 37.83 |
| 10m | rudb-native | q14 | 0.026070 | 0.028495 | 0.002760 | 0.039890 | 0.148398 | 84.09 |
| 10m | rudb-native | q15 | 0.022572 | 0.020764 | 0.001589 | 0.032143 | 0.111476 | 82.05 |
| 10m | rudb-native | q16 | 0.000993 | 0.001084 | 0.000113 | 0.009775 | 0.008537 | 11.89 |
| 10m | rudb-native | q17 | 0.032658 | 0.036440 | 0.001835 | 0.048447 | 0.205060 | 125.66 |
| 10m | rudb-native | q18 | 0.004056 | 0.004118 | 0.000150 | 0.012860 | 0.011637 | 19.67 |
| 10m | rudb-native | q19 | 0.069345 | 0.071542 | 0.004479 | 0.086595 | 0.404707 | 224.94 |
| 10m | rudb-native | q20 | 0.001614 | 0.001136 | 0.000124 | 0.009067 | 0.009566 | 14.92 |
| 10m | rudb-native | q21 | 0.049371 | 0.058951 | 0.004621 | 0.071365 | 0.224328 | 82.80 |
| 10m | rudb-native | q22 | 0.076272 | 0.071571 | 0.009589 | 0.084559 | 0.239868 | 100.69 |
| 10m | rudb-native | q23 | 0.183340 | 0.183467 | 0.012989 | 0.199037 | 0.465121 | 189.16 |
| 10m | rudb-native | q24 | 0.063728 | 0.065859 | 0.010619 | 0.079046 | 0.209629 | 111.88 |
| 10m | rudb-native | q25 | 0.004840 | 0.003853 | 0.000316 | 0.013899 | 0.016219 | 21.30 |
| 10m | rudb-native | q26 | 0.013125 | 0.012427 | 0.000922 | 0.023196 | 0.052929 | 34.80 |
| 10m | rudb-native | q27 | 0.006946 | 0.006800 | 0.000476 | 0.015881 | 0.018416 | 23.94 |
| 10m | rudb-native | q28 | 0.023085 | 0.024942 | 0.003262 | 0.036761 | 0.100909 | 67.25 |
| 10m | rudb-native | q29 | 0.388201 | 0.340723 | 0.046973 | 0.358613 | 1.749600 | 282.98 |
| 10m | rudb-native | q30 | 0.003778 | 0.003561 | 0.000154 | 0.012107 | 0.010937 | 13.36 |
| 10m | rudb-native | q31 | 0.041722 | 0.033050 | 0.004024 | 0.046018 | 0.177490 | 98.02 |
| 10m | rudb-native | q32 | 0.002431 | 0.002080 | 0.000533 | 0.011803 | 0.010354 | 14.58 |
| 10m | rudb-native | q33 | 0.001259 | 0.001373 | 0.000274 | 0.013154 | 0.011314 | 12.89 |
| 10m | rudb-native | q34 | 0.001308 | 0.001288 | 0.000236 | 0.012836 | 0.011126 | 13.84 |
| 10m | rudb-native | q35 | 0.041389 | 0.026838 | 0.004242 | 0.040135 | 0.118796 | 96.06 |
| 10m | rudb-native | q36 | 0.002063 | 0.001246 | 0.000097 | 0.010400 | 0.009204 | 12.47 |
| 10m | rudb-native | q37 | 0.010619 | 0.008115 | 0.001277 | 0.019620 | 0.030399 | 41.64 |
| 10m | rudb-native | q38 | 0.008330 | 0.006042 | 0.000923 | 0.015308 | 0.024842 | 35.88 |
| 10m | rudb-native | q39 | 0.006155 | 0.005234 | 0.000286 | 0.013479 | 0.016198 | 28.30 |
| 10m | rudb-native | q40 | 0.029300 | 0.033494 | 0.002476 | 0.044929 | 0.144800 | 91.16 |
| 10m | rudb-native | q41 | 0.014758 | 0.006532 | 0.000497 | 0.015138 | 0.025475 | 40.59 |
| 10m | rudb-native | q42 | 0.006419 | 0.005545 | 0.000618 | 0.014804 | 0.021901 | 31.20 |
| 10m | rudb-native | q43 | 0.004262 | 0.004234 | 0.000845 | 0.012884 | 0.021407 | 25.02 |
| 10m | duckdb-parquet | q1 | 0.015000 | 0.015000 | 0.001000 | 0.040402 | 0.039612 | 31.80 |
| 10m | duckdb-parquet | q2 | 0.026000 | 0.021000 | 0.001000 | 0.050276 | 0.074223 | 37.03 |
| 10m | duckdb-parquet | q3 | 0.026000 | 0.026000 | 0.002000 | 0.056505 | 0.099620 | 39.80 |
| 10m | duckdb-parquet | q4 | 0.024000 | 0.025000 | 0.003000 | 0.054421 | 0.093058 | 51.89 |
| 10m | duckdb-parquet | q5 | 0.047000 | 0.050000 | 0.004000 | 0.085380 | 0.236349 | 112.22 |
| 10m | duckdb-parquet | q6 | 0.113000 | 0.116000 | 0.008000 | 0.155453 | 0.579946 | 252.81 |
| 10m | duckdb-parquet | q7 | 0.021000 | 0.017000 | 0.001000 | 0.046985 | 0.046539 | 34.70 |
| 10m | duckdb-parquet | q8 | 0.022000 | 0.022000 | 0.002000 | 0.050979 | 0.076488 | 40.14 |
| 10m | duckdb-parquet | q9 | 0.073000 | 0.076000 | 0.003000 | 0.112689 | 0.379889 | 126.41 |
| 10m | duckdb-parquet | q10 | 0.108000 | 0.112000 | 0.009000 | 0.149044 | 0.588625 | 139.14 |
| 10m | duckdb-parquet | q11 | 0.040000 | 0.034000 | 0.001000 | 0.069899 | 0.145964 | 73.61 |
| 10m | duckdb-parquet | q12 | 0.038000 | 0.038000 | 0.001000 | 0.071547 | 0.165110 | 74.56 |
| 10m | duckdb-parquet | q13 | 0.122000 | 0.126000 | 0.002000 | 0.168340 | 0.611650 | 284.66 |
| 10m | duckdb-parquet | q14 | 0.182000 | 0.187000 | 0.022000 | 0.233339 | 0.935504 | 338.06 |
| 10m | duckdb-parquet | q15 | 0.141000 | 0.128000 | 0.006000 | 0.168014 | 0.645112 | 266.52 |
| 10m | duckdb-parquet | q16 | 0.063000 | 0.063000 | 0.005000 | 0.101019 | 0.302289 | 131.27 |
| 10m | duckdb-parquet | q17 | 0.180000 | 0.183000 | 0.009000 | 0.226813 | 0.947454 | 335.08 |
| 10m | duckdb-parquet | q18 | 0.157000 | 0.161000 | 0.015000 | 0.202128 | 0.783378 | 313.62 |
| 10m | duckdb-parquet | q19 | 0.320000 | 0.320000 | 0.014000 | 0.373346 | 1.718404 | 658.03 |
| 10m | duckdb-parquet | q20 | 0.022000 | 0.019000 | 0.002000 | 0.049477 | 0.058237 | 45.23 |
| 10m | duckdb-parquet | q21 | 0.205000 | 0.221000 | 0.005000 | 0.265537 | 1.204911 | 320.06 |
| 10m | duckdb-parquet | q22 | 0.187000 | 0.194000 | 0.013000 | 0.240679 | 1.056238 | 390.38 |
| 10m | duckdb-parquet | q23 | 0.364000 | 0.413000 | 0.020000 | 0.460863 | 2.210616 | 635.25 |
| 10m | duckdb-parquet | q24 | 0.264000 | 0.307000 | 0.049000 | 0.360877 | 1.473373 | 465.47 |
| 10m | duckdb-parquet | q25 | 0.099000 | 0.083000 | 0.003000 | 0.120603 | 0.317569 | 111.23 |
| 10m | duckdb-parquet | q26 | 0.082000 | 0.086000 | 0.007000 | 0.123998 | 0.402489 | 98.42 |
| 10m | duckdb-parquet | q27 | 0.069000 | 0.066000 | 0.005000 | 0.102777 | 0.297338 | 107.75 |
| 10m | duckdb-parquet | q28 | 0.192000 | 0.208000 | 0.036000 | 0.262199 | 1.053297 | 405.16 |
| 10m | duckdb-parquet | q29 | 2.523000 | 2.375000 | 0.146000 | 2.430134 | 13.240701 | 620.28 |
| 10m | duckdb-parquet | q30 | 0.032000 | 0.032000 | 0.001000 | 0.066956 | 0.100734 | 47.84 |
| 10m | duckdb-parquet | q31 | 0.133000 | 0.138000 | 0.027000 | 0.181647 | 0.673866 | 172.92 |
| 10m | duckdb-parquet | q32 | 0.162000 | 0.154000 | 0.018000 | 0.203864 | 0.750640 | 224.48 |
| 10m | duckdb-parquet | q33 | 0.426000 | 0.373000 | 0.037000 | 0.437524 | 1.895777 | 750.66 |
| 10m | duckdb-parquet | q34 | 0.476000 | 0.460000 | 0.057000 | 0.521579 | 2.388134 | 961.36 |
| 10m | duckdb-parquet | q35 | 0.479000 | 0.487000 | 0.046000 | 0.549310 | 2.526582 | 994.19 |
| 10m | duckdb-parquet | q36 | 0.099000 | 0.100000 | 0.003000 | 0.143270 | 0.497468 | 117.48 |
| 10m | duckdb-parquet | q37 | 0.059000 | 0.057000 | 0.020000 | 0.096657 | 0.238323 | 208.00 |
| 10m | duckdb-parquet | q38 | 0.030000 | 0.026000 | 0.003000 | 0.058346 | 0.089829 | 58.52 |
| 10m | duckdb-parquet | q39 | 0.036000 | 0.031000 | 0.003000 | 0.060819 | 0.111441 | 136.73 |
| 10m | duckdb-parquet | q40 | 0.073000 | 0.083000 | 0.007000 | 0.125131 | 0.385093 | 317.98 |
| 10m | duckdb-parquet | q41 | 0.031000 | 0.025000 | 0.003000 | 0.056703 | 0.079389 | 59.45 |
| 10m | duckdb-parquet | q42 | 0.025000 | 0.025000 | 0.002000 | 0.058700 | 0.079740 | 54.73 |
| 10m | duckdb-parquet | q43 | 0.023000 | 0.025000 | 0.004000 | 0.056327 | 0.084574 | 55.11 |
| 10m | rudb-parquet | q1 | 0.011099 | 0.009293 | 0.000845 | 0.014694 | 0.016476 | 22.36 |
| 10m | rudb-parquet | q2 | 0.017109 | 0.016124 | 0.000421 | 0.022023 | 0.053453 | 29.19 |
| 10m | rudb-parquet | q3 | 0.017782 | 0.017274 | 0.000853 | 0.023153 | 0.058509 | 35.38 |
| 10m | rudb-parquet | q4 | 0.017296 | 0.018591 | 0.000664 | 0.024952 | 0.063287 | 43.38 |
| 10m | rudb-parquet | q5 | 0.054891 | 0.056226 | 0.004364 | 0.064221 | 0.265959 | 154.70 |
| 10m | rudb-parquet | q6 | 0.174190 | 0.178931 | 0.010044 | 0.188913 | 0.836554 | 207.81 |
| 10m | rudb-parquet | q7 | 0.009423 | 0.005028 | 0.000578 | 0.010777 | 0.009636 | 17.06 |
| 10m | rudb-parquet | q8 | 0.016793 | 0.016150 | 0.000805 | 0.022127 | 0.054093 | 29.16 |
| 10m | rudb-parquet | q9 | 0.041475 | 0.044711 | 0.002228 | 0.052560 | 0.198678 | 84.89 |
| 10m | rudb-parquet | q10 | 0.063673 | 0.064560 | 0.002635 | 0.073183 | 0.314710 | 92.28 |
| 10m | rudb-parquet | q11 | 0.049054 | 0.044338 | 0.004450 | 0.051304 | 0.200536 | 53.98 |
| 10m | rudb-parquet | q12 | 0.045468 | 0.047172 | 0.001993 | 0.053980 | 0.219088 | 59.38 |
| 10m | rudb-parquet | q13 | 0.125354 | 0.138497 | 0.012209 | 0.150222 | 0.701517 | 210.58 |
| 10m | rudb-parquet | q14 | 0.174704 | 0.188384 | 0.035064 | 0.200736 | 0.921521 | 294.78 |
| 10m | rudb-parquet | q15 | 0.148476 | 0.150949 | 0.006788 | 0.160924 | 0.764417 | 217.39 |
| 10m | rudb-parquet | q16 | 0.031018 | 0.030841 | 0.000499 | 0.038307 | 0.125919 | 75.92 |
| 10m | rudb-parquet | q17 | 0.252416 | 0.261134 | 0.023434 | 0.275788 | 1.263351 | 397.94 |
| 10m | rudb-parquet | q18 | 0.108279 | 0.110835 | 0.005007 | 0.119497 | 0.546742 | 118.00 |
| 10m | rudb-parquet | q19 | 0.470358 | 0.397740 | 0.011462 | 0.423211 | 2.150738 | 676.42 |
| 10m | rudb-parquet | q20 | 0.016223 | 0.013984 | 0.000968 | 0.020655 | 0.039072 | 41.92 |
| 10m | rudb-parquet | q21 | 0.269143 | 0.307362 | 0.017468 | 0.318744 | 1.676311 | 305.16 |
| 10m | rudb-parquet | q22 | 0.318622 | 0.335314 | 0.042743 | 0.357686 | 1.782735 | 354.89 |
| 10m | rudb-parquet | q23 | 0.772523 | 0.756341 | 0.060553 | 0.775072 | 4.276537 | 468.94 |
| 10m | rudb-parquet | q24 | 0.394209 | 0.426987 | 0.066202 | 0.442136 | 2.424430 | 370.12 |
| 10m | rudb-parquet | q25 | 0.125403 | 0.122704 | 0.004932 | 0.135543 | 0.613321 | 159.17 |
| 10m | rudb-parquet | q26 | 0.091276 | 0.095722 | 0.011280 | 0.108421 | 0.453445 | 135.66 |
| 10m | rudb-parquet | q27 | 0.125891 | 0.112957 | 0.006409 | 0.121879 | 0.574754 | 155.83 |
| 10m | rudb-parquet | q28 | 0.268018 | 0.257896 | 0.037586 | 0.275401 | 1.362918 | 298.77 |
| 10m | rudb-parquet | q29 | 0.750303 | 0.730014 | 0.069031 | 0.763489 | 3.906610 | 723.09 |
| 10m | rudb-parquet | q30 | 0.024429 | 0.020933 | 0.001047 | 0.028713 | 0.055815 | 29.80 |
| 10m | rudb-parquet | q31 | 0.128674 | 0.143365 | 0.028207 | 0.154698 | 0.670219 | 150.62 |
| 10m | rudb-parquet | q32 | 0.127690 | 0.135129 | 0.008078 | 0.147197 | 0.684921 | 170.67 |
| 10m | rudb-parquet | q33 | 0.119271 | 0.121470 | 0.005865 | 0.137884 | 0.607523 | 244.66 |
| 10m | rudb-parquet | q34 | 0.740524 | 0.661211 | 0.046359 | 0.694179 | 3.460803 | 808.06 |
| 10m | rudb-parquet | q35 | 0.627400 | 0.650990 | 0.036655 | 0.691686 | 3.431488 | 819.11 |
| 10m | rudb-parquet | q36 | 0.047023 | 0.033785 | 0.000668 | 0.042249 | 0.134970 | 70.02 |
| 10m | rudb-parquet | q37 | 0.070653 | 0.069099 | 0.006315 | 0.080528 | 0.258806 | 192.67 |
| 10m | rudb-parquet | q38 | 0.027360 | 0.024076 | 0.004509 | 0.030870 | 0.087371 | 59.80 |
| 10m | rudb-parquet | q39 | 0.035685 | 0.038171 | 0.003992 | 0.046477 | 0.123338 | 146.62 |
| 10m | rudb-parquet | q40 | 0.105705 | 0.124297 | 0.006238 | 0.137125 | 0.552967 | 347.67 |
| 10m | rudb-parquet | q41 | 0.021296 | 0.020540 | 0.001407 | 0.028126 | 0.053721 | 61.86 |
| 10m | rudb-parquet | q42 | 0.018493 | 0.018579 | 0.002847 | 0.025769 | 0.043701 | 56.91 |
| 10m | rudb-parquet | q43 | 0.018722 | 0.018401 | 0.001127 | 0.024510 | 0.051991 | 49.52 |
