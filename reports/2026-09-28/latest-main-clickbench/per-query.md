# ClickBench measurement audit

All 43 SQL queries are attempted on DuckDB native, rudb native, DuckDB Parquet, and rudb Parquet. Five hot repetitions are required for a complete row. A failed repetition invalidates that query; successful fragments are never averaged into a result.

First execution is not disk-cold: the page cache is not flushed. Each repetition uses a fresh process. Query seconds are the CLI timer, including result rendering; wall and CPU seconds and peak RSS cover the whole child process. CPU and RSS come from wait4 for that child. RSS is the maximum resident set, not allocated bytes or an incremental memory delta. Both native rows open a loaded single-file database. Both Parquet rows query the same source file with native mirroring disabled; metadata-only paths remain available. These are sample results, not official ClickBench scores.

| Size | Engine | Load wall (s) | Load CPU (s) | Load peak RSS (MiB) | Native bytes |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 0.085185 | 0.083477 | 61.53 | 1060864 |
| 1k | rudb-native | 0.182850 | 0.163938 | 41.14 | 1876635 |
| 10k | duckdb-native | 0.301813 | 0.325739 | 77.36 | 4993024 |
| 10k | rudb-native | 1.040556 | 0.954820 | 72.19 | 7732602 |
| 1m | duckdb-native | 13.297152 | 13.939996 | 968.94 | 451686400 |
| 1m | rudb-native | 5.921976 | 13.409111 | 1096.81 | 246641277 |
| 10m | duckdb-native | 286.880730 | 120.873997 | 820.33 | 2366910464 |
| 10m | rudb-native | 31.598715 | 85.186103 | 1492.08 | 1468552139 |

| Size | Engine | Complete / 43 | Query median sum (s) | Process wall median sum (s) | CPU median sum (s) | Peak RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 43 | 0.116000 | 1.629770 | 1.467870 | 35.66 |
| 1k | rudb-native | 43 | 0.075450 | 0.563184 | 0.451579 | 13.34 |
| 1k | duckdb-parquet | 43 | 0.148000 | 1.581968 | 1.476268 | 33.02 |
| 1k | rudb-parquet | 43 | 0.097245 | 0.537224 | 0.432982 | 13.11 |
| 10k | duckdb-native | 43 | 0.407000 | 3.951780 | 2.058538 | 38.61 |
| 10k | rudb-native | 43 | 0.146282 | 1.164642 | 0.727191 | 16.50 |
| 10k | duckdb-parquet | 43 | 0.788000 | 4.292464 | 2.111781 | 40.77 |
| 10k | rudb-parquet | 43 | 0.239411 | 1.054163 | 0.696653 | 16.06 |
| 1m | duckdb-native | 43 | 1.474000 | 3.036153 | 5.665424 | 259.69 |
| 1m | rudb-native | 43 | 0.348615 | 0.913806 | 1.397736 | 82.20 |
| 1m | duckdb-parquet | 43 | 2.277000 | 3.894818 | 7.021692 | 385.80 |
| 1m | rudb-parquet | 43 | 2.211793 | 2.789783 | 5.656837 | 333.89 |
| 10m | duckdb-native | 43 | 7.576000 | 9.705089 | 29.652533 | 1155.86 |
| 10m | rudb-native | 43 | 1.185415 | 1.759973 | 5.053635 | 282.33 |
| 10m | duckdb-parquet | 43 | 10.622000 | 12.675247 | 39.138627 | 948.27 |
| 10m | rudb-parquet | 43 | 8.229717 | 8.813854 | 34.238575 | 829.09 |

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
| 1k | duckdb-native | q1 | 0.000000 | 0.001000 | 0.001000 | 0.032262 | 0.029291 | 20.28 |
| 1k | duckdb-native | q2 | 0.001000 | 0.001000 | 0.000000 | 0.033492 | 0.030470 | 20.81 |
| 1k | duckdb-native | q3 | 0.001000 | 0.001000 | 0.000000 | 0.031740 | 0.029433 | 21.38 |
| 1k | duckdb-native | q4 | 0.001000 | 0.001000 | 0.000000 | 0.035284 | 0.030646 | 20.89 |
| 1k | duckdb-native | q5 | 0.002000 | 0.002000 | 0.000000 | 0.032746 | 0.031904 | 24.52 |
| 1k | duckdb-native | q6 | 0.001000 | 0.002000 | 0.001000 | 0.035630 | 0.034194 | 25.20 |
| 1k | duckdb-native | q7 | 0.001000 | 0.001000 | 0.001000 | 0.032493 | 0.029382 | 19.64 |
| 1k | duckdb-native | q8 | 0.002000 | 0.002000 | 0.000000 | 0.034018 | 0.030457 | 21.78 |
| 1k | duckdb-native | q9 | 0.004000 | 0.003000 | 0.001000 | 0.037632 | 0.033567 | 29.91 |
| 1k | duckdb-native | q10 | 0.003000 | 0.004000 | 0.001000 | 0.044730 | 0.036620 | 32.42 |
| 1k | duckdb-native | q11 | 0.003000 | 0.003000 | 0.001000 | 0.033779 | 0.033208 | 31.28 |
| 1k | duckdb-native | q12 | 0.003000 | 0.004000 | 0.001000 | 0.034947 | 0.033972 | 31.58 |
| 1k | duckdb-native | q13 | 0.002000 | 0.002000 | 0.000000 | 0.032643 | 0.031898 | 26.75 |
| 1k | duckdb-native | q14 | 0.007000 | 0.004000 | 0.002000 | 0.038955 | 0.035032 | 31.48 |
| 1k | duckdb-native | q15 | 0.002000 | 0.003000 | 0.001000 | 0.034501 | 0.031960 | 26.56 |
| 1k | duckdb-native | q16 | 0.003000 | 0.002000 | 0.001000 | 0.032673 | 0.031880 | 26.56 |
| 1k | duckdb-native | q17 | 0.004000 | 0.002000 | 0.001000 | 0.034489 | 0.032224 | 28.08 |
| 1k | duckdb-native | q18 | 0.003000 | 0.003000 | 0.002000 | 0.044413 | 0.034547 | 26.84 |
| 1k | duckdb-native | q19 | 0.002000 | 0.003000 | 0.000000 | 0.033395 | 0.031238 | 28.77 |
| 1k | duckdb-native | q20 | 0.000000 | 0.001000 | 0.000000 | 0.029585 | 0.027092 | 20.06 |
| 1k | duckdb-native | q21 | 0.001000 | 0.001000 | 0.000000 | 0.027308 | 0.025476 | 21.11 |
| 1k | duckdb-native | q22 | 0.002000 | 0.001000 | 0.000000 | 0.025293 | 0.024026 | 21.58 |
| 1k | duckdb-native | q23 | 0.001000 | 0.002000 | 0.001000 | 0.024730 | 0.023507 | 21.41 |
| 1k | duckdb-native | q24 | 0.005000 | 0.005000 | 0.001000 | 0.029585 | 0.029764 | 35.66 |
| 1k | duckdb-native | q25 | 0.002000 | 0.002000 | 0.001000 | 0.025435 | 0.024428 | 23.69 |
| 1k | duckdb-native | q26 | 0.002000 | 0.001000 | 0.000000 | 0.023588 | 0.022884 | 21.59 |
| 1k | duckdb-native | q27 | 0.001000 | 0.002000 | 0.000000 | 0.033127 | 0.028642 | 21.69 |
| 1k | duckdb-native | q28 | 0.004000 | 0.002000 | 0.000000 | 0.029962 | 0.029459 | 26.94 |
| 1k | duckdb-native | q29 | 0.005000 | 0.005000 | 0.001000 | 0.037545 | 0.037136 | 27.78 |
| 1k | duckdb-native | q30 | 0.016000 | 0.010000 | 0.002000 | 0.048180 | 0.040337 | 31.94 |
| 1k | duckdb-native | q31 | 0.003000 | 0.002000 | 0.000000 | 0.028859 | 0.028309 | 28.20 |
| 1k | duckdb-native | q32 | 0.003000 | 0.003000 | 0.014000 | 0.039853 | 0.037350 | 27.72 |
| 1k | duckdb-native | q33 | 0.007000 | 0.005000 | 0.004000 | 0.071191 | 0.052373 | 28.44 |
| 1k | duckdb-native | q34 | 0.004000 | 0.004000 | 0.007000 | 0.046109 | 0.046271 | 26.86 |
| 1k | duckdb-native | q35 | 0.003000 | 0.002000 | 0.001000 | 0.036562 | 0.037748 | 27.53 |
| 1k | duckdb-native | q36 | 0.012000 | 0.004000 | 0.001000 | 0.063157 | 0.049752 | 27.84 |
| 1k | duckdb-native | q37 | 0.009000 | 0.005000 | 0.005000 | 0.089602 | 0.053552 | 26.73 |
| 1k | duckdb-native | q38 | 0.002000 | 0.003000 | 0.002000 | 0.052482 | 0.047401 | 26.00 |
| 1k | duckdb-native | q39 | 0.002000 | 0.002000 | 0.000000 | 0.048775 | 0.042632 | 22.09 |
| 1k | duckdb-native | q40 | 0.003000 | 0.003000 | 0.001000 | 0.038859 | 0.038403 | 28.88 |
| 1k | duckdb-native | q41 | 0.003000 | 0.003000 | 0.001000 | 0.034417 | 0.034963 | 26.84 |
| 1k | duckdb-native | q42 | 0.002000 | 0.002000 | 0.000000 | 0.034164 | 0.034432 | 24.31 |
| 1k | duckdb-native | q43 | 0.002000 | 0.002000 | 0.000000 | 0.041580 | 0.040010 | 26.31 |
| 1k | rudb-native | q1 | 0.004299 | 0.001029 | 0.000644 | 0.011912 | 0.009560 | 9.62 |
| 1k | rudb-native | q2 | 0.001532 | 0.001042 | 0.000100 | 0.012284 | 0.009586 | 10.09 |
| 1k | rudb-native | q3 | 0.001403 | 0.001277 | 0.000410 | 0.011989 | 0.009755 | 10.31 |
| 1k | rudb-native | q4 | 0.001248 | 0.001186 | 0.002060 | 0.013833 | 0.010340 | 9.97 |
| 1k | rudb-native | q5 | 0.001854 | 0.001128 | 0.000222 | 0.012455 | 0.010228 | 9.64 |
| 1k | rudb-native | q6 | 0.001118 | 0.001029 | 0.000036 | 0.012300 | 0.010079 | 9.72 |
| 1k | rudb-native | q7 | 0.001059 | 0.001073 | 0.000047 | 0.012450 | 0.010037 | 9.41 |
| 1k | rudb-native | q8 | 0.001511 | 0.001627 | 0.000354 | 0.012933 | 0.009597 | 12.61 |
| 1k | rudb-native | q9 | 0.001240 | 0.001534 | 0.000077 | 0.011605 | 0.009470 | 11.44 |
| 1k | rudb-native | q10 | 0.001912 | 0.001932 | 0.000529 | 0.015454 | 0.010246 | 12.27 |
| 1k | rudb-native | q11 | 0.012580 | 0.001488 | 0.000700 | 0.012001 | 0.009214 | 11.84 |
| 1k | rudb-native | q12 | 0.001757 | 0.001624 | 0.000269 | 0.011489 | 0.009500 | 12.06 |
| 1k | rudb-native | q13 | 0.001877 | 0.001881 | 0.000137 | 0.012564 | 0.010389 | 12.33 |
| 1k | rudb-native | q14 | 0.001767 | 0.001536 | 0.000202 | 0.013025 | 0.010266 | 12.28 |
| 1k | rudb-native | q15 | 0.002387 | 0.002101 | 0.001561 | 0.012045 | 0.008749 | 12.61 |
| 1k | rudb-native | q16 | 0.001060 | 0.001272 | 0.000134 | 0.012371 | 0.009451 | 11.00 |
| 1k | rudb-native | q17 | 0.001416 | 0.001119 | 0.000147 | 0.012623 | 0.009141 | 10.64 |
| 1k | rudb-native | q18 | 0.001396 | 0.001675 | 0.000247 | 0.012449 | 0.010244 | 11.55 |
| 1k | rudb-native | q19 | 0.001455 | 0.001236 | 0.000191 | 0.011112 | 0.009230 | 11.05 |
| 1k | rudb-native | q20 | 0.001193 | 0.000905 | 0.000201 | 0.010619 | 0.008380 | 10.73 |
| 1k | rudb-native | q21 | 0.001431 | 0.001580 | 0.000599 | 0.009754 | 0.007495 | 11.95 |
| 1k | rudb-native | q22 | 0.001538 | 0.001229 | 0.000257 | 0.008708 | 0.007206 | 11.77 |
| 1k | rudb-native | q23 | 0.001455 | 0.001287 | 0.000025 | 0.007683 | 0.006752 | 12.05 |
| 1k | rudb-native | q24 | 0.001116 | 0.001235 | 0.000228 | 0.008667 | 0.007395 | 10.91 |
| 1k | rudb-native | q25 | 0.001406 | 0.001051 | 0.000120 | 0.007852 | 0.006581 | 10.45 |
| 1k | rudb-native | q26 | 0.000996 | 0.001138 | 0.000318 | 0.009388 | 0.007897 | 10.84 |
| 1k | rudb-native | q27 | 0.001219 | 0.001861 | 0.000506 | 0.011264 | 0.009604 | 11.91 |
| 1k | rudb-native | q28 | 0.002014 | 0.001465 | 0.000322 | 0.010733 | 0.009668 | 12.48 |
| 1k | rudb-native | q29 | 0.002353 | 0.002201 | 0.000957 | 0.011798 | 0.009691 | 12.58 |
| 1k | rudb-native | q30 | 0.003451 | 0.004500 | 0.001579 | 0.013912 | 0.012882 | 12.56 |
| 1k | rudb-native | q31 | 0.001480 | 0.001577 | 0.000116 | 0.010421 | 0.008638 | 12.25 |
| 1k | rudb-native | q32 | 0.001945 | 0.001836 | 0.001464 | 0.014386 | 0.010864 | 12.23 |
| 1k | rudb-native | q33 | 0.001260 | 0.002561 | 0.000657 | 0.023784 | 0.019054 | 11.83 |
| 1k | rudb-native | q34 | 0.003322 | 0.002042 | 0.000259 | 0.019700 | 0.016034 | 11.83 |
| 1k | rudb-native | q35 | 0.002030 | 0.002794 | 0.001053 | 0.013886 | 0.012712 | 12.92 |
| 1k | rudb-native | q36 | 0.004269 | 0.002555 | 0.000496 | 0.019051 | 0.014795 | 12.83 |
| 1k | rudb-native | q37 | 0.006897 | 0.003843 | 0.000675 | 0.027357 | 0.018112 | 13.34 |
| 1k | rudb-native | q38 | 0.002876 | 0.002866 | 0.000998 | 0.017669 | 0.014459 | 13.27 |
| 1k | rudb-native | q39 | 0.001513 | 0.002428 | 0.000836 | 0.018311 | 0.014867 | 12.72 |
| 1k | rudb-native | q40 | 0.002136 | 0.001941 | 0.001143 | 0.013942 | 0.012091 | 12.52 |
| 1k | rudb-native | q41 | 0.001709 | 0.001726 | 0.000205 | 0.011488 | 0.009550 | 11.92 |
| 1k | rudb-native | q42 | 0.002261 | 0.002128 | 0.000450 | 0.011934 | 0.010056 | 11.66 |
| 1k | rudb-native | q43 | 0.001573 | 0.001913 | 0.000286 | 0.013983 | 0.011714 | 12.42 |
| 1k | duckdb-parquet | q1 | 0.002000 | 0.002000 | 0.001000 | 0.032822 | 0.029688 | 19.78 |
| 1k | duckdb-parquet | q2 | 0.002000 | 0.002000 | 0.001000 | 0.035255 | 0.031595 | 20.75 |
| 1k | duckdb-parquet | q3 | 0.003000 | 0.002000 | 0.000000 | 0.034029 | 0.031572 | 20.42 |
| 1k | duckdb-parquet | q4 | 0.002000 | 0.002000 | 0.000000 | 0.035201 | 0.031297 | 20.12 |
| 1k | duckdb-parquet | q5 | 0.006000 | 0.002000 | 0.001000 | 0.034063 | 0.031446 | 24.62 |
| 1k | duckdb-parquet | q6 | 0.003000 | 0.003000 | 0.000000 | 0.035902 | 0.033190 | 24.62 |
| 1k | duckdb-parquet | q7 | 0.001000 | 0.001000 | 0.001000 | 0.033138 | 0.031384 | 19.81 |
| 1k | duckdb-parquet | q8 | 0.002000 | 0.003000 | 0.001000 | 0.034453 | 0.031299 | 21.53 |
| 1k | duckdb-parquet | q9 | 0.003000 | 0.003000 | 0.000000 | 0.033394 | 0.033384 | 30.72 |
| 1k | duckdb-parquet | q10 | 0.003000 | 0.004000 | 0.001000 | 0.039863 | 0.035974 | 33.02 |
| 1k | duckdb-parquet | q11 | 0.003000 | 0.003000 | 0.000000 | 0.032891 | 0.032307 | 30.62 |
| 1k | duckdb-parquet | q12 | 0.005000 | 0.003000 | 0.000000 | 0.034681 | 0.034588 | 31.14 |
| 1k | duckdb-parquet | q13 | 0.004000 | 0.003000 | 0.001000 | 0.037087 | 0.031122 | 25.81 |
| 1k | duckdb-parquet | q14 | 0.004000 | 0.005000 | 0.001000 | 0.041959 | 0.036195 | 32.08 |
| 1k | duckdb-parquet | q15 | 0.004000 | 0.004000 | 0.001000 | 0.033510 | 0.031693 | 25.94 |
| 1k | duckdb-parquet | q16 | 0.003000 | 0.003000 | 0.001000 | 0.035379 | 0.032049 | 25.16 |
| 1k | duckdb-parquet | q17 | 0.004000 | 0.003000 | 0.002000 | 0.036236 | 0.035488 | 26.45 |
| 1k | duckdb-parquet | q18 | 0.008000 | 0.004000 | 0.001000 | 0.034712 | 0.032049 | 25.72 |
| 1k | duckdb-parquet | q19 | 0.004000 | 0.003000 | 0.001000 | 0.032938 | 0.030726 | 27.75 |
| 1k | duckdb-parquet | q20 | 0.001000 | 0.002000 | 0.001000 | 0.032647 | 0.029631 | 20.59 |
| 1k | duckdb-parquet | q21 | 0.002000 | 0.002000 | 0.000000 | 0.031222 | 0.028260 | 21.19 |
| 1k | duckdb-parquet | q22 | 0.002000 | 0.002000 | 0.000000 | 0.025363 | 0.024068 | 21.53 |
| 1k | duckdb-parquet | q23 | 0.003000 | 0.002000 | 0.000000 | 0.024038 | 0.024232 | 21.55 |
| 1k | duckdb-parquet | q24 | 0.006000 | 0.006000 | 0.001000 | 0.029084 | 0.031902 | 32.80 |
| 1k | duckdb-parquet | q25 | 0.002000 | 0.003000 | 0.000000 | 0.025030 | 0.025469 | 23.09 |
| 1k | duckdb-parquet | q26 | 0.002000 | 0.002000 | 0.000000 | 0.026330 | 0.025706 | 20.94 |
| 1k | duckdb-parquet | q27 | 0.002000 | 0.002000 | 0.000000 | 0.030376 | 0.027115 | 21.42 |
| 1k | duckdb-parquet | q28 | 0.004000 | 0.003000 | 0.000000 | 0.033680 | 0.032391 | 26.59 |
| 1k | duckdb-parquet | q29 | 0.003000 | 0.006000 | 0.002000 | 0.039384 | 0.035319 | 27.22 |
| 1k | duckdb-parquet | q30 | 0.009000 | 0.011000 | 0.002000 | 0.042253 | 0.037759 | 31.66 |
| 1k | duckdb-parquet | q31 | 0.003000 | 0.003000 | 0.001000 | 0.031512 | 0.029548 | 27.64 |
| 1k | duckdb-parquet | q32 | 0.015000 | 0.004000 | 0.002000 | 0.044941 | 0.036602 | 27.44 |
| 1k | duckdb-parquet | q33 | 0.005000 | 0.005000 | 0.007000 | 0.051263 | 0.051486 | 27.52 |
| 1k | duckdb-parquet | q34 | 0.006000 | 0.004000 | 0.002000 | 0.049024 | 0.047135 | 26.50 |
| 1k | duckdb-parquet | q35 | 0.003000 | 0.004000 | 0.001000 | 0.041779 | 0.039908 | 26.91 |
| 1k | duckdb-parquet | q36 | 0.008000 | 0.005000 | 0.001000 | 0.052848 | 0.046793 | 27.72 |
| 1k | duckdb-parquet | q37 | 0.009000 | 0.006000 | 0.003000 | 0.067784 | 0.055015 | 26.16 |
| 1k | duckdb-parquet | q38 | 0.005000 | 0.004000 | 0.001000 | 0.045200 | 0.043052 | 26.00 |
| 1k | duckdb-parquet | q39 | 0.003000 | 0.003000 | 0.000000 | 0.039356 | 0.037513 | 21.31 |
| 1k | duckdb-parquet | q40 | 0.004000 | 0.004000 | 0.001000 | 0.040157 | 0.039788 | 29.20 |
| 1k | duckdb-parquet | q41 | 0.003000 | 0.004000 | 0.001000 | 0.034175 | 0.034178 | 26.16 |
| 1k | duckdb-parquet | q42 | 0.003000 | 0.003000 | 0.000000 | 0.034421 | 0.035341 | 24.33 |
| 1k | duckdb-parquet | q43 | 0.003000 | 0.003000 | 0.001000 | 0.042588 | 0.041011 | 25.00 |
| 1k | rudb-parquet | q1 | 0.001795 | 0.001392 | 0.000143 | 0.011961 | 0.009279 | 9.86 |
| 1k | rudb-parquet | q2 | 0.001771 | 0.001813 | 0.000046 | 0.012360 | 0.010622 | 10.47 |
| 1k | rudb-parquet | q3 | 0.001777 | 0.001684 | 0.000288 | 0.012364 | 0.009677 | 9.89 |
| 1k | rudb-parquet | q4 | 0.001556 | 0.002800 | 0.001016 | 0.012789 | 0.010403 | 10.78 |
| 1k | rudb-parquet | q5 | 0.003031 | 0.001834 | 0.000325 | 0.013227 | 0.009486 | 10.23 |
| 1k | rudb-parquet | q6 | 0.002584 | 0.001715 | 0.000125 | 0.012088 | 0.009698 | 10.44 |
| 1k | rudb-parquet | q7 | 0.001364 | 0.001140 | 0.000052 | 0.011300 | 0.009485 | 9.44 |
| 1k | rudb-parquet | q8 | 0.001897 | 0.001700 | 0.000159 | 0.011861 | 0.009751 | 11.02 |
| 1k | rudb-parquet | q9 | 0.001239 | 0.002145 | 0.000233 | 0.011616 | 0.009673 | 11.08 |
| 1k | rudb-parquet | q10 | 0.001804 | 0.002743 | 0.001249 | 0.014268 | 0.010548 | 11.88 |
| 1k | rudb-parquet | q11 | 0.001801 | 0.001942 | 0.000459 | 0.012261 | 0.010063 | 11.61 |
| 1k | rudb-parquet | q12 | 0.001537 | 0.001484 | 0.000030 | 0.011062 | 0.009148 | 10.94 |
| 1k | rudb-parquet | q13 | 0.004969 | 0.002013 | 0.000352 | 0.011863 | 0.009668 | 11.27 |
| 1k | rudb-parquet | q14 | 0.001788 | 0.001926 | 0.000188 | 0.012625 | 0.009246 | 11.38 |
| 1k | rudb-parquet | q15 | 0.001544 | 0.001636 | 0.000270 | 0.011145 | 0.008589 | 11.47 |
| 1k | rudb-parquet | q16 | 0.001476 | 0.001923 | 0.000663 | 0.011553 | 0.009664 | 11.44 |
| 1k | rudb-parquet | q17 | 0.002084 | 0.001566 | 0.000486 | 0.011489 | 0.009541 | 10.94 |
| 1k | rudb-parquet | q18 | 0.002339 | 0.001921 | 0.000123 | 0.012320 | 0.010205 | 11.44 |
| 1k | rudb-parquet | q19 | 0.003005 | 0.001773 | 0.000897 | 0.010025 | 0.008670 | 11.73 |
| 1k | rudb-parquet | q20 | 0.001479 | 0.001266 | 0.000292 | 0.010148 | 0.008131 | 10.14 |
| 1k | rudb-parquet | q21 | 0.001522 | 0.003896 | 0.002194 | 0.012751 | 0.007531 | 11.08 |
| 1k | rudb-parquet | q22 | 0.001629 | 0.001645 | 0.000146 | 0.009536 | 0.007569 | 11.03 |
| 1k | rudb-parquet | q23 | 0.001774 | 0.001825 | 0.000161 | 0.008072 | 0.006867 | 10.50 |
| 1k | rudb-parquet | q24 | 0.001823 | 0.001373 | 0.000177 | 0.007619 | 0.006702 | 10.50 |
| 1k | rudb-parquet | q25 | 0.001280 | 0.001246 | 0.000159 | 0.007221 | 0.006203 | 10.12 |
| 1k | rudb-parquet | q26 | 0.001229 | 0.001154 | 0.000115 | 0.007947 | 0.006846 | 10.67 |
| 1k | rudb-parquet | q27 | 0.001145 | 0.001408 | 0.000195 | 0.010797 | 0.008579 | 10.88 |
| 1k | rudb-parquet | q28 | 0.002277 | 0.001908 | 0.000349 | 0.009990 | 0.009023 | 11.84 |
| 1k | rudb-parquet | q29 | 0.002848 | 0.002900 | 0.001313 | 0.015152 | 0.010135 | 12.59 |
| 1k | rudb-parquet | q30 | 0.009637 | 0.006411 | 0.000160 | 0.016388 | 0.013744 | 13.11 |
| 1k | rudb-parquet | q31 | 0.001703 | 0.002700 | 0.000362 | 0.011366 | 0.009174 | 11.81 |
| 1k | rudb-parquet | q32 | 0.005388 | 0.002357 | 0.000594 | 0.011248 | 0.009515 | 12.20 |
| 1k | rudb-parquet | q33 | 0.008482 | 0.004543 | 0.000261 | 0.016768 | 0.014431 | 12.09 |
| 1k | rudb-parquet | q34 | 0.003643 | 0.003258 | 0.001085 | 0.016058 | 0.014184 | 11.95 |
| 1k | rudb-parquet | q35 | 0.002564 | 0.002606 | 0.000849 | 0.014138 | 0.012029 | 11.94 |
| 1k | rudb-parquet | q36 | 0.004837 | 0.003668 | 0.000696 | 0.019548 | 0.015101 | 12.00 |
| 1k | rudb-parquet | q37 | 0.004359 | 0.004022 | 0.001259 | 0.029056 | 0.018249 | 12.34 |
| 1k | rudb-parquet | q38 | 0.002700 | 0.003032 | 0.000191 | 0.016944 | 0.014917 | 11.95 |
| 1k | rudb-parquet | q39 | 0.001914 | 0.002054 | 0.000389 | 0.010602 | 0.009709 | 11.77 |
| 1k | rudb-parquet | q40 | 0.004028 | 0.002711 | 0.000425 | 0.012398 | 0.011119 | 12.23 |
| 1k | rudb-parquet | q41 | 0.001874 | 0.001864 | 0.000028 | 0.011060 | 0.009243 | 10.55 |
| 1k | rudb-parquet | q42 | 0.001919 | 0.001948 | 0.000254 | 0.010837 | 0.009125 | 10.48 |
| 1k | rudb-parquet | q43 | 0.002159 | 0.002300 | 0.000396 | 0.013403 | 0.011440 | 11.36 |
| 10k | duckdb-native | q1 | 0.001000 | 0.001000 | 0.001000 | 0.037830 | 0.035450 | 19.62 |
| 10k | duckdb-native | q2 | 0.001000 | 0.001000 | 0.001000 | 0.044379 | 0.039751 | 21.00 |
| 10k | duckdb-native | q3 | 0.001000 | 0.001000 | 0.001000 | 0.042922 | 0.039673 | 21.38 |
| 10k | duckdb-native | q4 | 0.001000 | 0.001000 | 0.000000 | 0.041980 | 0.038129 | 21.00 |
| 10k | duckdb-native | q5 | 0.003000 | 0.002000 | 0.001000 | 0.043629 | 0.041975 | 25.16 |
| 10k | duckdb-native | q6 | 0.002000 | 0.003000 | 0.002000 | 0.047888 | 0.044291 | 25.69 |
| 10k | duckdb-native | q7 | 0.001000 | 0.001000 | 0.001000 | 0.077518 | 0.047839 | 20.75 |
| 10k | duckdb-native | q8 | 0.008000 | 0.003000 | 0.001000 | 0.045764 | 0.042860 | 22.31 |
| 10k | duckdb-native | q9 | 0.006000 | 0.008000 | 0.002000 | 0.076733 | 0.051437 | 32.06 |
| 10k | duckdb-native | q10 | 0.009000 | 0.016000 | 0.010000 | 0.083339 | 0.052678 | 32.83 |
| 10k | duckdb-native | q11 | 0.004000 | 0.004000 | 0.002000 | 0.054570 | 0.047127 | 31.64 |
| 10k | duckdb-native | q12 | 0.005000 | 0.004000 | 0.001000 | 0.059385 | 0.052585 | 33.19 |
| 10k | duckdb-native | q13 | 0.002000 | 0.003000 | 0.001000 | 0.036454 | 0.036665 | 27.33 |
| 10k | duckdb-native | q14 | 0.003000 | 0.003000 | 0.001000 | 0.038166 | 0.039230 | 33.36 |
| 10k | duckdb-native | q15 | 0.003000 | 0.004000 | 0.001000 | 0.060338 | 0.048969 | 27.55 |
| 10k | duckdb-native | q16 | 0.006000 | 0.004000 | 0.000000 | 0.074272 | 0.052098 | 27.38 |
| 10k | duckdb-native | q17 | 0.005000 | 0.007000 | 0.011000 | 0.075092 | 0.054286 | 28.81 |
| 10k | duckdb-native | q18 | 0.034000 | 0.004000 | 0.000000 | 0.067856 | 0.050537 | 27.72 |
| 10k | duckdb-native | q19 | 0.008000 | 0.005000 | 0.009000 | 0.054002 | 0.050024 | 30.91 |
| 10k | duckdb-native | q20 | 0.001000 | 0.001000 | 0.001000 | 0.074776 | 0.044840 | 21.02 |
| 10k | duckdb-native | q21 | 0.003000 | 0.002000 | 0.000000 | 0.035657 | 0.033351 | 21.28 |
| 10k | duckdb-native | q22 | 0.002000 | 0.003000 | 0.000000 | 0.051170 | 0.041658 | 22.84 |
| 10k | duckdb-native | q23 | 0.009000 | 0.010000 | 0.004000 | 0.068742 | 0.053967 | 29.41 |
| 10k | duckdb-native | q24 | 0.034000 | 0.012000 | 0.010000 | 0.069721 | 0.061544 | 38.61 |
| 10k | duckdb-native | q25 | 0.007000 | 0.004000 | 0.000000 | 0.058776 | 0.049362 | 24.11 |
| 10k | duckdb-native | q26 | 0.004000 | 0.002000 | 0.000000 | 0.065882 | 0.042274 | 21.41 |
| 10k | duckdb-native | q27 | 0.003000 | 0.002000 | 0.000000 | 0.050148 | 0.041870 | 21.59 |
| 10k | duckdb-native | q28 | 0.007000 | 0.003000 | 0.000000 | 0.037414 | 0.038182 | 27.11 |
| 10k | duckdb-native | q29 | 0.016000 | 0.015000 | 0.007000 | 0.055502 | 0.057592 | 29.33 |
| 10k | duckdb-native | q30 | 0.015000 | 0.014000 | 0.005000 | 0.060413 | 0.054707 | 31.83 |
| 10k | duckdb-native | q31 | 0.005000 | 0.007000 | 0.002000 | 0.072285 | 0.052981 | 28.14 |
| 10k | duckdb-native | q32 | 0.008000 | 0.008000 | 0.007000 | 0.080105 | 0.050355 | 28.88 |
| 10k | duckdb-native | q33 | 0.007000 | 0.018000 | 0.079000 | 0.137916 | 0.053715 | 29.41 |
| 10k | duckdb-native | q34 | 0.025000 | 0.060000 | 0.095000 | 0.390302 | 0.057094 | 27.95 |
| 10k | duckdb-native | q35 | 0.115000 | 0.033000 | 0.024000 | 0.298961 | 0.056318 | 29.78 |
| 10k | duckdb-native | q36 | 0.063000 | 0.031000 | 0.037000 | 0.260603 | 0.055787 | 28.27 |
| 10k | duckdb-native | q37 | 0.076000 | 0.010000 | 0.023000 | 0.141410 | 0.052864 | 27.44 |
| 10k | duckdb-native | q38 | 0.150000 | 0.008000 | 0.004000 | 0.071921 | 0.051407 | 27.78 |
| 10k | duckdb-native | q39 | 0.005000 | 0.006000 | 0.013000 | 0.119791 | 0.050732 | 26.55 |
| 10k | duckdb-native | q40 | 0.020000 | 0.016000 | 0.159000 | 0.154667 | 0.054365 | 29.84 |
| 10k | duckdb-native | q41 | 0.005000 | 0.022000 | 0.047000 | 0.344665 | 0.053307 | 26.56 |
| 10k | duckdb-native | q42 | 0.185000 | 0.043000 | 0.146000 | 0.153462 | 0.052436 | 26.92 |
| 10k | duckdb-native | q43 | 0.002000 | 0.002000 | 0.001000 | 0.035374 | 0.032226 | 26.64 |
| 10k | rudb-native | q1 | 0.001617 | 0.001109 | 0.000074 | 0.012740 | 0.010799 | 10.55 |
| 10k | rudb-native | q2 | 0.001200 | 0.001354 | 0.000467 | 0.015211 | 0.011424 | 11.34 |
| 10k | rudb-native | q3 | 0.001099 | 0.001401 | 0.000191 | 0.014885 | 0.012383 | 10.52 |
| 10k | rudb-native | q4 | 0.002066 | 0.001556 | 0.000494 | 0.015745 | 0.013373 | 11.25 |
| 10k | rudb-native | q5 | 0.001072 | 0.001163 | 0.000054 | 0.012811 | 0.011336 | 10.34 |
| 10k | rudb-native | q6 | 0.001224 | 0.001438 | 0.000486 | 0.015591 | 0.013305 | 10.66 |
| 10k | rudb-native | q7 | 0.001860 | 0.001694 | 0.000582 | 0.025550 | 0.017622 | 11.06 |
| 10k | rudb-native | q8 | 0.010588 | 0.003006 | 0.000923 | 0.021349 | 0.017680 | 12.89 |
| 10k | rudb-native | q9 | 0.003902 | 0.002726 | 0.000420 | 0.027407 | 0.017010 | 12.52 |
| 10k | rudb-native | q10 | 0.005356 | 0.005042 | 0.006775 | 0.030518 | 0.022181 | 13.64 |
| 10k | rudb-native | q11 | 0.002678 | 0.002656 | 0.000258 | 0.018720 | 0.015653 | 12.81 |
| 10k | rudb-native | q12 | 0.002275 | 0.002226 | 0.000141 | 0.016942 | 0.014415 | 13.12 |
| 10k | rudb-native | q13 | 0.002097 | 0.002118 | 0.000153 | 0.013652 | 0.012055 | 12.84 |
| 10k | rudb-native | q14 | 0.002060 | 0.002294 | 0.000580 | 0.015009 | 0.013867 | 13.09 |
| 10k | rudb-native | q15 | 0.003444 | 0.004637 | 0.001498 | 0.025492 | 0.020480 | 13.44 |
| 10k | rudb-native | q16 | 0.003975 | 0.005595 | 0.007933 | 0.031180 | 0.020773 | 13.16 |
| 10k | rudb-native | q17 | 0.005209 | 0.003938 | 0.001112 | 0.022303 | 0.019474 | 13.66 |
| 10k | rudb-native | q18 | 0.002481 | 0.002937 | 0.001176 | 0.020913 | 0.016844 | 12.38 |
| 10k | rudb-native | q19 | 0.020797 | 0.003727 | 0.000707 | 0.019902 | 0.016813 | 14.08 |
| 10k | rudb-native | q20 | 0.004082 | 0.001901 | 0.000484 | 0.021699 | 0.015357 | 11.19 |
| 10k | rudb-native | q21 | 0.003082 | 0.001973 | 0.000498 | 0.012888 | 0.010852 | 12.33 |
| 10k | rudb-native | q22 | 0.001796 | 0.003020 | 0.003249 | 0.021728 | 0.016419 | 12.89 |
| 10k | rudb-native | q23 | 0.004386 | 0.004402 | 0.000472 | 0.023950 | 0.019361 | 15.08 |
| 10k | rudb-native | q24 | 0.009117 | 0.002327 | 0.001063 | 0.018624 | 0.015388 | 12.16 |
| 10k | rudb-native | q25 | 0.004208 | 0.002640 | 0.000041 | 0.019127 | 0.016687 | 12.53 |
| 10k | rudb-native | q26 | 0.002445 | 0.002523 | 0.000462 | 0.020904 | 0.017352 | 12.20 |
| 10k | rudb-native | q27 | 0.006080 | 0.002621 | 0.001014 | 0.018482 | 0.013543 | 11.64 |
| 10k | rudb-native | q28 | 0.003956 | 0.002688 | 0.000985 | 0.015373 | 0.014383 | 14.00 |
| 10k | rudb-native | q29 | 0.003928 | 0.005161 | 0.000799 | 0.018213 | 0.020031 | 16.50 |
| 10k | rudb-native | q30 | 0.007174 | 0.007103 | 0.001259 | 0.021939 | 0.018462 | 12.34 |
| 10k | rudb-native | q31 | 0.003558 | 0.004875 | 0.003084 | 0.051617 | 0.017548 | 13.72 |
| 10k | rudb-native | q32 | 0.002942 | 0.002808 | 0.002582 | 0.031998 | 0.018005 | 12.39 |
| 10k | rudb-native | q33 | 0.002586 | 0.003150 | 0.004583 | 0.058195 | 0.019469 | 11.95 |
| 10k | rudb-native | q34 | 0.002581 | 0.003130 | 0.001681 | 0.048899 | 0.017514 | 11.88 |
| 10k | rudb-native | q35 | 0.102558 | 0.005850 | 0.012126 | 0.049240 | 0.022363 | 14.06 |
| 10k | rudb-native | q36 | 0.003222 | 0.002767 | 0.001126 | 0.059184 | 0.019250 | 12.16 |
| 10k | rudb-native | q37 | 0.008219 | 0.006118 | 0.000779 | 0.035828 | 0.023135 | 13.69 |
| 10k | rudb-native | q38 | 0.005011 | 0.004611 | 0.001405 | 0.032803 | 0.021697 | 13.81 |
| 10k | rudb-native | q39 | 0.002566 | 0.004803 | 0.001742 | 0.027061 | 0.018725 | 13.34 |
| 10k | rudb-native | q40 | 0.006355 | 0.008427 | 0.002840 | 0.054244 | 0.020901 | 13.55 |
| 10k | rudb-native | q41 | 0.003513 | 0.003990 | 0.000681 | 0.042869 | 0.020592 | 13.59 |
| 10k | rudb-native | q42 | 0.020576 | 0.006425 | 0.007679 | 0.068692 | 0.020522 | 13.55 |
| 10k | rudb-native | q43 | 0.002081 | 0.002350 | 0.000766 | 0.015165 | 0.012148 | 12.86 |
| 10k | duckdb-parquet | q1 | 0.003000 | 0.001000 | 0.001000 | 0.037732 | 0.035678 | 20.53 |
| 10k | duckdb-parquet | q2 | 0.003000 | 0.002000 | 0.000000 | 0.039386 | 0.037915 | 20.52 |
| 10k | duckdb-parquet | q3 | 0.002000 | 0.002000 | 0.000000 | 0.040438 | 0.037310 | 21.00 |
| 10k | duckdb-parquet | q4 | 0.003000 | 0.002000 | 0.000000 | 0.040972 | 0.038001 | 20.81 |
| 10k | duckdb-parquet | q5 | 0.003000 | 0.003000 | 0.001000 | 0.043817 | 0.041391 | 25.22 |
| 10k | duckdb-parquet | q6 | 0.004000 | 0.003000 | 0.002000 | 0.045817 | 0.041923 | 25.09 |
| 10k | duckdb-parquet | q7 | 0.002000 | 0.002000 | 0.001000 | 0.070113 | 0.044709 | 20.67 |
| 10k | duckdb-parquet | q8 | 0.005000 | 0.004000 | 0.000000 | 0.054923 | 0.044890 | 21.69 |
| 10k | duckdb-parquet | q9 | 0.011000 | 0.007000 | 0.006000 | 0.096762 | 0.055467 | 32.67 |
| 10k | duckdb-parquet | q10 | 0.007000 | 0.015000 | 0.015000 | 0.081584 | 0.056065 | 33.31 |
| 10k | duckdb-parquet | q11 | 0.006000 | 0.005000 | 0.003000 | 0.071946 | 0.048319 | 31.33 |
| 10k | duckdb-parquet | q12 | 0.006000 | 0.005000 | 0.004000 | 0.040927 | 0.043709 | 31.58 |
| 10k | duckdb-parquet | q13 | 0.003000 | 0.004000 | 0.001000 | 0.037832 | 0.038067 | 26.08 |
| 10k | duckdb-parquet | q14 | 0.004000 | 0.005000 | 0.002000 | 0.040102 | 0.041124 | 33.19 |
| 10k | duckdb-parquet | q15 | 0.004000 | 0.004000 | 0.002000 | 0.051329 | 0.046801 | 27.20 |
| 10k | duckdb-parquet | q16 | 0.017000 | 0.006000 | 0.003000 | 0.068209 | 0.051797 | 26.17 |
| 10k | duckdb-parquet | q17 | 0.006000 | 0.007000 | 0.001000 | 0.051692 | 0.050956 | 27.97 |
| 10k | duckdb-parquet | q18 | 0.007000 | 0.008000 | 0.001000 | 0.053569 | 0.047511 | 27.17 |
| 10k | duckdb-parquet | q19 | 0.008000 | 0.010000 | 0.009000 | 0.059737 | 0.054164 | 29.72 |
| 10k | duckdb-parquet | q20 | 0.002000 | 0.003000 | 0.001000 | 0.060787 | 0.043284 | 20.48 |
| 10k | duckdb-parquet | q21 | 0.004000 | 0.004000 | 0.000000 | 0.034971 | 0.033023 | 21.75 |
| 10k | duckdb-parquet | q22 | 0.005000 | 0.005000 | 0.002000 | 0.051662 | 0.048238 | 23.05 |
| 10k | duckdb-parquet | q23 | 0.006000 | 0.010000 | 0.002000 | 0.066668 | 0.050592 | 31.50 |
| 10k | duckdb-parquet | q24 | 0.024000 | 0.031000 | 0.016000 | 0.083798 | 0.072676 | 40.77 |
| 10k | duckdb-parquet | q25 | 0.009000 | 0.007000 | 0.001000 | 0.062140 | 0.049774 | 24.47 |
| 10k | duckdb-parquet | q26 | 0.004000 | 0.004000 | 0.002000 | 0.058223 | 0.044170 | 21.44 |
| 10k | duckdb-parquet | q27 | 0.015000 | 0.004000 | 0.001000 | 0.047121 | 0.043060 | 21.45 |
| 10k | duckdb-parquet | q28 | 0.004000 | 0.005000 | 0.000000 | 0.038779 | 0.039270 | 27.30 |
| 10k | duckdb-parquet | q29 | 0.017000 | 0.020000 | 0.004000 | 0.061259 | 0.063659 | 29.48 |
| 10k | duckdb-parquet | q30 | 0.017000 | 0.012000 | 0.005000 | 0.057384 | 0.050988 | 30.94 |
| 10k | duckdb-parquet | q31 | 0.006000 | 0.009000 | 0.000000 | 0.062785 | 0.050853 | 27.83 |
| 10k | duckdb-parquet | q32 | 0.013000 | 0.008000 | 0.009000 | 0.064555 | 0.052724 | 27.97 |
| 10k | duckdb-parquet | q33 | 0.007000 | 0.011000 | 0.059000 | 0.082734 | 0.056318 | 28.48 |
| 10k | duckdb-parquet | q34 | 0.164000 | 0.058000 | 0.096000 | 0.374238 | 0.068695 | 29.55 |
| 10k | duckdb-parquet | q35 | 0.018000 | 0.193000 | 0.058000 | 0.379969 | 0.060919 | 29.84 |
| 10k | duckdb-parquet | q36 | 0.024000 | 0.051000 | 0.137000 | 0.379252 | 0.056056 | 27.28 |
| 10k | duckdb-parquet | q37 | 0.008000 | 0.008000 | 0.005000 | 0.103013 | 0.053277 | 27.03 |
| 10k | duckdb-parquet | q38 | 0.006000 | 0.010000 | 0.003000 | 0.086547 | 0.056541 | 26.59 |
| 10k | duckdb-parquet | q39 | 0.044000 | 0.014000 | 0.013000 | 0.096749 | 0.054148 | 27.70 |
| 10k | duckdb-parquet | q40 | 0.008000 | 0.013000 | 0.005000 | 0.113894 | 0.059655 | 29.81 |
| 10k | duckdb-parquet | q41 | 0.009000 | 0.131000 | 0.104000 | 0.452059 | 0.060703 | 26.08 |
| 10k | duckdb-parquet | q42 | 0.008000 | 0.078000 | 0.076000 | 0.409551 | 0.053586 | 26.00 |
| 10k | duckdb-parquet | q43 | 0.003000 | 0.004000 | 0.000000 | 0.037439 | 0.033775 | 25.62 |
| 10k | rudb-parquet | q1 | 0.001599 | 0.001569 | 0.000053 | 0.011870 | 0.009887 | 10.42 |
| 10k | rudb-parquet | q2 | 0.001788 | 0.001910 | 0.000789 | 0.012543 | 0.011138 | 11.14 |
| 10k | rudb-parquet | q3 | 0.001947 | 0.001804 | 0.000535 | 0.011953 | 0.010068 | 11.69 |
| 10k | rudb-parquet | q4 | 0.002246 | 0.001827 | 0.000298 | 0.012253 | 0.010603 | 11.39 |
| 10k | rudb-parquet | q5 | 0.002062 | 0.002055 | 0.000343 | 0.012365 | 0.010376 | 10.98 |
| 10k | rudb-parquet | q6 | 0.002616 | 0.002780 | 0.002952 | 0.013300 | 0.011355 | 12.33 |
| 10k | rudb-parquet | q7 | 0.001347 | 0.004571 | 0.002723 | 0.024313 | 0.013460 | 10.25 |
| 10k | rudb-parquet | q8 | 0.003587 | 0.002670 | 0.000386 | 0.017282 | 0.012988 | 11.17 |
| 10k | rudb-parquet | q9 | 0.002591 | 0.003940 | 0.001310 | 0.019965 | 0.016138 | 12.14 |
| 10k | rudb-parquet | q10 | 0.004176 | 0.005980 | 0.002028 | 0.023836 | 0.018701 | 12.81 |
| 10k | rudb-parquet | q11 | 0.003315 | 0.003642 | 0.001202 | 0.027869 | 0.015798 | 12.17 |
| 10k | rudb-parquet | q12 | 0.003686 | 0.003293 | 0.002703 | 0.017061 | 0.014632 | 12.50 |
| 10k | rudb-parquet | q13 | 0.002206 | 0.002878 | 0.000747 | 0.013002 | 0.010683 | 12.23 |
| 10k | rudb-parquet | q14 | 0.002299 | 0.002513 | 0.000073 | 0.012334 | 0.010527 | 11.70 |
| 10k | rudb-parquet | q15 | 0.002275 | 0.005890 | 0.002101 | 0.024077 | 0.016042 | 12.50 |
| 10k | rudb-parquet | q16 | 0.003966 | 0.003365 | 0.000229 | 0.020052 | 0.015536 | 12.31 |
| 10k | rudb-parquet | q17 | 0.004867 | 0.005489 | 0.000831 | 0.021884 | 0.018632 | 13.16 |
| 10k | rudb-parquet | q18 | 0.004626 | 0.003623 | 0.001027 | 0.021660 | 0.014936 | 11.53 |
| 10k | rudb-parquet | q19 | 0.008497 | 0.005980 | 0.000649 | 0.025423 | 0.019267 | 14.17 |
| 10k | rudb-parquet | q20 | 0.002711 | 0.002168 | 0.000438 | 0.020041 | 0.013411 | 10.72 |
| 10k | rudb-parquet | q21 | 0.004632 | 0.004186 | 0.000298 | 0.014650 | 0.012587 | 12.83 |
| 10k | rudb-parquet | q22 | 0.004447 | 0.006241 | 0.001284 | 0.019500 | 0.016488 | 13.94 |
| 10k | rudb-parquet | q23 | 0.010114 | 0.016239 | 0.016279 | 0.035255 | 0.023149 | 16.06 |
| 10k | rudb-parquet | q24 | 0.005732 | 0.006135 | 0.000454 | 0.021142 | 0.018128 | 12.75 |
| 10k | rudb-parquet | q25 | 0.002225 | 0.003934 | 0.000928 | 0.020986 | 0.015434 | 11.34 |
| 10k | rudb-parquet | q26 | 0.002791 | 0.003659 | 0.000685 | 0.030328 | 0.016406 | 11.66 |
| 10k | rudb-parquet | q27 | 0.012392 | 0.002955 | 0.000338 | 0.018178 | 0.013339 | 11.89 |
| 10k | rudb-parquet | q28 | 0.004091 | 0.004330 | 0.000626 | 0.013859 | 0.011940 | 13.00 |
| 10k | rudb-parquet | q29 | 0.006545 | 0.009651 | 0.001708 | 0.022585 | 0.019810 | 15.03 |
| 10k | rudb-parquet | q30 | 0.009610 | 0.007751 | 0.001409 | 0.020264 | 0.017444 | 12.80 |
| 10k | rudb-parquet | q31 | 0.003577 | 0.007770 | 0.000282 | 0.035949 | 0.019328 | 12.94 |
| 10k | rudb-parquet | q32 | 0.014935 | 0.006240 | 0.003260 | 0.021577 | 0.017445 | 12.94 |
| 10k | rudb-parquet | q33 | 0.003841 | 0.012881 | 0.007425 | 0.080411 | 0.020485 | 13.00 |
| 10k | rudb-parquet | q34 | 0.093372 | 0.009808 | 0.000402 | 0.038065 | 0.022604 | 15.77 |
| 10k | rudb-parquet | q35 | 0.010413 | 0.010673 | 0.001416 | 0.044180 | 0.024242 | 15.34 |
| 10k | rudb-parquet | q36 | 0.007831 | 0.008579 | 0.133612 | 0.035863 | 0.018645 | 12.34 |
| 10k | rudb-parquet | q37 | 0.006643 | 0.007749 | 0.000892 | 0.034030 | 0.020498 | 13.83 |
| 10k | rudb-parquet | q38 | 0.010999 | 0.010407 | 0.004369 | 0.031613 | 0.020435 | 13.97 |
| 10k | rudb-parquet | q39 | 0.006389 | 0.005927 | 0.001810 | 0.038694 | 0.019968 | 13.95 |
| 10k | rudb-parquet | q40 | 0.014089 | 0.011907 | 0.000698 | 0.037972 | 0.024162 | 15.88 |
| 10k | rudb-parquet | q41 | 0.016724 | 0.008055 | 0.001116 | 0.033772 | 0.021236 | 12.70 |
| 10k | rudb-parquet | q42 | 0.021414 | 0.004049 | 0.000852 | 0.029286 | 0.018433 | 12.31 |
| 10k | rudb-parquet | q43 | 0.002047 | 0.002341 | 0.000166 | 0.012921 | 0.010269 | 11.02 |
| 1m | duckdb-native | q1 | 0.001000 | 0.001000 | 0.001000 | 0.034849 | 0.030226 | 20.81 |
| 1m | duckdb-native | q2 | 0.002000 | 0.002000 | 0.000000 | 0.033207 | 0.031757 | 23.48 |
| 1m | duckdb-native | q3 | 0.003000 | 0.003000 | 0.001000 | 0.034711 | 0.034600 | 26.08 |
| 1m | duckdb-native | q4 | 0.008000 | 0.003000 | 0.003000 | 0.042517 | 0.039503 | 29.94 |
| 1m | duckdb-native | q5 | 0.020000 | 0.023000 | 0.005000 | 0.058064 | 0.109359 | 66.17 |
| 1m | duckdb-native | q6 | 0.015000 | 0.014000 | 0.001000 | 0.044846 | 0.081502 | 53.25 |
| 1m | duckdb-native | q7 | 0.001000 | 0.001000 | 0.000000 | 0.030130 | 0.027353 | 20.47 |
| 1m | duckdb-native | q8 | 0.003000 | 0.002000 | 0.000000 | 0.031812 | 0.031706 | 24.64 |
| 1m | duckdb-native | q9 | 0.031000 | 0.035000 | 0.013000 | 0.073206 | 0.139923 | 77.28 |
| 1m | duckdb-native | q10 | 0.030000 | 0.035000 | 0.007000 | 0.070349 | 0.165723 | 87.28 |
| 1m | duckdb-native | q11 | 0.007000 | 0.009000 | 0.002000 | 0.041217 | 0.058208 | 50.50 |
| 1m | duckdb-native | q12 | 0.024000 | 0.011000 | 0.002000 | 0.044102 | 0.061571 | 52.58 |
| 1m | duckdb-native | q13 | 0.032000 | 0.014000 | 0.003000 | 0.055030 | 0.080381 | 55.91 |
| 1m | duckdb-native | q14 | 0.053000 | 0.028000 | 0.008000 | 0.065170 | 0.110342 | 85.69 |
| 1m | duckdb-native | q15 | 0.013000 | 0.015000 | 0.003000 | 0.047998 | 0.082427 | 60.91 |
| 1m | duckdb-native | q16 | 0.026000 | 0.030000 | 0.016000 | 0.065121 | 0.129011 | 86.19 |
| 1m | duckdb-native | q17 | 0.044000 | 0.064000 | 0.010000 | 0.114746 | 0.224157 | 155.50 |
| 1m | duckdb-native | q18 | 0.036000 | 0.038000 | 0.002000 | 0.072447 | 0.171585 | 135.14 |
| 1m | duckdb-native | q19 | 0.048000 | 0.052000 | 0.001000 | 0.092338 | 0.265793 | 172.48 |
| 1m | duckdb-native | q20 | 0.002000 | 0.003000 | 0.001000 | 0.034221 | 0.034649 | 29.17 |
| 1m | duckdb-native | q21 | 0.050000 | 0.054000 | 0.016000 | 0.092863 | 0.167007 | 89.44 |
| 1m | duckdb-native | q22 | 0.040000 | 0.028000 | 0.000000 | 0.065943 | 0.146865 | 102.22 |
| 1m | duckdb-native | q23 | 0.068000 | 0.035000 | 0.017000 | 0.077028 | 0.136192 | 122.89 |
| 1m | duckdb-native | q24 | 0.106000 | 0.072000 | 0.020000 | 0.119463 | 0.227782 | 226.70 |
| 1m | duckdb-native | q25 | 0.008000 | 0.007000 | 0.005000 | 0.039544 | 0.052683 | 37.81 |
| 1m | duckdb-native | q26 | 0.008000 | 0.007000 | 0.000000 | 0.041317 | 0.056588 | 30.44 |
| 1m | duckdb-native | q27 | 0.006000 | 0.006000 | 0.001000 | 0.041123 | 0.054149 | 34.59 |
| 1m | duckdb-native | q28 | 0.043000 | 0.031000 | 0.005000 | 0.069103 | 0.152056 | 99.00 |
| 1m | duckdb-native | q29 | 0.521000 | 0.571000 | 0.104000 | 0.607306 | 1.248482 | 139.89 |
| 1m | duckdb-native | q30 | 0.009000 | 0.011000 | 0.001000 | 0.043034 | 0.044866 | 34.59 |
| 1m | duckdb-native | q31 | 0.019000 | 0.015000 | 0.001000 | 0.048655 | 0.089681 | 65.89 |
| 1m | duckdb-native | q32 | 0.019000 | 0.015000 | 0.003000 | 0.049947 | 0.088785 | 71.03 |
| 1m | duckdb-native | q33 | 0.041000 | 0.051000 | 0.006000 | 0.091642 | 0.224162 | 133.55 |
| 1m | duckdb-native | q34 | 0.070000 | 0.062000 | 0.033000 | 0.106816 | 0.308952 | 259.69 |
| 1m | duckdb-native | q35 | 0.076000 | 0.067000 | 0.006000 | 0.108037 | 0.329739 | 233.83 |
| 1m | duckdb-native | q36 | 0.031000 | 0.027000 | 0.003000 | 0.064017 | 0.145813 | 78.16 |
| 1m | duckdb-native | q37 | 0.004000 | 0.005000 | 0.001000 | 0.037688 | 0.040389 | 32.52 |
| 1m | duckdb-native | q38 | 0.004000 | 0.004000 | 0.004000 | 0.039773 | 0.039336 | 30.94 |
| 1m | duckdb-native | q39 | 0.004000 | 0.005000 | 0.004000 | 0.053623 | 0.042208 | 30.78 |
| 1m | duckdb-native | q40 | 0.007000 | 0.007000 | 0.002000 | 0.040435 | 0.045419 | 39.64 |
| 1m | duckdb-native | q41 | 0.004000 | 0.004000 | 0.000000 | 0.037487 | 0.036068 | 31.02 |
| 1m | duckdb-native | q42 | 0.013000 | 0.003000 | 0.001000 | 0.037226 | 0.038828 | 30.27 |
| 1m | duckdb-native | q43 | 0.003000 | 0.004000 | 0.000000 | 0.038002 | 0.039598 | 29.55 |
| 1m | rudb-native | q1 | 0.003521 | 0.001019 | 0.000155 | 0.012257 | 0.010001 | 10.50 |
| 1m | rudb-native | q2 | 0.001133 | 0.001249 | 0.000245 | 0.012862 | 0.009912 | 10.91 |
| 1m | rudb-native | q3 | 0.001136 | 0.001259 | 0.000158 | 0.012147 | 0.010970 | 11.02 |
| 1m | rudb-native | q4 | 0.000958 | 0.001137 | 0.000071 | 0.014146 | 0.011060 | 10.72 |
| 1m | rudb-native | q5 | 0.001988 | 0.001066 | 0.000164 | 0.014095 | 0.010824 | 11.20 |
| 1m | rudb-native | q6 | 0.000953 | 0.000936 | 0.000212 | 0.011306 | 0.010009 | 11.34 |
| 1m | rudb-native | q7 | 0.000920 | 0.000999 | 0.000037 | 0.010716 | 0.009179 | 10.89 |
| 1m | rudb-native | q8 | 0.003639 | 0.002079 | 0.000155 | 0.013298 | 0.012174 | 14.44 |
| 1m | rudb-native | q9 | 0.012475 | 0.013560 | 0.004230 | 0.027160 | 0.046734 | 48.12 |
| 1m | rudb-native | q10 | 0.010671 | 0.012340 | 0.001418 | 0.023988 | 0.059003 | 52.59 |
| 1m | rudb-native | q11 | 0.003767 | 0.003675 | 0.000225 | 0.015013 | 0.019861 | 25.06 |
| 1m | rudb-native | q12 | 0.004861 | 0.004868 | 0.001086 | 0.017118 | 0.022447 | 25.95 |
| 1m | rudb-native | q13 | 0.004657 | 0.004034 | 0.000194 | 0.016657 | 0.018518 | 17.86 |
| 1m | rudb-native | q14 | 0.007576 | 0.008545 | 0.005179 | 0.027259 | 0.031458 | 32.44 |
| 1m | rudb-native | q15 | 0.010954 | 0.007468 | 0.000792 | 0.019689 | 0.031185 | 26.91 |
| 1m | rudb-native | q16 | 0.001139 | 0.001256 | 0.000105 | 0.012408 | 0.009727 | 11.27 |
| 1m | rudb-native | q17 | 0.012763 | 0.013288 | 0.002651 | 0.027878 | 0.059801 | 53.06 |
| 1m | rudb-native | q18 | 0.003012 | 0.002858 | 0.000306 | 0.015129 | 0.011936 | 13.69 |
| 1m | rudb-native | q19 | 0.015378 | 0.015573 | 0.002167 | 0.029267 | 0.067780 | 58.06 |
| 1m | rudb-native | q20 | 0.001876 | 0.001520 | 0.000126 | 0.014706 | 0.013594 | 13.80 |
| 1m | rudb-native | q21 | 0.022367 | 0.024059 | 0.003072 | 0.038959 | 0.081067 | 46.30 |
| 1m | rudb-native | q22 | 0.037513 | 0.026052 | 0.001315 | 0.038581 | 0.090353 | 51.39 |
| 1m | rudb-native | q23 | 0.023969 | 0.025247 | 0.006263 | 0.041400 | 0.067735 | 44.83 |
| 1m | rudb-native | q24 | 0.031672 | 0.030922 | 0.002437 | 0.046300 | 0.098236 | 68.09 |
| 1m | rudb-native | q25 | 0.003534 | 0.004024 | 0.001169 | 0.016167 | 0.016821 | 16.66 |
| 1m | rudb-native | q26 | 0.004458 | 0.004976 | 0.000225 | 0.018677 | 0.023405 | 18.38 |
| 1m | rudb-native | q27 | 0.003956 | 0.003986 | 0.000615 | 0.017988 | 0.017599 | 17.23 |
| 1m | rudb-native | q28 | 0.010472 | 0.007768 | 0.000818 | 0.021326 | 0.028297 | 31.41 |
| 1m | rudb-native | q29 | 0.093875 | 0.069688 | 0.003500 | 0.086303 | 0.271132 | 82.20 |
| 1m | rudb-native | q30 | 0.006742 | 0.005691 | 0.000391 | 0.018182 | 0.015656 | 12.78 |
| 1m | rudb-native | q31 | 0.006431 | 0.006319 | 0.002057 | 0.020660 | 0.029787 | 29.95 |
| 1m | rudb-native | q32 | 0.002178 | 0.001914 | 0.000025 | 0.014292 | 0.012420 | 12.00 |
| 1m | rudb-native | q33 | 0.001568 | 0.001499 | 0.000068 | 0.014261 | 0.011745 | 10.77 |
| 1m | rudb-native | q34 | 0.001211 | 0.001531 | 0.000047 | 0.013877 | 0.011781 | 10.72 |
| 1m | rudb-native | q35 | 0.006001 | 0.005771 | 0.000416 | 0.020065 | 0.024205 | 24.81 |
| 1m | rudb-native | q36 | 0.002464 | 0.001628 | 0.000089 | 0.014661 | 0.011192 | 11.52 |
| 1m | rudb-native | q37 | 0.004912 | 0.004210 | 0.000782 | 0.019097 | 0.015908 | 17.00 |
| 1m | rudb-native | q38 | 0.003416 | 0.004171 | 0.000897 | 0.016963 | 0.014612 | 16.47 |
| 1m | rudb-native | q39 | 0.002631 | 0.005958 | 0.003355 | 0.021346 | 0.014982 | 15.77 |
| 1m | rudb-native | q40 | 0.005671 | 0.005987 | 0.000247 | 0.019420 | 0.018523 | 20.16 |
| 1m | rudb-native | q41 | 0.003109 | 0.002850 | 0.000515 | 0.014868 | 0.014561 | 15.59 |
| 1m | rudb-native | q42 | 0.002833 | 0.002728 | 0.000099 | 0.016609 | 0.015680 | 15.33 |
| 1m | rudb-native | q43 | 0.002972 | 0.002910 | 0.000248 | 0.016705 | 0.015866 | 15.48 |
| 1m | duckdb-parquet | q1 | 0.003000 | 0.004000 | 0.000000 | 0.039237 | 0.033794 | 21.81 |
| 1m | duckdb-parquet | q2 | 0.008000 | 0.005000 | 0.001000 | 0.038031 | 0.037565 | 22.44 |
| 1m | duckdb-parquet | q3 | 0.005000 | 0.009000 | 0.003000 | 0.041909 | 0.042940 | 24.64 |
| 1m | duckdb-parquet | q4 | 0.008000 | 0.007000 | 0.001000 | 0.042180 | 0.042657 | 36.73 |
| 1m | duckdb-parquet | q5 | 0.030000 | 0.023000 | 0.007000 | 0.066954 | 0.111222 | 69.42 |
| 1m | duckdb-parquet | q6 | 0.020000 | 0.017000 | 0.003000 | 0.050646 | 0.084270 | 58.44 |
| 1m | duckdb-parquet | q7 | 0.003000 | 0.003000 | 0.001000 | 0.032503 | 0.030200 | 22.12 |
| 1m | duckdb-parquet | q8 | 0.005000 | 0.005000 | 0.001000 | 0.035353 | 0.036180 | 23.52 |
| 1m | duckdb-parquet | q9 | 0.030000 | 0.033000 | 0.012000 | 0.065883 | 0.142446 | 80.30 |
| 1m | duckdb-parquet | q10 | 0.046000 | 0.035000 | 0.005000 | 0.070466 | 0.168859 | 85.19 |
| 1m | duckdb-parquet | q11 | 0.009000 | 0.014000 | 0.005000 | 0.046292 | 0.059976 | 54.41 |
| 1m | duckdb-parquet | q12 | 0.012000 | 0.014000 | 0.004000 | 0.048059 | 0.065414 | 55.50 |
| 1m | duckdb-parquet | q13 | 0.021000 | 0.021000 | 0.002000 | 0.058043 | 0.097402 | 62.70 |
| 1m | duckdb-parquet | q14 | 0.034000 | 0.028000 | 0.004000 | 0.062228 | 0.121644 | 86.81 |
| 1m | duckdb-parquet | q15 | 0.021000 | 0.029000 | 0.013000 | 0.068682 | 0.100327 | 65.56 |
| 1m | duckdb-parquet | q16 | 0.024000 | 0.032000 | 0.015000 | 0.071183 | 0.140931 | 90.81 |
| 1m | duckdb-parquet | q17 | 0.098000 | 0.056000 | 0.016000 | 0.092155 | 0.245014 | 159.91 |
| 1m | duckdb-parquet | q18 | 0.045000 | 0.047000 | 0.006000 | 0.085576 | 0.187143 | 139.50 |
| 1m | duckdb-parquet | q19 | 0.056000 | 0.058000 | 0.002000 | 0.096946 | 0.282424 | 176.64 |
| 1m | duckdb-parquet | q20 | 0.005000 | 0.006000 | 0.000000 | 0.039099 | 0.043205 | 37.11 |
| 1m | duckdb-parquet | q21 | 0.047000 | 0.051000 | 0.020000 | 0.090213 | 0.201243 | 141.52 |
| 1m | duckdb-parquet | q22 | 0.048000 | 0.047000 | 0.008000 | 0.086108 | 0.196057 | 168.45 |
| 1m | duckdb-parquet | q23 | 0.340000 | 0.221000 | 0.054000 | 0.293451 | 0.371185 | 304.33 |
| 1m | duckdb-parquet | q24 | 0.319000 | 0.386000 | 0.086000 | 0.449708 | 0.552169 | 385.80 |
| 1m | duckdb-parquet | q25 | 0.027000 | 0.029000 | 0.009000 | 0.062625 | 0.114879 | 61.95 |
| 1m | duckdb-parquet | q26 | 0.021000 | 0.013000 | 0.004000 | 0.050373 | 0.075313 | 38.64 |
| 1m | duckdb-parquet | q27 | 0.015000 | 0.017000 | 0.006000 | 0.051247 | 0.089001 | 49.11 |
| 1m | duckdb-parquet | q28 | 0.036000 | 0.047000 | 0.032000 | 0.085318 | 0.186755 | 159.81 |
| 1m | duckdb-parquet | q29 | 1.123000 | 0.593000 | 0.045000 | 0.634676 | 1.293505 | 186.12 |
| 1m | duckdb-parquet | q30 | 0.015000 | 0.013000 | 0.002000 | 0.046061 | 0.048792 | 34.83 |
| 1m | duckdb-parquet | q31 | 0.020000 | 0.022000 | 0.001000 | 0.056027 | 0.109041 | 62.41 |
| 1m | duckdb-parquet | q32 | 0.024000 | 0.020000 | 0.001000 | 0.056273 | 0.108671 | 72.08 |
| 1m | duckdb-parquet | q33 | 0.053000 | 0.044000 | 0.004000 | 0.081024 | 0.225169 | 126.48 |
| 1m | duckdb-parquet | q34 | 0.104000 | 0.084000 | 0.003000 | 0.126419 | 0.360912 | 274.05 |
| 1m | duckdb-parquet | q35 | 0.094000 | 0.088000 | 0.003000 | 0.129992 | 0.383895 | 279.28 |
| 1m | duckdb-parquet | q36 | 0.026000 | 0.029000 | 0.002000 | 0.063871 | 0.151507 | 81.89 |
| 1m | duckdb-parquet | q37 | 0.022000 | 0.025000 | 0.001000 | 0.063338 | 0.082645 | 68.11 |
| 1m | duckdb-parquet | q38 | 0.019000 | 0.023000 | 0.002000 | 0.061129 | 0.071806 | 62.84 |
| 1m | duckdb-parquet | q39 | 0.023000 | 0.026000 | 0.004000 | 0.064423 | 0.084101 | 67.12 |
| 1m | duckdb-parquet | q40 | 0.027000 | 0.030000 | 0.003000 | 0.065409 | 0.105542 | 102.09 |
| 1m | duckdb-parquet | q41 | 0.007000 | 0.008000 | 0.000000 | 0.041103 | 0.045092 | 35.75 |
| 1m | duckdb-parquet | q42 | 0.007000 | 0.007000 | 0.001000 | 0.042478 | 0.045080 | 32.75 |
| 1m | duckdb-parquet | q43 | 0.006000 | 0.008000 | 0.001000 | 0.042127 | 0.045719 | 31.34 |
| 1m | rudb-parquet | q1 | 0.005291 | 0.003186 | 0.000234 | 0.013424 | 0.011600 | 12.42 |
| 1m | rudb-parquet | q2 | 0.007930 | 0.004212 | 0.000254 | 0.014663 | 0.016927 | 17.47 |
| 1m | rudb-parquet | q3 | 0.005497 | 0.007106 | 0.002429 | 0.017875 | 0.019520 | 21.91 |
| 1m | rudb-parquet | q4 | 0.007862 | 0.006459 | 0.001067 | 0.016715 | 0.018738 | 34.59 |
| 1m | rudb-parquet | q5 | 0.012811 | 0.014873 | 0.005617 | 0.028374 | 0.056203 | 46.94 |
| 1m | rudb-parquet | q6 | 0.026499 | 0.028310 | 0.005718 | 0.038912 | 0.100563 | 53.69 |
| 1m | rudb-parquet | q7 | 0.002578 | 0.001670 | 0.000590 | 0.010235 | 0.008584 | 10.62 |
| 1m | rudb-parquet | q8 | 0.005733 | 0.004493 | 0.000772 | 0.013613 | 0.016894 | 17.58 |
| 1m | rudb-parquet | q9 | 0.011636 | 0.020682 | 0.011856 | 0.035623 | 0.052211 | 59.53 |
| 1m | rudb-parquet | q10 | 0.016319 | 0.018347 | 0.002384 | 0.028487 | 0.073360 | 67.55 |
| 1m | rudb-parquet | q11 | 0.009933 | 0.012444 | 0.002670 | 0.023767 | 0.040707 | 42.00 |
| 1m | rudb-parquet | q12 | 0.012544 | 0.011810 | 0.006640 | 0.021931 | 0.043399 | 43.30 |
| 1m | rudb-parquet | q13 | 0.024430 | 0.024011 | 0.001328 | 0.035042 | 0.090658 | 51.67 |
| 1m | rudb-parquet | q14 | 0.053906 | 0.034925 | 0.006902 | 0.046952 | 0.117885 | 80.69 |
| 1m | rudb-parquet | q15 | 0.028211 | 0.025008 | 0.002882 | 0.035557 | 0.095885 | 54.88 |
| 1m | rudb-parquet | q16 | 0.008860 | 0.010096 | 0.003768 | 0.021931 | 0.038771 | 50.78 |
| 1m | rudb-parquet | q17 | 0.059597 | 0.056191 | 0.025141 | 0.070397 | 0.224748 | 123.59 |
| 1m | rudb-parquet | q18 | 0.018296 | 0.016564 | 0.000443 | 0.029327 | 0.070660 | 50.34 |
| 1m | rudb-parquet | q19 | 0.068263 | 0.069242 | 0.007174 | 0.085148 | 0.266867 | 157.11 |
| 1m | rudb-parquet | q20 | 0.004320 | 0.004226 | 0.000410 | 0.015783 | 0.019796 | 33.11 |
| 1m | rudb-parquet | q21 | 0.073909 | 0.081296 | 0.011283 | 0.100560 | 0.234725 | 170.19 |
| 1m | rudb-parquet | q22 | 0.069754 | 0.082468 | 0.019267 | 0.097107 | 0.262625 | 176.02 |
| 1m | rudb-parquet | q23 | 0.440438 | 0.544814 | 0.143431 | 0.569263 | 0.558252 | 290.06 |
| 1m | rudb-parquet | q24 | 0.372011 | 0.294246 | 0.142430 | 0.318428 | 0.728714 | 333.89 |
| 1m | rudb-parquet | q25 | 0.018894 | 0.017491 | 0.001144 | 0.029350 | 0.069854 | 48.78 |
| 1m | rudb-parquet | q26 | 0.012764 | 0.012762 | 0.000213 | 0.024185 | 0.055843 | 31.05 |
| 1m | rudb-parquet | q27 | 0.019818 | 0.019968 | 0.001166 | 0.032356 | 0.071938 | 50.22 |
| 1m | rudb-parquet | q28 | 0.052288 | 0.062498 | 0.013584 | 0.078939 | 0.224963 | 176.55 |
| 1m | rudb-parquet | q29 | 0.222704 | 0.173385 | 0.052101 | 0.193189 | 0.459264 | 197.25 |
| 1m | rudb-parquet | q30 | 0.012233 | 0.008572 | 0.000313 | 0.019926 | 0.022876 | 19.25 |
| 1m | rudb-parquet | q31 | 0.019686 | 0.019050 | 0.001962 | 0.032687 | 0.078234 | 57.86 |
| 1m | rudb-parquet | q32 | 0.026514 | 0.021894 | 0.002522 | 0.034982 | 0.082972 | 68.23 |
| 1m | rudb-parquet | q33 | 0.019521 | 0.019471 | 0.001624 | 0.033359 | 0.081897 | 70.50 |
| 1m | rudb-parquet | q34 | 0.119956 | 0.118302 | 0.009946 | 0.137215 | 0.424104 | 269.08 |
| 1m | rudb-parquet | q35 | 0.112324 | 0.111551 | 0.006831 | 0.130988 | 0.441242 | 269.25 |
| 1m | rudb-parquet | q36 | 0.008955 | 0.010177 | 0.002484 | 0.022059 | 0.039428 | 39.73 |
| 1m | rudb-parquet | q37 | 0.037893 | 0.050925 | 0.012848 | 0.065309 | 0.074476 | 65.20 |
| 1m | rudb-parquet | q38 | 0.052497 | 0.059795 | 0.005814 | 0.072088 | 0.100278 | 59.69 |
| 1m | rudb-parquet | q39 | 0.038471 | 0.040149 | 0.015234 | 0.054829 | 0.074509 | 64.88 |
| 1m | rudb-parquet | q40 | 0.075980 | 0.066761 | 0.006539 | 0.083841 | 0.126749 | 110.73 |
| 1m | rudb-parquet | q41 | 0.007091 | 0.008007 | 0.001306 | 0.019208 | 0.020296 | 28.66 |
| 1m | rudb-parquet | q42 | 0.006947 | 0.006631 | 0.000734 | 0.018028 | 0.019638 | 26.20 |
| 1m | rudb-parquet | q43 | 0.006683 | 0.007725 | 0.001842 | 0.018131 | 0.019984 | 24.36 |
| 10m | duckdb-native | q1 | 0.004000 | 0.003000 | 0.000000 | 0.035880 | 0.030157 | 21.30 |
| 10m | duckdb-native | q2 | 0.018000 | 0.007000 | 0.001000 | 0.040450 | 0.041734 | 28.42 |
| 10m | duckdb-native | q3 | 0.020000 | 0.012000 | 0.001000 | 0.037379 | 0.076801 | 36.44 |
| 10m | duckdb-native | q4 | 0.023000 | 0.015000 | 0.005000 | 0.047156 | 0.078809 | 42.11 |
| 10m | duckdb-native | q5 | 0.051000 | 0.054000 | 0.003000 | 0.086565 | 0.288159 | 104.64 |
| 10m | duckdb-native | q6 | 0.120000 | 0.110000 | 0.011000 | 0.148852 | 0.546645 | 243.27 |
| 10m | duckdb-native | q7 | 0.005000 | 0.003000 | 0.000000 | 0.028912 | 0.026366 | 22.62 |
| 10m | duckdb-native | q8 | 0.010000 | 0.006000 | 0.000000 | 0.030299 | 0.038106 | 29.75 |
| 10m | duckdb-native | q9 | 0.070000 | 0.067000 | 0.004000 | 0.096338 | 0.353661 | 127.34 |
| 10m | duckdb-native | q10 | 0.124000 | 0.123000 | 0.011000 | 0.158656 | 0.639656 | 144.72 |
| 10m | duckdb-native | q11 | 0.037000 | 0.023000 | 0.001000 | 0.054871 | 0.139000 | 72.98 |
| 10m | duckdb-native | q12 | 0.033000 | 0.028000 | 0.003000 | 0.062728 | 0.153235 | 79.55 |
| 10m | duckdb-native | q13 | 0.090000 | 0.085000 | 0.006000 | 0.120666 | 0.454742 | 251.52 |
| 10m | duckdb-native | q14 | 0.143000 | 0.145000 | 0.018000 | 0.192287 | 0.743684 | 366.56 |
| 10m | duckdb-native | q15 | 0.125000 | 0.091000 | 0.004000 | 0.124526 | 0.489711 | 263.88 |
| 10m | duckdb-native | q16 | 0.069000 | 0.073000 | 0.006000 | 0.104451 | 0.364933 | 122.66 |
| 10m | duckdb-native | q17 | 0.170000 | 0.161000 | 0.002000 | 0.201635 | 0.818987 | 336.45 |
| 10m | duckdb-native | q18 | 0.126000 | 0.133000 | 0.008000 | 0.176957 | 0.660656 | 302.33 |
| 10m | duckdb-native | q19 | 0.311000 | 0.293000 | 0.021000 | 0.356200 | 1.494517 | 696.22 |
| 10m | duckdb-native | q20 | 0.004000 | 0.004000 | 0.000000 | 0.029058 | 0.033401 | 34.12 |
| 10m | duckdb-native | q21 | 0.268000 | 0.161000 | 0.001000 | 0.233199 | 0.827409 | 450.61 |
| 10m | duckdb-native | q22 | 0.130000 | 0.131000 | 0.010000 | 0.223565 | 0.688998 | 501.31 |
| 10m | duckdb-native | q23 | 0.419000 | 0.233000 | 0.136000 | 0.365346 | 1.175326 | 840.62 |
| 10m | duckdb-native | q24 | 0.187000 | 0.146000 | 0.020000 | 0.218645 | 0.706201 | 510.48 |
| 10m | duckdb-native | q25 | 0.011000 | 0.010000 | 0.001000 | 0.035271 | 0.060467 | 40.84 |
| 10m | duckdb-native | q26 | 0.039000 | 0.040000 | 0.005000 | 0.066658 | 0.219993 | 75.72 |
| 10m | duckdb-native | q27 | 0.010000 | 0.009000 | 0.000000 | 0.031603 | 0.054155 | 39.72 |
| 10m | duckdb-native | q28 | 0.112000 | 0.115000 | 0.007000 | 0.205356 | 0.594364 | 459.39 |
| 10m | duckdb-native | q29 | 3.188000 | 3.226000 | 0.548000 | 3.329909 | 10.018586 | 672.73 |
| 10m | duckdb-native | q30 | 0.023000 | 0.021000 | 0.005000 | 0.051785 | 0.079136 | 41.78 |
| 10m | duckdb-native | q31 | 0.100000 | 0.079000 | 0.012000 | 0.116259 | 0.430279 | 193.98 |
| 10m | duckdb-native | q32 | 0.151000 | 0.154000 | 0.039000 | 0.210687 | 0.586558 | 315.97 |
| 10m | duckdb-native | q33 | 0.626000 | 0.668000 | 0.034000 | 0.787439 | 1.777971 | 830.45 |
| 10m | duckdb-native | q34 | 0.676000 | 0.474000 | 0.263000 | 0.650448 | 1.770093 | 1119.42 |
| 10m | duckdb-native | q35 | 0.439000 | 0.452000 | 0.049000 | 0.602421 | 2.025498 | 1155.86 |
| 10m | duckdb-native | q36 | 0.073000 | 0.083000 | 0.006000 | 0.112948 | 0.452551 | 114.69 |
| 10m | duckdb-native | q37 | 0.032000 | 0.032000 | 0.011000 | 0.064690 | 0.156756 | 129.36 |
| 10m | duckdb-native | q38 | 0.012000 | 0.009000 | 0.002000 | 0.035484 | 0.054903 | 48.97 |
| 10m | duckdb-native | q39 | 0.014000 | 0.013000 | 0.001000 | 0.039846 | 0.068764 | 71.23 |
| 10m | duckdb-native | q40 | 0.082000 | 0.061000 | 0.011000 | 0.093577 | 0.286092 | 237.70 |
| 10m | duckdb-native | q41 | 0.018000 | 0.007000 | 0.000000 | 0.029903 | 0.044437 | 47.70 |
| 10m | duckdb-native | q42 | 0.010000 | 0.006000 | 0.002000 | 0.029891 | 0.042382 | 40.98 |
| 10m | duckdb-native | q43 | 0.012000 | 0.010000 | 0.000000 | 0.036293 | 0.058654 | 37.41 |
| 10m | rudb-native | q1 | 0.001897 | 0.001217 | 0.001078 | 0.016116 | 0.012562 | 12.83 |
| 10m | rudb-native | q2 | 0.001102 | 0.001300 | 0.000077 | 0.014824 | 0.013418 | 13.45 |
| 10m | rudb-native | q3 | 0.001211 | 0.000910 | 0.000053 | 0.012529 | 0.010709 | 13.27 |
| 10m | rudb-native | q4 | 0.000922 | 0.001010 | 0.000307 | 0.014366 | 0.011431 | 13.19 |
| 10m | rudb-native | q5 | 0.001154 | 0.000798 | 0.000026 | 0.011192 | 0.009902 | 12.48 |
| 10m | rudb-native | q6 | 0.000966 | 0.000921 | 0.000171 | 0.012932 | 0.011528 | 13.11 |
| 10m | rudb-native | q7 | 0.000857 | 0.000893 | 0.000073 | 0.011897 | 0.010288 | 12.73 |
| 10m | rudb-native | q8 | 0.005273 | 0.003699 | 0.000326 | 0.014710 | 0.023276 | 20.48 |
| 10m | rudb-native | q9 | 0.023785 | 0.024383 | 0.001088 | 0.036104 | 0.123521 | 73.97 |
| 10m | rudb-native | q10 | 0.031561 | 0.032449 | 0.005344 | 0.045539 | 0.170969 | 82.83 |
| 10m | rudb-native | q11 | 0.008967 | 0.009182 | 0.000699 | 0.023510 | 0.053091 | 36.98 |
| 10m | rudb-native | q12 | 0.016648 | 0.012061 | 0.002321 | 0.026250 | 0.060882 | 37.66 |
| 10m | rudb-native | q13 | 0.010866 | 0.010915 | 0.001045 | 0.022776 | 0.053450 | 38.50 |
| 10m | rudb-native | q14 | 0.030635 | 0.029436 | 0.000879 | 0.045902 | 0.146515 | 81.16 |
| 10m | rudb-native | q15 | 0.023214 | 0.022568 | 0.000498 | 0.037053 | 0.119946 | 81.16 |
| 10m | rudb-native | q16 | 0.001231 | 0.001035 | 0.000141 | 0.011556 | 0.010257 | 13.62 |
| 10m | rudb-native | q17 | 0.043901 | 0.038615 | 0.001060 | 0.051879 | 0.209498 | 125.17 |
| 10m | rudb-native | q18 | 0.006527 | 0.004082 | 0.000046 | 0.015493 | 0.014030 | 22.92 |
| 10m | rudb-native | q19 | 0.075795 | 0.076615 | 0.005482 | 0.092412 | 0.406113 | 220.78 |
| 10m | rudb-native | q20 | 0.001522 | 0.001284 | 0.000115 | 0.011779 | 0.012216 | 15.75 |
| 10m | rudb-native | q21 | 0.037337 | 0.041651 | 0.002885 | 0.056721 | 0.204616 | 69.23 |
| 10m | rudb-native | q22 | 0.046861 | 0.055836 | 0.002122 | 0.070336 | 0.225288 | 87.84 |
| 10m | rudb-native | q23 | 0.078790 | 0.086896 | 0.005505 | 0.103114 | 0.346868 | 145.36 |
| 10m | rudb-native | q24 | 0.060444 | 0.049711 | 0.005146 | 0.064265 | 0.193867 | 98.27 |
| 10m | rudb-native | q25 | 0.004295 | 0.003691 | 0.000122 | 0.014510 | 0.016933 | 21.66 |
| 10m | rudb-native | q26 | 0.032314 | 0.011863 | 0.001781 | 0.022883 | 0.051285 | 35.11 |
| 10m | rudb-native | q27 | 0.006366 | 0.006462 | 0.000391 | 0.016185 | 0.018681 | 24.75 |
| 10m | rudb-native | q28 | 0.023951 | 0.026090 | 0.002289 | 0.040467 | 0.103905 | 68.52 |
| 10m | rudb-native | q29 | 0.373569 | 0.478150 | 0.011437 | 0.506269 | 1.735508 | 282.33 |
| 10m | rudb-native | q30 | 0.005085 | 0.004362 | 0.001112 | 0.017327 | 0.014881 | 14.52 |
| 10m | rudb-native | q31 | 0.054881 | 0.032958 | 0.007700 | 0.047037 | 0.171964 | 95.27 |
| 10m | rudb-native | q32 | 0.002358 | 0.002529 | 0.000399 | 0.017442 | 0.015211 | 15.89 |
| 10m | rudb-native | q33 | 0.001355 | 0.001466 | 0.000224 | 0.015963 | 0.013570 | 13.42 |
| 10m | rudb-native | q34 | 0.001784 | 0.001542 | 0.001064 | 0.016839 | 0.012967 | 13.67 |
| 10m | rudb-native | q35 | 0.043223 | 0.029129 | 0.004533 | 0.046069 | 0.118792 | 96.94 |
| 10m | rudb-native | q36 | 0.002224 | 0.001496 | 0.000083 | 0.013443 | 0.011835 | 13.28 |
| 10m | rudb-native | q37 | 0.013896 | 0.010182 | 0.001620 | 0.023322 | 0.032664 | 41.89 |
| 10m | rudb-native | q38 | 0.011623 | 0.006931 | 0.001154 | 0.018470 | 0.028993 | 35.88 |
| 10m | rudb-native | q39 | 0.010528 | 0.005587 | 0.001106 | 0.017099 | 0.020105 | 29.62 |
| 10m | rudb-native | q40 | 0.038689 | 0.036603 | 0.002847 | 0.051025 | 0.153218 | 93.22 |
| 10m | rudb-native | q41 | 0.013547 | 0.006842 | 0.000770 | 0.016842 | 0.026855 | 40.41 |
| 10m | rudb-native | q42 | 0.011589 | 0.005615 | 0.002246 | 0.016171 | 0.023539 | 32.06 |
| 10m | rudb-native | q43 | 0.009540 | 0.006449 | 0.001221 | 0.019355 | 0.028488 | 28.00 |
| 10m | duckdb-parquet | q1 | 0.028000 | 0.028000 | 0.007000 | 0.076228 | 0.068560 | 35.59 |
| 10m | duckdb-parquet | q2 | 0.047000 | 0.040000 | 0.015000 | 0.090086 | 0.106958 | 38.62 |
| 10m | duckdb-parquet | q3 | 0.040000 | 0.036000 | 0.006000 | 0.076403 | 0.117986 | 41.95 |
| 10m | duckdb-parquet | q4 | 0.042000 | 0.038000 | 0.001000 | 0.082581 | 0.119680 | 56.61 |
| 10m | duckdb-parquet | q5 | 0.081000 | 0.058000 | 0.010000 | 0.102266 | 0.249101 | 114.12 |
| 10m | duckdb-parquet | q6 | 0.159000 | 0.139000 | 0.014000 | 0.181003 | 0.601730 | 253.58 |
| 10m | duckdb-parquet | q7 | 0.021000 | 0.022000 | 0.002000 | 0.060013 | 0.054532 | 36.45 |
| 10m | duckdb-parquet | q8 | 0.029000 | 0.028000 | 0.001000 | 0.064415 | 0.091195 | 37.97 |
| 10m | duckdb-parquet | q9 | 0.090000 | 0.088000 | 0.002000 | 0.128218 | 0.402097 | 129.39 |
| 10m | duckdb-parquet | q10 | 0.121000 | 0.134000 | 0.010000 | 0.184156 | 0.628444 | 138.70 |
| 10m | duckdb-parquet | q11 | 0.047000 | 0.053000 | 0.006000 | 0.097798 | 0.179163 | 74.45 |
| 10m | duckdb-parquet | q12 | 0.064000 | 0.055000 | 0.001000 | 0.101089 | 0.208634 | 79.03 |
| 10m | duckdb-parquet | q13 | 0.138000 | 0.143000 | 0.008000 | 0.192710 | 0.628011 | 268.92 |
| 10m | duckdb-parquet | q14 | 0.196000 | 0.201000 | 0.014000 | 0.251481 | 0.907062 | 331.56 |
| 10m | duckdb-parquet | q15 | 0.167000 | 0.148000 | 0.002000 | 0.191231 | 0.684577 | 265.11 |
| 10m | duckdb-parquet | q16 | 0.080000 | 0.070000 | 0.003000 | 0.113359 | 0.325956 | 130.41 |
| 10m | duckdb-parquet | q17 | 0.255000 | 0.215000 | 0.030000 | 0.265302 | 0.989367 | 322.80 |
| 10m | duckdb-parquet | q18 | 0.231000 | 0.197000 | 0.021000 | 0.239683 | 0.826264 | 307.42 |
| 10m | duckdb-parquet | q19 | 0.378000 | 0.360000 | 0.025000 | 0.417323 | 1.769429 | 653.84 |
| 10m | duckdb-parquet | q20 | 0.024000 | 0.021000 | 0.002000 | 0.055628 | 0.065868 | 48.42 |
| 10m | duckdb-parquet | q21 | 0.224000 | 0.246000 | 0.003000 | 0.296162 | 1.202441 | 329.06 |
| 10m | duckdb-parquet | q22 | 0.213000 | 0.260000 | 0.057000 | 0.314090 | 1.064009 | 411.30 |
| 10m | duckdb-parquet | q23 | 0.452000 | 0.431000 | 0.025000 | 0.483717 | 2.139197 | 652.92 |
| 10m | duckdb-parquet | q24 | 0.296000 | 0.296000 | 0.017000 | 0.355306 | 1.357548 | 469.05 |
| 10m | duckdb-parquet | q25 | 0.087000 | 0.080000 | 0.001000 | 0.118116 | 0.311806 | 117.05 |
| 10m | duckdb-parquet | q26 | 0.083000 | 0.086000 | 0.002000 | 0.122559 | 0.375597 | 96.61 |
| 10m | duckdb-parquet | q27 | 0.067000 | 0.065000 | 0.006000 | 0.100825 | 0.280796 | 108.75 |
| 10m | duckdb-parquet | q28 | 0.186000 | 0.201000 | 0.011000 | 0.245022 | 0.977650 | 415.39 |
| 10m | duckdb-parquet | q29 | 3.384000 | 4.212000 | 0.378000 | 4.277911 | 12.832851 | 491.27 |
| 10m | duckdb-parquet | q30 | 0.047000 | 0.040000 | 0.003000 | 0.083178 | 0.108351 | 52.39 |
| 10m | duckdb-parquet | q31 | 0.161000 | 0.149000 | 0.004000 | 0.199799 | 0.651810 | 175.23 |
| 10m | duckdb-parquet | q32 | 0.185000 | 0.205000 | 0.030000 | 0.253278 | 0.750876 | 231.52 |
| 10m | duckdb-parquet | q33 | 0.678000 | 0.645000 | 0.121000 | 0.727459 | 1.768580 | 748.62 |
| 10m | duckdb-parquet | q34 | 0.955000 | 0.634000 | 0.088000 | 0.720114 | 2.295903 | 937.14 |
| 10m | duckdb-parquet | q35 | 0.505000 | 0.562000 | 0.061000 | 0.634299 | 2.346217 | 948.27 |
| 10m | duckdb-parquet | q36 | 0.118000 | 0.113000 | 0.018000 | 0.158185 | 0.484151 | 118.09 |
| 10m | duckdb-parquet | q37 | 0.068000 | 0.066000 | 0.003000 | 0.115178 | 0.248081 | 205.41 |
| 10m | duckdb-parquet | q38 | 0.046000 | 0.031000 | 0.002000 | 0.071002 | 0.102991 | 60.62 |
| 10m | duckdb-parquet | q39 | 0.049000 | 0.044000 | 0.009000 | 0.084011 | 0.141997 | 139.97 |
| 10m | duckdb-parquet | q40 | 0.105000 | 0.096000 | 0.015000 | 0.147039 | 0.403869 | 312.47 |
| 10m | duckdb-parquet | q41 | 0.032000 | 0.026000 | 0.007000 | 0.060965 | 0.082087 | 61.89 |
| 10m | duckdb-parquet | q42 | 0.031000 | 0.026000 | 0.003000 | 0.064144 | 0.085533 | 56.44 |
| 10m | duckdb-parquet | q43 | 0.030000 | 0.034000 | 0.003000 | 0.071915 | 0.101672 | 54.31 |
| 10m | rudb-parquet | q1 | 0.019808 | 0.018053 | 0.004118 | 0.030901 | 0.023702 | 24.27 |
| 10m | rudb-parquet | q2 | 0.057005 | 0.028922 | 0.008902 | 0.042143 | 0.076102 | 32.53 |
| 10m | rudb-parquet | q3 | 0.029866 | 0.024324 | 0.001270 | 0.033032 | 0.070099 | 38.12 |
| 10m | rudb-parquet | q4 | 0.024630 | 0.026479 | 0.001124 | 0.037017 | 0.075150 | 48.58 |
| 10m | rudb-parquet | q5 | 0.076586 | 0.070354 | 0.003321 | 0.081032 | 0.296392 | 151.62 |
| 10m | rudb-parquet | q6 | 0.202229 | 0.211959 | 0.014521 | 0.223239 | 0.863249 | 204.19 |
| 10m | rudb-parquet | q7 | 0.007158 | 0.007497 | 0.001695 | 0.014495 | 0.012940 | 17.64 |
| 10m | rudb-parquet | q8 | 0.021952 | 0.024514 | 0.003016 | 0.033004 | 0.066527 | 31.61 |
| 10m | rudb-parquet | q9 | 0.048979 | 0.047817 | 0.003525 | 0.057839 | 0.199863 | 84.58 |
| 10m | rudb-parquet | q10 | 0.065512 | 0.069936 | 0.010166 | 0.080833 | 0.315642 | 96.12 |
| 10m | rudb-parquet | q11 | 0.061448 | 0.061143 | 0.012998 | 0.071939 | 0.225954 | 58.47 |
| 10m | rudb-parquet | q12 | 0.059715 | 0.061474 | 0.006358 | 0.073812 | 0.251823 | 63.34 |
| 10m | rudb-parquet | q13 | 0.150019 | 0.146457 | 0.013255 | 0.155736 | 0.722215 | 206.02 |
| 10m | rudb-parquet | q14 | 0.207898 | 0.199637 | 0.012059 | 0.213751 | 0.940351 | 289.03 |
| 10m | rudb-parquet | q15 | 0.169043 | 0.173228 | 0.009431 | 0.185756 | 0.804785 | 210.69 |
| 10m | rudb-parquet | q16 | 0.035412 | 0.036964 | 0.004286 | 0.045945 | 0.136213 | 78.69 |
| 10m | rudb-parquet | q17 | 0.295431 | 0.296072 | 0.022445 | 0.315630 | 1.294239 | 366.73 |
| 10m | rudb-parquet | q18 | 0.117162 | 0.118051 | 0.006782 | 0.127446 | 0.555448 | 125.44 |
| 10m | rudb-parquet | q19 | 0.432823 | 0.448019 | 0.059401 | 0.470788 | 2.132219 | 646.38 |
| 10m | rudb-parquet | q20 | 0.017928 | 0.017247 | 0.000885 | 0.024964 | 0.044149 | 45.39 |
| 10m | rudb-parquet | q21 | 0.254580 | 0.309177 | 0.056358 | 0.328368 | 1.420748 | 287.95 |
| 10m | rudb-parquet | q22 | 0.341168 | 0.333329 | 0.025049 | 0.349396 | 1.660843 | 339.62 |
| 10m | rudb-parquet | q23 | 0.972868 | 0.804826 | 0.153362 | 0.826235 | 4.044157 | 468.27 |
| 10m | rudb-parquet | q24 | 0.411041 | 0.397111 | 0.159079 | 0.414716 | 2.004869 | 355.56 |
| 10m | rudb-parquet | q25 | 0.130893 | 0.120715 | 0.006878 | 0.131288 | 0.570334 | 161.11 |
| 10m | rudb-parquet | q26 | 0.089404 | 0.096218 | 0.011026 | 0.106456 | 0.401732 | 135.38 |
| 10m | rudb-parquet | q27 | 0.126568 | 0.117424 | 0.005049 | 0.128054 | 0.541587 | 162.28 |
| 10m | rudb-parquet | q28 | 0.254848 | 0.254254 | 0.006677 | 0.265281 | 1.285928 | 284.20 |
| 10m | rudb-parquet | q29 | 1.143667 | 1.141524 | 0.121156 | 1.174788 | 3.756261 | 734.17 |
| 10m | rudb-parquet | q30 | 0.028494 | 0.027490 | 0.000680 | 0.036675 | 0.059431 | 32.09 |
| 10m | rudb-parquet | q31 | 0.157935 | 0.126834 | 0.001619 | 0.139247 | 0.620228 | 162.05 |
| 10m | rudb-parquet | q32 | 0.162340 | 0.149486 | 0.025076 | 0.160963 | 0.654599 | 173.73 |
| 10m | rudb-parquet | q33 | 0.116094 | 0.122943 | 0.005790 | 0.139497 | 0.561810 | 245.39 |
| 10m | rudb-parquet | q34 | 1.322397 | 0.912144 | 0.227691 | 0.944076 | 3.021762 | 829.09 |
| 10m | rudb-parquet | q35 | 0.852147 | 0.799469 | 0.085413 | 0.830221 | 3.106398 | 808.80 |
| 10m | rudb-parquet | q36 | 0.059044 | 0.033613 | 0.000232 | 0.042870 | 0.133493 | 71.59 |
| 10m | rudb-parquet | q37 | 0.063267 | 0.081847 | 0.006688 | 0.094770 | 0.266802 | 189.38 |
| 10m | rudb-parquet | q38 | 0.030329 | 0.034149 | 0.005528 | 0.042327 | 0.105977 | 61.23 |
| 10m | rudb-parquet | q39 | 0.050170 | 0.059227 | 0.003419 | 0.074277 | 0.162405 | 148.56 |
| 10m | rudb-parquet | q40 | 0.189037 | 0.151501 | 0.012677 | 0.171609 | 0.585970 | 347.06 |
| 10m | rudb-parquet | q41 | 0.022367 | 0.023862 | 0.002759 | 0.031249 | 0.054283 | 64.17 |
| 10m | rudb-parquet | q42 | 0.031155 | 0.022063 | 0.007086 | 0.030633 | 0.049914 | 57.30 |
| 10m | rudb-parquet | q43 | 0.023936 | 0.022363 | 0.002008 | 0.031556 | 0.061982 | 53.78 |
