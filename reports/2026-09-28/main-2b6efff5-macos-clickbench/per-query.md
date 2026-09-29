# ClickBench measurement audit

All 43 SQL queries are attempted on DuckDB native, rudb native, DuckDB Parquet, and rudb Parquet. 7 hot repetitions are required for a complete row. A failed repetition invalidates that query; successful fragments are never averaged into a result.

First execution is not disk-cold: the page cache is not flushed. Each repetition uses a fresh process. Query seconds are the CLI timer, including result rendering; wall and CPU seconds and peak RSS cover the whole child process. CPU and RSS come from wait4 for that child. RSS is the maximum resident set, not allocated bytes or an incremental memory delta. Both native rows open a loaded single-file database. Both Parquet rows query the same source file with native mirroring disabled; metadata-only paths remain available. These are sample results, not official ClickBench scores.

| Size | Engine | Load wall (s) | Load CPU (s) | Load peak RSS (MiB) | Native bytes |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 0.047024 | 0.045532 | 47.27 | 1323008 |
| 1k | rudb-native | 0.116578 | 0.091045 | 34.62 | 1876635 |
| 10k | duckdb-native | 0.126933 | 0.129837 | 65.48 | 4468736 |
| 10k | rudb-native | 0.408833 | 0.457844 | 70.47 | 7732602 |
| 1m | duckdb-native | 5.651165 | 13.309386 | 1106.48 | 452210688 |
| 1m | rudb-native | 3.060578 | 11.813455 | 1438.95 | 246807042 |
| 10m | duckdb-native | 59.336346 | 126.041996 | 2527.08 | 2351443968 |
| 10m | rudb-native | 23.154862 | 92.590243 | 2147.77 | 1468068158 |

| Size | Engine | Complete / 43 | Query median sum (s) | Process wall median sum (s) | CPU median sum (s) | Peak RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 43 | 0.065000 | 0.917632 | 0.907267 | 35.88 |
| 1k | rudb-native | 43 | 0.049511 | 0.300308 | 0.254406 | 11.89 |
| 1k | duckdb-parquet | 43 | 0.084000 | 0.920728 | 0.917962 | 34.05 |
| 1k | rudb-parquet | 43 | 0.056517 | 0.302427 | 0.255200 | 11.61 |
| 10k | duckdb-native | 43 | 0.078000 | 0.865927 | 0.871849 | 38.27 |
| 10k | rudb-native | 43 | 0.055889 | 0.304752 | 0.263985 | 14.25 |
| 10k | duckdb-parquet | 43 | 0.109000 | 0.879320 | 0.894707 | 39.14 |
| 10k | rudb-parquet | 43 | 0.082725 | 0.313851 | 0.267894 | 14.94 |
| 1m | duckdb-native | 43 | 0.841000 | 1.884610 | 4.669835 | 232.33 |
| 1m | rudb-native | 43 | 0.313489 | 0.637281 | 1.157284 | 83.33 |
| 1m | duckdb-parquet | 43 | 1.188000 | 2.236635 | 5.771636 | 399.03 |
| 1m | rudb-parquet | 43 | 1.175238 | 1.529730 | 4.813758 | 334.78 |
| 10m | duckdb-native | 43 | 7.009000 | 9.300146 | 32.097585 | 1144.39 |
| 10m | rudb-native | 43 | 1.460018 | 2.025644 | 5.487058 | 286.94 |
| 10m | duckdb-parquet | 43 | 10.299000 | 12.403160 | 41.923234 | 1013.48 |
| 10m | rudb-parquet | 43 | 8.651043 | 9.297486 | 37.261612 | 816.97 |

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
| 1k | duckdb-native | q1 | 0.001000 | 0.001000 | 0.001000 | 0.017581 | 0.016330 | 18.08 |
| 1k | duckdb-native | q2 | 0.001000 | 0.001000 | 0.001000 | 0.020002 | 0.018925 | 19.64 |
| 1k | duckdb-native | q3 | 0.001000 | 0.001000 | 0.000000 | 0.018382 | 0.017315 | 19.16 |
| 1k | duckdb-native | q4 | 0.000000 | 0.001000 | 0.001000 | 0.019079 | 0.017949 | 18.67 |
| 1k | duckdb-native | q5 | 0.001000 | 0.001000 | 0.001000 | 0.017849 | 0.017673 | 23.88 |
| 1k | duckdb-native | q6 | 0.001000 | 0.001000 | 0.001000 | 0.018408 | 0.017890 | 23.14 |
| 1k | duckdb-native | q7 | 0.001000 | 0.000000 | 0.001000 | 0.017753 | 0.016666 | 18.50 |
| 1k | duckdb-native | q8 | 0.001000 | 0.001000 | 0.000000 | 0.017449 | 0.016836 | 20.39 |
| 1k | duckdb-native | q9 | 0.002000 | 0.001000 | 0.001000 | 0.018515 | 0.019839 | 30.92 |
| 1k | duckdb-native | q10 | 0.001000 | 0.002000 | 0.000000 | 0.018871 | 0.020192 | 32.28 |
| 1k | duckdb-native | q11 | 0.002000 | 0.001000 | 0.001000 | 0.019136 | 0.019995 | 29.83 |
| 1k | duckdb-native | q12 | 0.002000 | 0.002000 | 0.001000 | 0.020608 | 0.021321 | 31.45 |
| 1k | duckdb-native | q13 | 0.001000 | 0.001000 | 0.001000 | 0.020819 | 0.021190 | 25.09 |
| 1k | duckdb-native | q14 | 0.002000 | 0.002000 | 0.000000 | 0.022466 | 0.023864 | 33.30 |
| 1k | duckdb-native | q15 | 0.002000 | 0.001000 | 0.001000 | 0.022154 | 0.021831 | 25.69 |
| 1k | duckdb-native | q16 | 0.001000 | 0.001000 | 0.001000 | 0.021330 | 0.021113 | 26.27 |
| 1k | duckdb-native | q17 | 0.002000 | 0.002000 | 0.001000 | 0.022756 | 0.022322 | 26.55 |
| 1k | duckdb-native | q18 | 0.001000 | 0.001000 | 0.001000 | 0.021055 | 0.020355 | 25.72 |
| 1k | duckdb-native | q19 | 0.002000 | 0.002000 | 0.000000 | 0.022286 | 0.022267 | 27.19 |
| 1k | duckdb-native | q20 | 0.001000 | 0.001000 | 0.001000 | 0.020354 | 0.019175 | 18.64 |
| 1k | duckdb-native | q21 | 0.001000 | 0.000000 | 0.001000 | 0.019218 | 0.018013 | 19.08 |
| 1k | duckdb-native | q22 | 0.001000 | 0.001000 | 0.000000 | 0.019893 | 0.019214 | 19.80 |
| 1k | duckdb-native | q23 | 0.002000 | 0.001000 | 0.000000 | 0.019185 | 0.018534 | 19.97 |
| 1k | duckdb-native | q24 | 0.004000 | 0.005000 | 0.000000 | 0.027916 | 0.026806 | 35.88 |
| 1k | duckdb-native | q25 | 0.002000 | 0.002000 | 0.002000 | 0.026050 | 0.025216 | 24.08 |
| 1k | duckdb-native | q26 | 0.001000 | 0.001000 | 0.000000 | 0.025267 | 0.023697 | 20.47 |
| 1k | duckdb-native | q27 | 0.001000 | 0.001000 | 0.000000 | 0.022914 | 0.021574 | 20.25 |
| 1k | duckdb-native | q28 | 0.002000 | 0.001000 | 0.001000 | 0.022508 | 0.021897 | 24.55 |
| 1k | duckdb-native | q29 | 0.002000 | 0.002000 | 0.000000 | 0.022334 | 0.023239 | 26.03 |
| 1k | duckdb-native | q30 | 0.005000 | 0.005000 | 0.000000 | 0.025039 | 0.024111 | 29.36 |
| 1k | duckdb-native | q31 | 0.002000 | 0.002000 | 0.001000 | 0.021808 | 0.022152 | 28.55 |
| 1k | duckdb-native | q32 | 0.002000 | 0.002000 | 0.001000 | 0.022220 | 0.022194 | 27.02 |
| 1k | duckdb-native | q33 | 0.002000 | 0.002000 | 0.001000 | 0.022327 | 0.022411 | 26.73 |
| 1k | duckdb-native | q34 | 0.002000 | 0.001000 | 0.000000 | 0.021236 | 0.021233 | 25.67 |
| 1k | duckdb-native | q35 | 0.001000 | 0.002000 | 0.001000 | 0.021651 | 0.021914 | 26.16 |
| 1k | duckdb-native | q36 | 0.002000 | 0.002000 | 0.001000 | 0.025042 | 0.024725 | 26.64 |
| 1k | duckdb-native | q37 | 0.002000 | 0.002000 | 0.000000 | 0.023304 | 0.023256 | 25.08 |
| 1k | duckdb-native | q38 | 0.002000 | 0.001000 | 0.001000 | 0.021887 | 0.021793 | 23.80 |
| 1k | duckdb-native | q39 | 0.001000 | 0.001000 | 0.000000 | 0.022463 | 0.022035 | 20.86 |
| 1k | duckdb-native | q40 | 0.002000 | 0.002000 | 0.000000 | 0.022475 | 0.023634 | 27.42 |
| 1k | duckdb-native | q41 | 0.002000 | 0.002000 | 0.000000 | 0.022855 | 0.023448 | 25.25 |
| 1k | duckdb-native | q42 | 0.001000 | 0.002000 | 0.001000 | 0.021925 | 0.021776 | 24.58 |
| 1k | duckdb-native | q43 | 0.002000 | 0.001000 | 0.001000 | 0.021262 | 0.021347 | 25.12 |
| 1k | rudb-native | q1 | 0.000768 | 0.000671 | 0.000032 | 0.005764 | 0.004757 | 9.22 |
| 1k | rudb-native | q2 | 0.000802 | 0.000766 | 0.000113 | 0.006995 | 0.006012 | 10.52 |
| 1k | rudb-native | q3 | 0.000716 | 0.000718 | 0.000063 | 0.006051 | 0.005057 | 9.89 |
| 1k | rudb-native | q4 | 0.000788 | 0.000742 | 0.000072 | 0.006352 | 0.005241 | 9.11 |
| 1k | rudb-native | q5 | 0.000678 | 0.000644 | 0.000050 | 0.005323 | 0.004573 | 8.94 |
| 1k | rudb-native | q6 | 0.000636 | 0.000668 | 0.000072 | 0.005784 | 0.004838 | 9.25 |
| 1k | rudb-native | q7 | 0.000614 | 0.000663 | 0.000024 | 0.005925 | 0.004994 | 9.22 |
| 1k | rudb-native | q8 | 0.001192 | 0.001032 | 0.000154 | 0.005898 | 0.005149 | 10.77 |
| 1k | rudb-native | q9 | 0.001017 | 0.000919 | 0.000127 | 0.005951 | 0.004991 | 10.31 |
| 1k | rudb-native | q10 | 0.001304 | 0.001190 | 0.000045 | 0.006219 | 0.005214 | 11.03 |
| 1k | rudb-native | q11 | 0.001037 | 0.001022 | 0.000044 | 0.006134 | 0.005121 | 10.78 |
| 1k | rudb-native | q12 | 0.001087 | 0.001142 | 0.000063 | 0.006731 | 0.005553 | 10.84 |
| 1k | rudb-native | q13 | 0.001152 | 0.001172 | 0.000170 | 0.006858 | 0.006031 | 11.03 |
| 1k | rudb-native | q14 | 0.001189 | 0.001270 | 0.000062 | 0.007250 | 0.006112 | 11.56 |
| 1k | rudb-native | q15 | 0.001317 | 0.001411 | 0.000055 | 0.007276 | 0.006146 | 11.89 |
| 1k | rudb-native | q16 | 0.000813 | 0.000856 | 0.000075 | 0.006656 | 0.005639 | 9.61 |
| 1k | rudb-native | q17 | 0.000877 | 0.000866 | 0.000137 | 0.006482 | 0.005454 | 9.53 |
| 1k | rudb-native | q18 | 0.000962 | 0.000859 | 0.000102 | 0.006941 | 0.006051 | 9.73 |
| 1k | rudb-native | q19 | 0.001074 | 0.001007 | 0.000118 | 0.006829 | 0.005730 | 10.03 |
| 1k | rudb-native | q20 | 0.000767 | 0.000760 | 0.000103 | 0.006574 | 0.005442 | 9.39 |
| 1k | rudb-native | q21 | 0.001036 | 0.001020 | 0.000172 | 0.006533 | 0.005429 | 10.80 |
| 1k | rudb-native | q22 | 0.001158 | 0.001190 | 0.000126 | 0.007154 | 0.005864 | 10.75 |
| 1k | rudb-native | q23 | 0.001231 | 0.001206 | 0.000040 | 0.006671 | 0.005641 | 11.27 |
| 1k | rudb-native | q24 | 0.001058 | 0.001207 | 0.000121 | 0.007649 | 0.006438 | 10.78 |
| 1k | rudb-native | q25 | 0.001739 | 0.001201 | 0.000128 | 0.008344 | 0.007031 | 11.17 |
| 1k | rudb-native | q26 | 0.001255 | 0.001173 | 0.000058 | 0.008154 | 0.006981 | 10.06 |
| 1k | rudb-native | q27 | 0.001326 | 0.001140 | 0.000143 | 0.007229 | 0.006122 | 10.12 |
| 1k | rudb-native | q28 | 0.001583 | 0.001407 | 0.000192 | 0.007719 | 0.006539 | 11.25 |
| 1k | rudb-native | q29 | 0.001592 | 0.001588 | 0.000088 | 0.007684 | 0.006474 | 11.45 |
| 1k | rudb-native | q30 | 0.002997 | 0.003046 | 0.000062 | 0.008831 | 0.007670 | 10.98 |
| 1k | rudb-native | q31 | 0.001462 | 0.001501 | 0.000121 | 0.007579 | 0.006266 | 11.11 |
| 1k | rudb-native | q32 | 0.001182 | 0.001088 | 0.000067 | 0.006814 | 0.005712 | 10.14 |
| 1k | rudb-native | q33 | 0.000979 | 0.000954 | 0.000053 | 0.006677 | 0.005600 | 9.73 |
| 1k | rudb-native | q34 | 0.000995 | 0.001040 | 0.000142 | 0.006964 | 0.005863 | 9.98 |
| 1k | rudb-native | q35 | 0.001245 | 0.001355 | 0.000192 | 0.007314 | 0.006302 | 11.23 |
| 1k | rudb-native | q36 | 0.001437 | 0.001404 | 0.000095 | 0.008162 | 0.006827 | 10.66 |
| 1k | rudb-native | q37 | 0.001731 | 0.001665 | 0.000111 | 0.008001 | 0.007083 | 11.78 |
| 1k | rudb-native | q38 | 0.001700 | 0.001669 | 0.000097 | 0.007751 | 0.006703 | 11.89 |
| 1k | rudb-native | q39 | 0.001120 | 0.001228 | 0.000204 | 0.008142 | 0.006948 | 11.23 |
| 1k | rudb-native | q40 | 0.001278 | 0.001334 | 0.000055 | 0.007353 | 0.006081 | 10.98 |
| 1k | rudb-native | q41 | 0.001402 | 0.001259 | 0.000149 | 0.007395 | 0.006718 | 10.88 |
| 1k | rudb-native | q42 | 0.001282 | 0.001264 | 0.000043 | 0.007444 | 0.006331 | 10.95 |
| 1k | rudb-native | q43 | 0.001278 | 0.001194 | 0.000184 | 0.006751 | 0.005678 | 11.22 |
| 1k | duckdb-parquet | q1 | 0.001000 | 0.001000 | 0.001000 | 0.017702 | 0.016470 | 18.36 |
| 1k | duckdb-parquet | q2 | 0.001000 | 0.001000 | 0.001000 | 0.020249 | 0.018652 | 19.84 |
| 1k | duckdb-parquet | q3 | 0.001000 | 0.001000 | 0.000000 | 0.018957 | 0.017834 | 19.45 |
| 1k | duckdb-parquet | q4 | 0.001000 | 0.001000 | 0.000000 | 0.019651 | 0.018447 | 18.50 |
| 1k | duckdb-parquet | q5 | 0.002000 | 0.001000 | 0.000000 | 0.018655 | 0.018243 | 21.88 |
| 1k | duckdb-parquet | q6 | 0.001000 | 0.002000 | 0.001000 | 0.018822 | 0.018357 | 24.70 |
| 1k | duckdb-parquet | q7 | 0.000000 | 0.001000 | 0.001000 | 0.017649 | 0.016698 | 18.47 |
| 1k | duckdb-parquet | q8 | 0.002000 | 0.001000 | 0.000000 | 0.017515 | 0.017030 | 19.61 |
| 1k | duckdb-parquet | q9 | 0.002000 | 0.002000 | 0.000000 | 0.018681 | 0.019621 | 29.00 |
| 1k | duckdb-parquet | q10 | 0.002000 | 0.002000 | 0.001000 | 0.018483 | 0.020182 | 32.58 |
| 1k | duckdb-parquet | q11 | 0.001000 | 0.002000 | 0.000000 | 0.019129 | 0.020126 | 29.44 |
| 1k | duckdb-parquet | q12 | 0.002000 | 0.002000 | 0.000000 | 0.020858 | 0.021530 | 29.33 |
| 1k | duckdb-parquet | q13 | 0.002000 | 0.002000 | 0.001000 | 0.019619 | 0.019840 | 24.61 |
| 1k | duckdb-parquet | q14 | 0.002000 | 0.002000 | 0.000000 | 0.022269 | 0.023346 | 30.92 |
| 1k | duckdb-parquet | q15 | 0.002000 | 0.002000 | 0.000000 | 0.022577 | 0.022717 | 25.08 |
| 1k | duckdb-parquet | q16 | 0.002000 | 0.002000 | 0.000000 | 0.021286 | 0.022029 | 24.48 |
| 1k | duckdb-parquet | q17 | 0.002000 | 0.002000 | 0.000000 | 0.020343 | 0.021126 | 26.67 |
| 1k | duckdb-parquet | q18 | 0.002000 | 0.002000 | 0.000000 | 0.022029 | 0.021839 | 24.45 |
| 1k | duckdb-parquet | q19 | 0.003000 | 0.002000 | 0.001000 | 0.022071 | 0.022264 | 25.38 |
| 1k | duckdb-parquet | q20 | 0.001000 | 0.001000 | 0.000000 | 0.020354 | 0.019253 | 18.58 |
| 1k | duckdb-parquet | q21 | 0.001000 | 0.001000 | 0.000000 | 0.019188 | 0.018135 | 18.52 |
| 1k | duckdb-parquet | q22 | 0.001000 | 0.001000 | 0.001000 | 0.020046 | 0.019447 | 19.80 |
| 1k | duckdb-parquet | q23 | 0.001000 | 0.002000 | 0.001000 | 0.020175 | 0.019203 | 20.41 |
| 1k | duckdb-parquet | q24 | 0.004000 | 0.007000 | 0.001000 | 0.029156 | 0.030675 | 34.05 |
| 1k | duckdb-parquet | q25 | 0.002000 | 0.002000 | 0.001000 | 0.025640 | 0.025064 | 23.55 |
| 1k | duckdb-parquet | q26 | 0.002000 | 0.002000 | 0.001000 | 0.024980 | 0.023721 | 19.92 |
| 1k | duckdb-parquet | q27 | 0.001000 | 0.001000 | 0.001000 | 0.022345 | 0.021286 | 19.61 |
| 1k | duckdb-parquet | q28 | 0.002000 | 0.002000 | 0.000000 | 0.022956 | 0.023154 | 24.11 |
| 1k | duckdb-parquet | q29 | 0.003000 | 0.003000 | 0.001000 | 0.023033 | 0.023502 | 25.23 |
| 1k | duckdb-parquet | q30 | 0.005000 | 0.005000 | 0.000000 | 0.024908 | 0.024480 | 29.22 |
| 1k | duckdb-parquet | q31 | 0.002000 | 0.002000 | 0.001000 | 0.021845 | 0.021851 | 25.83 |
| 1k | duckdb-parquet | q32 | 0.002000 | 0.002000 | 0.001000 | 0.021962 | 0.022487 | 25.67 |
| 1k | duckdb-parquet | q33 | 0.002000 | 0.002000 | 0.000000 | 0.021832 | 0.022192 | 25.41 |
| 1k | duckdb-parquet | q34 | 0.002000 | 0.002000 | 0.001000 | 0.021148 | 0.021337 | 25.22 |
| 1k | duckdb-parquet | q35 | 0.002000 | 0.002000 | 0.001000 | 0.022436 | 0.022704 | 26.30 |
| 1k | duckdb-parquet | q36 | 0.003000 | 0.003000 | 0.001000 | 0.024814 | 0.024631 | 26.09 |
| 1k | duckdb-parquet | q37 | 0.002000 | 0.002000 | 0.000000 | 0.023442 | 0.023578 | 23.67 |
| 1k | duckdb-parquet | q38 | 0.003000 | 0.002000 | 0.000000 | 0.022992 | 0.022977 | 23.47 |
| 1k | duckdb-parquet | q39 | 0.001000 | 0.001000 | 0.001000 | 0.022304 | 0.021306 | 19.69 |
| 1k | duckdb-parquet | q40 | 0.002000 | 0.002000 | 0.001000 | 0.022735 | 0.023698 | 26.45 |
| 1k | duckdb-parquet | q41 | 0.003000 | 0.002000 | 0.001000 | 0.022107 | 0.022644 | 23.48 |
| 1k | duckdb-parquet | q42 | 0.002000 | 0.002000 | 0.000000 | 0.022441 | 0.022813 | 21.97 |
| 1k | duckdb-parquet | q43 | 0.002000 | 0.002000 | 0.001000 | 0.021344 | 0.021473 | 23.70 |
| 1k | rudb-parquet | q1 | 0.001055 | 0.000936 | 0.000091 | 0.006155 | 0.005565 | 9.69 |
| 1k | rudb-parquet | q2 | 0.001138 | 0.001091 | 0.000089 | 0.006647 | 0.005556 | 10.06 |
| 1k | rudb-parquet | q3 | 0.001003 | 0.000985 | 0.000067 | 0.006362 | 0.005322 | 9.72 |
| 1k | rudb-parquet | q4 | 0.001175 | 0.000980 | 0.000138 | 0.006274 | 0.005271 | 9.78 |
| 1k | rudb-parquet | q5 | 0.000980 | 0.000867 | 0.000011 | 0.005755 | 0.004887 | 9.70 |
| 1k | rudb-parquet | q6 | 0.000983 | 0.000983 | 0.000151 | 0.005790 | 0.004950 | 11.16 |
| 1k | rudb-parquet | q7 | 0.000622 | 0.000642 | 0.000052 | 0.005466 | 0.004585 | 8.77 |
| 1k | rudb-parquet | q8 | 0.001098 | 0.001097 | 0.000155 | 0.005902 | 0.004983 | 10.09 |
| 1k | rudb-parquet | q9 | 0.000926 | 0.001007 | 0.000044 | 0.005920 | 0.004959 | 9.77 |
| 1k | rudb-parquet | q10 | 0.001368 | 0.001247 | 0.000163 | 0.006139 | 0.005284 | 10.38 |
| 1k | rudb-parquet | q11 | 0.001021 | 0.001126 | 0.000111 | 0.006477 | 0.005302 | 10.09 |
| 1k | rudb-parquet | q12 | 0.001219 | 0.001165 | 0.000095 | 0.006633 | 0.005618 | 10.34 |
| 1k | rudb-parquet | q13 | 0.001129 | 0.001150 | 0.000040 | 0.006615 | 0.005569 | 10.11 |
| 1k | rudb-parquet | q14 | 0.001261 | 0.001270 | 0.000070 | 0.007116 | 0.006090 | 10.36 |
| 1k | rudb-parquet | q15 | 0.001278 | 0.001221 | 0.000160 | 0.006870 | 0.005786 | 10.36 |
| 1k | rudb-parquet | q16 | 0.001145 | 0.001211 | 0.000084 | 0.007105 | 0.005948 | 10.06 |
| 1k | rudb-parquet | q17 | 0.001227 | 0.001222 | 0.000120 | 0.006870 | 0.005802 | 9.88 |
| 1k | rudb-parquet | q18 | 0.001175 | 0.001146 | 0.000111 | 0.006830 | 0.005784 | 10.05 |
| 1k | rudb-parquet | q19 | 0.001396 | 0.001322 | 0.000112 | 0.006981 | 0.005961 | 10.05 |
| 1k | rudb-parquet | q20 | 0.000910 | 0.000871 | 0.000039 | 0.006598 | 0.005485 | 9.17 |
| 1k | rudb-parquet | q21 | 0.001168 | 0.001251 | 0.000053 | 0.006667 | 0.005656 | 10.20 |
| 1k | rudb-parquet | q22 | 0.001362 | 0.001328 | 0.000103 | 0.006820 | 0.005698 | 10.50 |
| 1k | rudb-parquet | q23 | 0.001563 | 0.001567 | 0.000056 | 0.007093 | 0.005989 | 10.78 |
| 1k | rudb-parquet | q24 | 0.001306 | 0.001488 | 0.000098 | 0.007988 | 0.006633 | 11.00 |
| 1k | rudb-parquet | q25 | 0.001198 | 0.001216 | 0.000117 | 0.007909 | 0.006626 | 10.28 |
| 1k | rudb-parquet | q26 | 0.001605 | 0.001250 | 0.000066 | 0.008396 | 0.007007 | 9.64 |
| 1k | rudb-parquet | q27 | 0.001272 | 0.001168 | 0.000114 | 0.007133 | 0.006063 | 10.16 |
| 1k | rudb-parquet | q28 | 0.001581 | 0.001460 | 0.000096 | 0.007325 | 0.006176 | 10.75 |
| 1k | rudb-parquet | q29 | 0.001613 | 0.001639 | 0.000076 | 0.007681 | 0.006367 | 11.03 |
| 1k | rudb-parquet | q30 | 0.003576 | 0.003491 | 0.000257 | 0.009224 | 0.008136 | 11.61 |
| 1k | rudb-parquet | q31 | 0.001390 | 0.001549 | 0.000129 | 0.007773 | 0.006427 | 10.47 |
| 1k | rudb-parquet | q32 | 0.001447 | 0.001434 | 0.000088 | 0.007238 | 0.006051 | 10.25 |
| 1k | rudb-parquet | q33 | 0.001355 | 0.001558 | 0.000078 | 0.007492 | 0.006251 | 10.11 |
| 1k | rudb-parquet | q34 | 0.001396 | 0.001428 | 0.000157 | 0.007106 | 0.005996 | 10.42 |
| 1k | rudb-parquet | q35 | 0.001456 | 0.001353 | 0.000060 | 0.007185 | 0.006079 | 10.08 |
| 1k | rudb-parquet | q36 | 0.001353 | 0.001426 | 0.000067 | 0.008291 | 0.006960 | 10.19 |
| 1k | rudb-parquet | q37 | 0.001588 | 0.001538 | 0.000078 | 0.007651 | 0.006480 | 10.42 |
| 1k | rudb-parquet | q38 | 0.001698 | 0.001638 | 0.000174 | 0.007698 | 0.006576 | 10.33 |
| 1k | rudb-parquet | q39 | 0.001353 | 0.001493 | 0.000140 | 0.007809 | 0.006565 | 10.33 |
| 1k | rudb-parquet | q40 | 0.001749 | 0.001674 | 0.000095 | 0.007464 | 0.006479 | 10.86 |
| 1k | rudb-parquet | q41 | 0.001448 | 0.001342 | 0.000119 | 0.007284 | 0.006123 | 10.45 |
| 1k | rudb-parquet | q42 | 0.001534 | 0.001393 | 0.000128 | 0.007884 | 0.006322 | 10.84 |
| 1k | rudb-parquet | q43 | 0.001404 | 0.001293 | 0.000100 | 0.006811 | 0.005833 | 10.47 |
| 10k | duckdb-native | q1 | 0.000000 | 0.000000 | 0.001000 | 0.018806 | 0.017523 | 19.22 |
| 10k | duckdb-native | q2 | 0.001000 | 0.001000 | 0.001000 | 0.018900 | 0.017676 | 18.98 |
| 10k | duckdb-native | q3 | 0.001000 | 0.001000 | 0.001000 | 0.018055 | 0.016987 | 21.12 |
| 10k | duckdb-native | q4 | 0.000000 | 0.000000 | 0.001000 | 0.017115 | 0.016083 | 18.98 |
| 10k | duckdb-native | q5 | 0.002000 | 0.001000 | 0.000000 | 0.016553 | 0.016057 | 23.50 |
| 10k | duckdb-native | q6 | 0.001000 | 0.001000 | 0.000000 | 0.015933 | 0.015596 | 23.16 |
| 10k | duckdb-native | q7 | 0.000000 | 0.001000 | 0.001000 | 0.015084 | 0.014115 | 18.61 |
| 10k | duckdb-native | q8 | 0.001000 | 0.001000 | 0.000000 | 0.015773 | 0.014893 | 20.36 |
| 10k | duckdb-native | q9 | 0.002000 | 0.001000 | 0.001000 | 0.016170 | 0.017199 | 29.70 |
| 10k | duckdb-native | q10 | 0.002000 | 0.002000 | 0.000000 | 0.016612 | 0.017811 | 33.25 |
| 10k | duckdb-native | q11 | 0.001000 | 0.001000 | 0.001000 | 0.016370 | 0.016649 | 31.22 |
| 10k | duckdb-native | q12 | 0.002000 | 0.001000 | 0.001000 | 0.016485 | 0.017167 | 30.09 |
| 10k | duckdb-native | q13 | 0.002000 | 0.001000 | 0.000000 | 0.016865 | 0.016850 | 25.47 |
| 10k | duckdb-native | q14 | 0.001000 | 0.002000 | 0.001000 | 0.016323 | 0.017338 | 32.28 |
| 10k | duckdb-native | q15 | 0.001000 | 0.001000 | 0.001000 | 0.016432 | 0.016457 | 25.09 |
| 10k | duckdb-native | q16 | 0.001000 | 0.001000 | 0.001000 | 0.016508 | 0.016508 | 24.95 |
| 10k | duckdb-native | q17 | 0.002000 | 0.001000 | 0.001000 | 0.017213 | 0.018081 | 27.39 |
| 10k | duckdb-native | q18 | 0.001000 | 0.002000 | 0.000000 | 0.016606 | 0.016359 | 25.66 |
| 10k | duckdb-native | q19 | 0.002000 | 0.002000 | 0.000000 | 0.017270 | 0.017712 | 28.55 |
| 10k | duckdb-native | q20 | 0.000000 | 0.000000 | 0.000000 | 0.015537 | 0.014191 | 18.77 |
| 10k | duckdb-native | q21 | 0.001000 | 0.001000 | 0.001000 | 0.016757 | 0.015650 | 20.44 |
| 10k | duckdb-native | q22 | 0.002000 | 0.001000 | 0.001000 | 0.016439 | 0.015684 | 21.06 |
| 10k | duckdb-native | q23 | 0.002000 | 0.002000 | 0.000000 | 0.019604 | 0.019266 | 28.34 |
| 10k | duckdb-native | q24 | 0.005000 | 0.006000 | 0.001000 | 0.027545 | 0.028972 | 38.27 |
| 10k | duckdb-native | q25 | 0.002000 | 0.002000 | 0.001000 | 0.027510 | 0.026853 | 22.91 |
| 10k | duckdb-native | q26 | 0.001000 | 0.001000 | 0.000000 | 0.026047 | 0.024337 | 19.59 |
| 10k | duckdb-native | q27 | 0.001000 | 0.001000 | 0.000000 | 0.022301 | 0.020545 | 19.86 |
| 10k | duckdb-native | q28 | 0.003000 | 0.002000 | 0.000000 | 0.022133 | 0.023063 | 25.17 |
| 10k | duckdb-native | q29 | 0.008000 | 0.008000 | 0.001000 | 0.028356 | 0.035271 | 27.97 |
| 10k | duckdb-native | q30 | 0.005000 | 0.005000 | 0.000000 | 0.025158 | 0.024732 | 30.22 |
| 10k | duckdb-native | q31 | 0.002000 | 0.002000 | 0.001000 | 0.022716 | 0.023296 | 27.98 |
| 10k | duckdb-native | q32 | 0.002000 | 0.002000 | 0.000000 | 0.022402 | 0.022736 | 27.19 |
| 10k | duckdb-native | q33 | 0.003000 | 0.003000 | 0.001000 | 0.022849 | 0.023831 | 28.64 |
| 10k | duckdb-native | q34 | 0.003000 | 0.002000 | 0.001000 | 0.022903 | 0.024091 | 28.11 |
| 10k | duckdb-native | q35 | 0.003000 | 0.003000 | 0.001000 | 0.023358 | 0.025179 | 29.80 |
| 10k | duckdb-native | q36 | 0.002000 | 0.002000 | 0.000000 | 0.022382 | 0.022933 | 27.00 |
| 10k | duckdb-native | q37 | 0.002000 | 0.002000 | 0.000000 | 0.022089 | 0.022432 | 25.73 |
| 10k | duckdb-native | q38 | 0.004000 | 0.002000 | 0.001000 | 0.022171 | 0.022072 | 27.91 |
| 10k | duckdb-native | q39 | 0.002000 | 0.002000 | 0.001000 | 0.021908 | 0.022573 | 25.78 |
| 10k | duckdb-native | q40 | 0.003000 | 0.002000 | 0.001000 | 0.023098 | 0.023999 | 28.84 |
| 10k | duckdb-native | q41 | 0.001000 | 0.002000 | 0.000000 | 0.022705 | 0.022391 | 26.69 |
| 10k | duckdb-native | q42 | 0.002000 | 0.002000 | 0.000000 | 0.024073 | 0.023973 | 25.38 |
| 10k | duckdb-native | q43 | 0.002000 | 0.002000 | 0.000000 | 0.026813 | 0.026718 | 26.16 |
| 10k | rudb-native | q1 | 0.000649 | 0.000682 | 0.000031 | 0.006375 | 0.005326 | 9.25 |
| 10k | rudb-native | q2 | 0.000760 | 0.000783 | 0.000073 | 0.006723 | 0.005661 | 9.69 |
| 10k | rudb-native | q3 | 0.000798 | 0.000737 | 0.000075 | 0.005776 | 0.004939 | 9.31 |
| 10k | rudb-native | q4 | 0.000696 | 0.000649 | 0.000032 | 0.005688 | 0.004838 | 9.33 |
| 10k | rudb-native | q5 | 0.000624 | 0.000594 | 0.000059 | 0.005214 | 0.004378 | 8.95 |
| 10k | rudb-native | q6 | 0.000540 | 0.000594 | 0.000085 | 0.005083 | 0.004292 | 9.38 |
| 10k | rudb-native | q7 | 0.000678 | 0.000624 | 0.000026 | 0.005504 | 0.004612 | 9.14 |
| 10k | rudb-native | q8 | 0.000934 | 0.000946 | 0.000049 | 0.005534 | 0.004775 | 10.95 |
| 10k | rudb-native | q9 | 0.000966 | 0.001028 | 0.000091 | 0.005498 | 0.004648 | 11.09 |
| 10k | rudb-native | q10 | 0.001302 | 0.001392 | 0.000108 | 0.006044 | 0.005120 | 11.61 |
| 10k | rudb-native | q11 | 0.000970 | 0.001004 | 0.000073 | 0.005928 | 0.005129 | 11.44 |
| 10k | rudb-native | q12 | 0.000989 | 0.001034 | 0.000061 | 0.005880 | 0.005101 | 11.20 |
| 10k | rudb-native | q13 | 0.001107 | 0.001154 | 0.000029 | 0.006160 | 0.005398 | 11.89 |
| 10k | rudb-native | q14 | 0.001243 | 0.001246 | 0.000169 | 0.006351 | 0.005374 | 11.56 |
| 10k | rudb-native | q15 | 0.001545 | 0.001510 | 0.000077 | 0.006639 | 0.005793 | 11.97 |
| 10k | rudb-native | q16 | 0.001182 | 0.001081 | 0.000134 | 0.006273 | 0.005219 | 11.16 |
| 10k | rudb-native | q17 | 0.001400 | 0.001312 | 0.000136 | 0.006326 | 0.005439 | 11.56 |
| 10k | rudb-native | q18 | 0.000900 | 0.001005 | 0.000087 | 0.006046 | 0.005089 | 10.59 |
| 10k | rudb-native | q19 | 0.001393 | 0.001463 | 0.000077 | 0.006536 | 0.005610 | 12.09 |
| 10k | rudb-native | q20 | 0.000642 | 0.000642 | 0.000033 | 0.005370 | 0.004534 | 9.45 |
| 10k | rudb-native | q21 | 0.000987 | 0.001039 | 0.000113 | 0.006009 | 0.005018 | 10.88 |
| 10k | rudb-native | q22 | 0.001109 | 0.001177 | 0.000070 | 0.005885 | 0.005084 | 11.64 |
| 10k | rudb-native | q23 | 0.002989 | 0.002884 | 0.000104 | 0.008336 | 0.007507 | 13.31 |
| 10k | rudb-native | q24 | 0.001183 | 0.001350 | 0.000282 | 0.008449 | 0.007264 | 10.39 |
| 10k | rudb-native | q25 | 0.001612 | 0.001619 | 0.000165 | 0.009784 | 0.008225 | 10.45 |
| 10k | rudb-native | q26 | 0.001559 | 0.001523 | 0.000139 | 0.009452 | 0.008083 | 10.48 |
| 10k | rudb-native | q27 | 0.001437 | 0.001368 | 0.000068 | 0.008035 | 0.006751 | 10.38 |
| 10k | rudb-native | q28 | 0.001696 | 0.001546 | 0.000081 | 0.007571 | 0.006797 | 12.19 |
| 10k | rudb-native | q29 | 0.002560 | 0.002638 | 0.000219 | 0.009404 | 0.009527 | 14.25 |
| 10k | rudb-native | q30 | 0.003139 | 0.003152 | 0.000145 | 0.010166 | 0.008781 | 10.98 |
| 10k | rudb-native | q31 | 0.001938 | 0.001772 | 0.000059 | 0.008171 | 0.007099 | 11.59 |
| 10k | rudb-native | q32 | 0.001363 | 0.001260 | 0.000058 | 0.007523 | 0.006418 | 10.48 |
| 10k | rudb-native | q33 | 0.000978 | 0.001039 | 0.000121 | 0.007370 | 0.006383 | 9.81 |
| 10k | rudb-native | q34 | 0.001070 | 0.001026 | 0.000078 | 0.007305 | 0.006168 | 9.94 |
| 10k | rudb-native | q35 | 0.001684 | 0.001721 | 0.000075 | 0.008092 | 0.007176 | 11.80 |
| 10k | rudb-native | q36 | 0.001062 | 0.001093 | 0.000096 | 0.007363 | 0.006313 | 10.20 |
| 10k | rudb-native | q37 | 0.001710 | 0.001583 | 0.000212 | 0.007929 | 0.007129 | 12.38 |
| 10k | rudb-native | q38 | 0.001722 | 0.001759 | 0.000222 | 0.008299 | 0.007512 | 12.47 |
| 10k | rudb-native | q39 | 0.001311 | 0.001304 | 0.000161 | 0.007793 | 0.006888 | 11.42 |
| 10k | rudb-native | q40 | 0.001665 | 0.001409 | 0.000248 | 0.007799 | 0.006852 | 11.59 |
| 10k | rudb-native | q41 | 0.001545 | 0.001288 | 0.000186 | 0.007746 | 0.006663 | 11.83 |
| 10k | rudb-native | q42 | 0.001233 | 0.001372 | 0.000135 | 0.008271 | 0.007214 | 11.59 |
| 10k | rudb-native | q43 | 0.001359 | 0.001489 | 0.000050 | 0.009052 | 0.007858 | 11.56 |
| 10k | duckdb-parquet | q1 | 0.001000 | 0.001000 | 0.000000 | 0.018934 | 0.017780 | 18.42 |
| 10k | duckdb-parquet | q2 | 0.001000 | 0.001000 | 0.000000 | 0.019725 | 0.018734 | 19.19 |
| 10k | duckdb-parquet | q3 | 0.001000 | 0.001000 | 0.000000 | 0.017970 | 0.017035 | 19.23 |
| 10k | duckdb-parquet | q4 | 0.001000 | 0.001000 | 0.000000 | 0.017757 | 0.016974 | 18.78 |
| 10k | duckdb-parquet | q5 | 0.001000 | 0.001000 | 0.001000 | 0.017376 | 0.016992 | 22.83 |
| 10k | duckdb-parquet | q6 | 0.001000 | 0.001000 | 0.001000 | 0.015819 | 0.015543 | 22.25 |
| 10k | duckdb-parquet | q7 | 0.000000 | 0.000000 | 0.001000 | 0.015157 | 0.014189 | 18.38 |
| 10k | duckdb-parquet | q8 | 0.001000 | 0.001000 | 0.000000 | 0.015466 | 0.014733 | 19.66 |
| 10k | duckdb-parquet | q9 | 0.001000 | 0.002000 | 0.000000 | 0.015946 | 0.017176 | 28.52 |
| 10k | duckdb-parquet | q10 | 0.002000 | 0.002000 | 0.000000 | 0.016464 | 0.017736 | 29.69 |
| 10k | duckdb-parquet | q11 | 0.001000 | 0.002000 | 0.001000 | 0.016462 | 0.017059 | 29.00 |
| 10k | duckdb-parquet | q12 | 0.001000 | 0.002000 | 0.001000 | 0.016690 | 0.017257 | 28.69 |
| 10k | duckdb-parquet | q13 | 0.001000 | 0.002000 | 0.001000 | 0.016314 | 0.016298 | 24.06 |
| 10k | duckdb-parquet | q14 | 0.002000 | 0.002000 | 0.000000 | 0.017297 | 0.018383 | 32.05 |
| 10k | duckdb-parquet | q15 | 0.002000 | 0.001000 | 0.001000 | 0.016530 | 0.017242 | 25.30 |
| 10k | duckdb-parquet | q16 | 0.001000 | 0.002000 | 0.001000 | 0.017083 | 0.017097 | 24.73 |
| 10k | duckdb-parquet | q17 | 0.002000 | 0.002000 | 0.000000 | 0.016397 | 0.016913 | 26.56 |
| 10k | duckdb-parquet | q18 | 0.002000 | 0.002000 | 0.001000 | 0.016562 | 0.016293 | 25.00 |
| 10k | duckdb-parquet | q19 | 0.002000 | 0.002000 | 0.001000 | 0.017654 | 0.018120 | 27.50 |
| 10k | duckdb-parquet | q20 | 0.000000 | 0.001000 | 0.000000 | 0.015415 | 0.014314 | 18.34 |
| 10k | duckdb-parquet | q21 | 0.001000 | 0.002000 | 0.000000 | 0.017220 | 0.016293 | 20.16 |
| 10k | duckdb-parquet | q22 | 0.001000 | 0.001000 | 0.001000 | 0.016801 | 0.016384 | 20.86 |
| 10k | duckdb-parquet | q23 | 0.003000 | 0.003000 | 0.000000 | 0.019884 | 0.019836 | 27.58 |
| 10k | duckdb-parquet | q24 | 0.008000 | 0.010000 | 0.002000 | 0.031450 | 0.037089 | 39.14 |
| 10k | duckdb-parquet | q25 | 0.003000 | 0.004000 | 0.000000 | 0.029315 | 0.029117 | 22.81 |
| 10k | duckdb-parquet | q26 | 0.002000 | 0.002000 | 0.001000 | 0.025518 | 0.024027 | 19.41 |
| 10k | duckdb-parquet | q27 | 0.002000 | 0.002000 | 0.000000 | 0.022540 | 0.021147 | 19.55 |
| 10k | duckdb-parquet | q28 | 0.003000 | 0.003000 | 0.000000 | 0.022317 | 0.023511 | 25.78 |
| 10k | duckdb-parquet | q29 | 0.008000 | 0.009000 | 0.000000 | 0.028990 | 0.036293 | 28.17 |
| 10k | duckdb-parquet | q30 | 0.005000 | 0.005000 | 0.000000 | 0.025128 | 0.024704 | 29.80 |
| 10k | duckdb-parquet | q31 | 0.003000 | 0.003000 | 0.001000 | 0.022490 | 0.022738 | 26.05 |
| 10k | duckdb-parquet | q32 | 0.003000 | 0.002000 | 0.001000 | 0.022207 | 0.022445 | 27.02 |
| 10k | duckdb-parquet | q33 | 0.003000 | 0.003000 | 0.000000 | 0.023321 | 0.023823 | 28.30 |
| 10k | duckdb-parquet | q34 | 0.003000 | 0.003000 | 0.000000 | 0.023399 | 0.025163 | 27.97 |
| 10k | duckdb-parquet | q35 | 0.003000 | 0.004000 | 0.001000 | 0.023650 | 0.024955 | 29.47 |
| 10k | duckdb-parquet | q36 | 0.003000 | 0.003000 | 0.000000 | 0.022973 | 0.023563 | 26.03 |
| 10k | duckdb-parquet | q37 | 0.003000 | 0.003000 | 0.001000 | 0.022417 | 0.023009 | 25.72 |
| 10k | duckdb-parquet | q38 | 0.002000 | 0.003000 | 0.000000 | 0.023033 | 0.023527 | 24.69 |
| 10k | duckdb-parquet | q39 | 0.002000 | 0.003000 | 0.001000 | 0.022317 | 0.022479 | 26.61 |
| 10k | duckdb-parquet | q40 | 0.003000 | 0.003000 | 0.001000 | 0.023277 | 0.024801 | 28.50 |
| 10k | duckdb-parquet | q41 | 0.002000 | 0.003000 | 0.001000 | 0.022917 | 0.022587 | 26.41 |
| 10k | duckdb-parquet | q42 | 0.002000 | 0.003000 | 0.001000 | 0.024483 | 0.024537 | 24.77 |
| 10k | duckdb-parquet | q43 | 0.003000 | 0.003000 | 0.001000 | 0.026655 | 0.026811 | 25.16 |
| 10k | rudb-parquet | q1 | 0.001002 | 0.000918 | 0.000099 | 0.006426 | 0.005427 | 9.70 |
| 10k | rudb-parquet | q2 | 0.001124 | 0.001075 | 0.000153 | 0.006568 | 0.005430 | 10.22 |
| 10k | rudb-parquet | q3 | 0.000995 | 0.000978 | 0.000080 | 0.005830 | 0.004957 | 9.86 |
| 10k | rudb-parquet | q4 | 0.001053 | 0.000924 | 0.000052 | 0.005599 | 0.004734 | 9.98 |
| 10k | rudb-parquet | q5 | 0.000907 | 0.000898 | 0.000066 | 0.005335 | 0.004542 | 10.09 |
| 10k | rudb-parquet | q6 | 0.001239 | 0.001165 | 0.000035 | 0.005470 | 0.004590 | 10.58 |
| 10k | rudb-parquet | q7 | 0.000505 | 0.000560 | 0.000044 | 0.004839 | 0.004018 | 8.56 |
| 10k | rudb-parquet | q8 | 0.000974 | 0.000963 | 0.000176 | 0.005243 | 0.004387 | 10.16 |
| 10k | rudb-parquet | q9 | 0.001100 | 0.001085 | 0.000041 | 0.005359 | 0.004730 | 10.86 |
| 10k | rudb-parquet | q10 | 0.001401 | 0.001458 | 0.000116 | 0.005874 | 0.004916 | 11.48 |
| 10k | rudb-parquet | q11 | 0.001168 | 0.001074 | 0.000028 | 0.005656 | 0.004726 | 10.52 |
| 10k | rudb-parquet | q12 | 0.001136 | 0.001108 | 0.000075 | 0.005567 | 0.004697 | 10.69 |
| 10k | rudb-parquet | q13 | 0.001232 | 0.001210 | 0.000058 | 0.005769 | 0.004860 | 10.31 |
| 10k | rudb-parquet | q14 | 0.001405 | 0.001321 | 0.000148 | 0.005846 | 0.004898 | 11.16 |
| 10k | rudb-parquet | q15 | 0.001544 | 0.001377 | 0.000060 | 0.006020 | 0.005031 | 10.64 |
| 10k | rudb-parquet | q16 | 0.001201 | 0.001135 | 0.000108 | 0.005776 | 0.004883 | 10.89 |
| 10k | rudb-parquet | q17 | 0.001766 | 0.001734 | 0.000042 | 0.006190 | 0.005280 | 11.67 |
| 10k | rudb-parquet | q18 | 0.001198 | 0.001219 | 0.000173 | 0.005704 | 0.004756 | 11.08 |
| 10k | rudb-parquet | q19 | 0.002054 | 0.002011 | 0.000071 | 0.006606 | 0.005644 | 12.16 |
| 10k | rudb-parquet | q20 | 0.000755 | 0.000749 | 0.000085 | 0.005207 | 0.004323 | 9.47 |
| 10k | rudb-parquet | q21 | 0.002169 | 0.002161 | 0.000129 | 0.007119 | 0.005901 | 12.88 |
| 10k | rudb-parquet | q22 | 0.002220 | 0.002256 | 0.000229 | 0.006734 | 0.005841 | 13.50 |
| 10k | rudb-parquet | q23 | 0.004234 | 0.004430 | 0.000164 | 0.009731 | 0.008550 | 14.94 |
| 10k | rudb-parquet | q24 | 0.002713 | 0.003110 | 0.000307 | 0.009484 | 0.008201 | 12.83 |
| 10k | rudb-parquet | q25 | 0.001886 | 0.001829 | 0.000147 | 0.009329 | 0.007670 | 10.11 |
| 10k | rudb-parquet | q26 | 0.001868 | 0.001819 | 0.000267 | 0.009500 | 0.007876 | 9.75 |
| 10k | rudb-parquet | q27 | 0.001583 | 0.001551 | 0.000190 | 0.007825 | 0.006655 | 9.92 |
| 10k | rudb-parquet | q28 | 0.002701 | 0.002756 | 0.000128 | 0.008574 | 0.007437 | 12.30 |
| 10k | rudb-parquet | q29 | 0.004039 | 0.003993 | 0.000137 | 0.010037 | 0.008723 | 13.94 |
| 10k | rudb-parquet | q30 | 0.003512 | 0.003624 | 0.000183 | 0.009604 | 0.008331 | 11.00 |
| 10k | rudb-parquet | q31 | 0.002152 | 0.001941 | 0.000060 | 0.007723 | 0.006614 | 11.11 |
| 10k | rudb-parquet | q32 | 0.001966 | 0.001948 | 0.000038 | 0.007767 | 0.006609 | 10.97 |
| 10k | rudb-parquet | q33 | 0.001838 | 0.001759 | 0.000170 | 0.007728 | 0.006833 | 11.52 |
| 10k | rudb-parquet | q34 | 0.003551 | 0.003463 | 0.000057 | 0.009351 | 0.008195 | 13.05 |
| 10k | rudb-parquet | q35 | 0.003473 | 0.003460 | 0.000114 | 0.009439 | 0.008199 | 13.03 |
| 10k | rudb-parquet | q36 | 0.001659 | 0.001527 | 0.000278 | 0.007520 | 0.006398 | 11.19 |
| 10k | rudb-parquet | q37 | 0.002921 | 0.002798 | 0.000058 | 0.008714 | 0.007592 | 12.27 |
| 10k | rudb-parquet | q38 | 0.003744 | 0.003740 | 0.000179 | 0.009789 | 0.008585 | 12.34 |
| 10k | rudb-parquet | q39 | 0.002715 | 0.002678 | 0.000011 | 0.008584 | 0.007391 | 12.11 |
| 10k | rudb-parquet | q40 | 0.003978 | 0.003994 | 0.000107 | 0.010168 | 0.008815 | 13.53 |
| 10k | rudb-parquet | q41 | 0.001624 | 0.001596 | 0.000090 | 0.007512 | 0.006306 | 11.22 |
| 10k | rudb-parquet | q42 | 0.001518 | 0.001516 | 0.000128 | 0.007738 | 0.006818 | 10.97 |
| 10k | rudb-parquet | q43 | 0.001775 | 0.001811 | 0.000090 | 0.008997 | 0.007525 | 10.91 |
| 1m | duckdb-native | q1 | 0.001000 | 0.001000 | 0.001000 | 0.023622 | 0.022084 | 19.89 |
| 1m | duckdb-native | q2 | 0.001000 | 0.001000 | 0.000000 | 0.021574 | 0.021538 | 22.50 |
| 1m | duckdb-native | q3 | 0.002000 | 0.002000 | 0.000000 | 0.022573 | 0.025340 | 25.70 |
| 1m | duckdb-native | q4 | 0.003000 | 0.002000 | 0.000000 | 0.023980 | 0.026825 | 29.00 |
| 1m | duckdb-native | q5 | 0.013000 | 0.012000 | 0.001000 | 0.036071 | 0.080690 | 63.86 |
| 1m | duckdb-native | q6 | 0.012000 | 0.010000 | 0.001000 | 0.034274 | 0.067886 | 55.62 |
| 1m | duckdb-native | q7 | 0.000000 | 0.001000 | 0.001000 | 0.023090 | 0.021382 | 18.94 |
| 1m | duckdb-native | q8 | 0.002000 | 0.001000 | 0.001000 | 0.021016 | 0.021777 | 24.03 |
| 1m | duckdb-native | q9 | 0.014000 | 0.017000 | 0.002000 | 0.039558 | 0.102527 | 75.75 |
| 1m | duckdb-native | q10 | 0.021000 | 0.026000 | 0.005000 | 0.052568 | 0.146443 | 86.19 |
| 1m | duckdb-native | q11 | 0.007000 | 0.006000 | 0.001000 | 0.031446 | 0.048723 | 50.75 |
| 1m | duckdb-native | q12 | 0.006000 | 0.007000 | 0.001000 | 0.033003 | 0.051174 | 52.47 |
| 1m | duckdb-native | q13 | 0.011000 | 0.010000 | 0.002000 | 0.036694 | 0.066766 | 56.38 |
| 1m | duckdb-native | q14 | 0.015000 | 0.015000 | 0.001000 | 0.041539 | 0.090350 | 81.78 |
| 1m | duckdb-native | q15 | 0.010000 | 0.010000 | 0.002000 | 0.035373 | 0.066946 | 59.78 |
| 1m | duckdb-native | q16 | 0.016000 | 0.016000 | 0.001000 | 0.041263 | 0.102055 | 74.91 |
| 1m | duckdb-native | q17 | 0.035000 | 0.034000 | 0.002000 | 0.061741 | 0.189945 | 157.62 |
| 1m | duckdb-native | q18 | 0.029000 | 0.031000 | 0.003000 | 0.059043 | 0.149340 | 137.42 |
| 1m | duckdb-native | q19 | 0.045000 | 0.040000 | 0.003000 | 0.070410 | 0.219094 | 169.08 |
| 1m | duckdb-native | q20 | 0.002000 | 0.002000 | 0.000000 | 0.026789 | 0.028794 | 28.48 |
| 1m | duckdb-native | q21 | 0.042000 | 0.027000 | 0.002000 | 0.055474 | 0.144751 | 89.62 |
| 1m | duckdb-native | q22 | 0.022000 | 0.021000 | 0.002000 | 0.048799 | 0.118453 | 102.94 |
| 1m | duckdb-native | q23 | 0.039000 | 0.022000 | 0.004000 | 0.051088 | 0.122232 | 124.77 |
| 1m | duckdb-native | q24 | 0.053000 | 0.041000 | 0.004000 | 0.075386 | 0.205636 | 231.86 |
| 1m | duckdb-native | q25 | 0.006000 | 0.005000 | 0.000000 | 0.030174 | 0.042594 | 37.67 |
| 1m | duckdb-native | q26 | 0.005000 | 0.005000 | 0.001000 | 0.029378 | 0.044821 | 30.56 |
| 1m | duckdb-native | q27 | 0.005000 | 0.005000 | 0.000000 | 0.028743 | 0.041255 | 34.41 |
| 1m | duckdb-native | q28 | 0.021000 | 0.021000 | 0.001000 | 0.048525 | 0.119813 | 98.94 |
| 1m | duckdb-native | q29 | 0.267000 | 0.253000 | 0.012000 | 0.283022 | 1.174159 | 139.28 |
| 1m | duckdb-native | q30 | 0.008000 | 0.007000 | 0.002000 | 0.031838 | 0.031508 | 33.53 |
| 1m | duckdb-native | q31 | 0.025000 | 0.011000 | 0.002000 | 0.035248 | 0.065592 | 64.61 |
| 1m | duckdb-native | q32 | 0.012000 | 0.015000 | 0.005000 | 0.040565 | 0.073995 | 73.16 |
| 1m | duckdb-native | q33 | 0.031000 | 0.032000 | 0.006000 | 0.057594 | 0.170658 | 134.47 |
| 1m | duckdb-native | q34 | 0.045000 | 0.044000 | 0.002000 | 0.073425 | 0.238610 | 228.23 |
| 1m | duckdb-native | q35 | 0.044000 | 0.057000 | 0.014000 | 0.090900 | 0.279351 | 232.33 |
| 1m | duckdb-native | q36 | 0.022000 | 0.018000 | 0.003000 | 0.041156 | 0.107328 | 77.08 |
| 1m | duckdb-native | q37 | 0.004000 | 0.002000 | 0.001000 | 0.019131 | 0.021290 | 31.38 |
| 1m | duckdb-native | q38 | 0.002000 | 0.002000 | 0.001000 | 0.018015 | 0.019765 | 29.11 |
| 1m | duckdb-native | q39 | 0.001000 | 0.001000 | 0.001000 | 0.017606 | 0.018961 | 30.22 |
| 1m | duckdb-native | q40 | 0.003000 | 0.003000 | 0.001000 | 0.019179 | 0.022753 | 37.41 |
| 1m | duckdb-native | q41 | 0.002000 | 0.002000 | 0.001000 | 0.018068 | 0.019053 | 28.94 |
| 1m | duckdb-native | q42 | 0.002000 | 0.001000 | 0.001000 | 0.017863 | 0.018848 | 27.78 |
| 1m | duckdb-native | q43 | 0.002000 | 0.002000 | 0.001000 | 0.017806 | 0.018730 | 28.17 |
| 1m | rudb-native | q1 | 0.000754 | 0.000826 | 0.000059 | 0.008162 | 0.006853 | 9.58 |
| 1m | rudb-native | q2 | 0.000871 | 0.000790 | 0.000091 | 0.006762 | 0.005681 | 9.41 |
| 1m | rudb-native | q3 | 0.000888 | 0.000862 | 0.000050 | 0.007167 | 0.006140 | 9.44 |
| 1m | rudb-native | q4 | 0.000939 | 0.000839 | 0.000060 | 0.007852 | 0.006556 | 9.64 |
| 1m | rudb-native | q5 | 0.000857 | 0.000819 | 0.000061 | 0.007813 | 0.006584 | 9.56 |
| 1m | rudb-native | q6 | 0.001164 | 0.000845 | 0.000072 | 0.007898 | 0.006645 | 9.19 |
| 1m | rudb-native | q7 | 0.000848 | 0.000858 | 0.000079 | 0.007575 | 0.006393 | 9.44 |
| 1m | rudb-native | q8 | 0.001670 | 0.001471 | 0.000093 | 0.007921 | 0.008540 | 14.83 |
| 1m | rudb-native | q9 | 0.005828 | 0.006742 | 0.000726 | 0.014384 | 0.036250 | 47.84 |
| 1m | rudb-native | q10 | 0.009511 | 0.009527 | 0.001578 | 0.018176 | 0.048819 | 51.98 |
| 1m | rudb-native | q11 | 0.003494 | 0.003210 | 0.000396 | 0.011255 | 0.016050 | 25.23 |
| 1m | rudb-native | q12 | 0.003194 | 0.003580 | 0.000688 | 0.013239 | 0.018261 | 26.23 |
| 1m | rudb-native | q13 | 0.003747 | 0.003420 | 0.000624 | 0.011984 | 0.015089 | 18.09 |
| 1m | rudb-native | q14 | 0.006145 | 0.005950 | 0.000640 | 0.014664 | 0.027847 | 32.92 |
| 1m | rudb-native | q15 | 0.005505 | 0.005593 | 0.000361 | 0.014036 | 0.026647 | 26.86 |
| 1m | rudb-native | q16 | 0.001003 | 0.001039 | 0.000028 | 0.008134 | 0.006851 | 9.97 |
| 1m | rudb-native | q17 | 0.009012 | 0.009133 | 0.000658 | 0.018047 | 0.048656 | 52.44 |
| 1m | rudb-native | q18 | 0.002151 | 0.002277 | 0.000295 | 0.010455 | 0.008967 | 12.70 |
| 1m | rudb-native | q19 | 0.011038 | 0.011519 | 0.001915 | 0.022183 | 0.059198 | 57.16 |
| 1m | rudb-native | q20 | 0.001623 | 0.001271 | 0.000070 | 0.009137 | 0.009020 | 13.02 |
| 1m | rudb-native | q21 | 0.024878 | 0.031669 | 0.002290 | 0.041601 | 0.085795 | 51.20 |
| 1m | rudb-native | q22 | 0.031445 | 0.031680 | 0.000403 | 0.040680 | 0.082633 | 56.17 |
| 1m | rudb-native | q23 | 0.033259 | 0.033585 | 0.000647 | 0.042541 | 0.072351 | 53.02 |
| 1m | rudb-native | q24 | 0.031880 | 0.032459 | 0.001612 | 0.041976 | 0.094683 | 67.52 |
| 1m | rudb-native | q25 | 0.002708 | 0.002735 | 0.000205 | 0.010632 | 0.012088 | 16.45 |
| 1m | rudb-native | q26 | 0.003579 | 0.004057 | 0.000191 | 0.011561 | 0.016955 | 18.67 |
| 1m | rudb-native | q27 | 0.003078 | 0.003015 | 0.000418 | 0.010604 | 0.012291 | 17.47 |
| 1m | rudb-native | q28 | 0.006303 | 0.005359 | 0.000530 | 0.013784 | 0.021081 | 31.23 |
| 1m | rudb-native | q29 | 0.062585 | 0.068132 | 0.001435 | 0.079258 | 0.257610 | 83.33 |
| 1m | rudb-native | q30 | 0.004195 | 0.003114 | 0.000470 | 0.010306 | 0.008923 | 11.22 |
| 1m | rudb-native | q31 | 0.014266 | 0.004847 | 0.000652 | 0.012136 | 0.021957 | 30.47 |
| 1m | rudb-native | q32 | 0.001307 | 0.001425 | 0.000129 | 0.008395 | 0.007046 | 11.09 |
| 1m | rudb-native | q33 | 0.001631 | 0.001110 | 0.000141 | 0.008285 | 0.007039 | 10.58 |
| 1m | rudb-native | q34 | 0.001151 | 0.001150 | 0.000049 | 0.007958 | 0.006828 | 10.23 |
| 1m | rudb-native | q35 | 0.004451 | 0.004880 | 0.000759 | 0.013817 | 0.019105 | 25.27 |
| 1m | rudb-native | q36 | 0.001238 | 0.001088 | 0.000077 | 0.007500 | 0.006379 | 10.38 |
| 1m | rudb-native | q37 | 0.002378 | 0.002149 | 0.000772 | 0.007742 | 0.007461 | 15.92 |
| 1m | rudb-native | q38 | 0.001940 | 0.001856 | 0.000107 | 0.006814 | 0.006715 | 15.38 |
| 1m | rudb-native | q39 | 0.001357 | 0.001345 | 0.000118 | 0.006369 | 0.006249 | 14.80 |
| 1m | rudb-native | q40 | 0.002751 | 0.002960 | 0.000179 | 0.008147 | 0.008572 | 19.17 |
| 1m | rudb-native | q41 | 0.001363 | 0.001363 | 0.000165 | 0.006716 | 0.006782 | 14.31 |
| 1m | rudb-native | q42 | 0.001535 | 0.001412 | 0.000052 | 0.006714 | 0.006653 | 14.14 |
| 1m | rudb-native | q43 | 0.001617 | 0.001528 | 0.000104 | 0.006901 | 0.007041 | 14.20 |
| 1m | duckdb-parquet | q1 | 0.002000 | 0.002000 | 0.001000 | 0.025716 | 0.024006 | 20.55 |
| 1m | duckdb-parquet | q2 | 0.003000 | 0.003000 | 0.001000 | 0.023759 | 0.025793 | 22.44 |
| 1m | duckdb-parquet | q3 | 0.004000 | 0.004000 | 0.000000 | 0.024978 | 0.029142 | 23.80 |
| 1m | duckdb-parquet | q4 | 0.004000 | 0.004000 | 0.001000 | 0.026471 | 0.029799 | 36.41 |
| 1m | duckdb-parquet | q5 | 0.014000 | 0.014000 | 0.001000 | 0.038357 | 0.085555 | 66.72 |
| 1m | duckdb-parquet | q6 | 0.012000 | 0.013000 | 0.001000 | 0.036391 | 0.071395 | 56.70 |
| 1m | duckdb-parquet | q7 | 0.003000 | 0.002000 | 0.001000 | 0.024880 | 0.023423 | 19.33 |
| 1m | duckdb-parquet | q8 | 0.003000 | 0.003000 | 0.000000 | 0.023041 | 0.025298 | 22.55 |
| 1m | duckdb-parquet | q9 | 0.016000 | 0.019000 | 0.001000 | 0.041094 | 0.104612 | 77.69 |
| 1m | duckdb-parquet | q10 | 0.021000 | 0.025000 | 0.004000 | 0.051202 | 0.137158 | 84.59 |
| 1m | duckdb-parquet | q11 | 0.008000 | 0.007000 | 0.000000 | 0.032404 | 0.046293 | 52.61 |
| 1m | duckdb-parquet | q12 | 0.007000 | 0.009000 | 0.001000 | 0.034133 | 0.052150 | 53.98 |
| 1m | duckdb-parquet | q13 | 0.014000 | 0.015000 | 0.001000 | 0.042074 | 0.081048 | 61.66 |
| 1m | duckdb-parquet | q14 | 0.019000 | 0.019000 | 0.000000 | 0.045481 | 0.103226 | 83.78 |
| 1m | duckdb-parquet | q15 | 0.014000 | 0.015000 | 0.001000 | 0.040487 | 0.083262 | 65.19 |
| 1m | duckdb-parquet | q16 | 0.018000 | 0.019000 | 0.002000 | 0.043732 | 0.104164 | 78.64 |
| 1m | duckdb-parquet | q17 | 0.038000 | 0.038000 | 0.002000 | 0.064445 | 0.194969 | 155.23 |
| 1m | duckdb-parquet | q18 | 0.037000 | 0.036000 | 0.006000 | 0.065968 | 0.160231 | 141.47 |
| 1m | duckdb-parquet | q19 | 0.044000 | 0.048000 | 0.004000 | 0.076645 | 0.233831 | 175.25 |
| 1m | duckdb-parquet | q20 | 0.004000 | 0.004000 | 0.000000 | 0.029109 | 0.032129 | 35.58 |
| 1m | duckdb-parquet | q21 | 0.037000 | 0.040000 | 0.002000 | 0.067743 | 0.193382 | 141.61 |
| 1m | duckdb-parquet | q22 | 0.036000 | 0.034000 | 0.002000 | 0.062127 | 0.167605 | 168.50 |
| 1m | duckdb-parquet | q23 | 0.061000 | 0.067000 | 0.010000 | 0.096375 | 0.317775 | 282.02 |
| 1m | duckdb-parquet | q24 | 0.121000 | 0.116000 | 0.011000 | 0.149832 | 0.497835 | 399.03 |
| 1m | duckdb-parquet | q25 | 0.020000 | 0.019000 | 0.003000 | 0.044843 | 0.095630 | 58.64 |
| 1m | duckdb-parquet | q26 | 0.011000 | 0.010000 | 0.001000 | 0.033192 | 0.057004 | 37.92 |
| 1m | duckdb-parquet | q27 | 0.013000 | 0.013000 | 0.002000 | 0.036900 | 0.072572 | 48.17 |
| 1m | duckdb-parquet | q28 | 0.036000 | 0.033000 | 0.002000 | 0.060140 | 0.160625 | 158.91 |
| 1m | duckdb-parquet | q29 | 0.249000 | 0.267000 | 0.020000 | 0.298877 | 1.211907 | 187.03 |
| 1m | duckdb-parquet | q30 | 0.010000 | 0.009000 | 0.003000 | 0.036254 | 0.034459 | 34.50 |
| 1m | duckdb-parquet | q31 | 0.036000 | 0.015000 | 0.004000 | 0.037799 | 0.078051 | 61.12 |
| 1m | duckdb-parquet | q32 | 0.015000 | 0.017000 | 0.002000 | 0.042726 | 0.084617 | 70.14 |
| 1m | duckdb-parquet | q33 | 0.043000 | 0.035000 | 0.012000 | 0.061244 | 0.180807 | 126.48 |
| 1m | duckdb-parquet | q34 | 0.084000 | 0.064000 | 0.002000 | 0.094046 | 0.286510 | 275.11 |
| 1m | duckdb-parquet | q35 | 0.074000 | 0.070000 | 0.019000 | 0.103616 | 0.335985 | 282.38 |
| 1m | duckdb-parquet | q36 | 0.022000 | 0.021000 | 0.002000 | 0.043303 | 0.107519 | 76.05 |
| 1m | duckdb-parquet | q37 | 0.016000 | 0.012000 | 0.002000 | 0.030148 | 0.037458 | 68.91 |
| 1m | duckdb-parquet | q38 | 0.011000 | 0.010000 | 0.000000 | 0.026457 | 0.040468 | 61.08 |
| 1m | duckdb-parquet | q39 | 0.011000 | 0.010000 | 0.000000 | 0.026384 | 0.038707 | 65.14 |
| 1m | duckdb-parquet | q40 | 0.015000 | 0.016000 | 0.001000 | 0.033554 | 0.058481 | 94.03 |
| 1m | duckdb-parquet | q41 | 0.003000 | 0.004000 | 0.001000 | 0.020201 | 0.022176 | 32.03 |
| 1m | duckdb-parquet | q42 | 0.003000 | 0.003000 | 0.001000 | 0.020151 | 0.021879 | 30.98 |
| 1m | duckdb-parquet | q43 | 0.004000 | 0.004000 | 0.001000 | 0.020356 | 0.022700 | 30.66 |
| 1m | rudb-parquet | q1 | 0.002090 | 0.002183 | 0.000322 | 0.009094 | 0.008940 | 12.08 |
| 1m | rudb-parquet | q2 | 0.002860 | 0.002797 | 0.000209 | 0.009089 | 0.011862 | 17.03 |
| 1m | rudb-parquet | q3 | 0.003332 | 0.003241 | 0.000162 | 0.009885 | 0.013801 | 21.03 |
| 1m | rudb-parquet | q4 | 0.003691 | 0.003537 | 0.000397 | 0.010689 | 0.015334 | 31.64 |
| 1m | rudb-parquet | q5 | 0.008253 | 0.008426 | 0.000517 | 0.015704 | 0.041692 | 43.77 |
| 1m | rudb-parquet | q6 | 0.018963 | 0.019222 | 0.001048 | 0.026867 | 0.078853 | 53.84 |
| 1m | rudb-parquet | q7 | 0.001220 | 0.001180 | 0.000039 | 0.007531 | 0.006335 | 9.14 |
| 1m | rudb-parquet | q8 | 0.003165 | 0.002927 | 0.000323 | 0.008792 | 0.012108 | 17.73 |
| 1m | rudb-parquet | q9 | 0.008107 | 0.008839 | 0.001134 | 0.016001 | 0.040935 | 59.86 |
| 1m | rudb-parquet | q10 | 0.012046 | 0.013290 | 0.001702 | 0.021413 | 0.061298 | 64.36 |
| 1m | rudb-parquet | q11 | 0.007960 | 0.008299 | 0.001128 | 0.016177 | 0.034420 | 42.06 |
| 1m | rudb-parquet | q12 | 0.008795 | 0.009496 | 0.000624 | 0.017430 | 0.038773 | 43.77 |
| 1m | rudb-parquet | q13 | 0.016392 | 0.017475 | 0.002438 | 0.026354 | 0.076969 | 51.52 |
| 1m | rudb-parquet | q14 | 0.021550 | 0.021176 | 0.002089 | 0.029406 | 0.096724 | 79.61 |
| 1m | rudb-parquet | q15 | 0.019108 | 0.018000 | 0.001786 | 0.026826 | 0.082209 | 56.61 |
| 1m | rudb-parquet | q16 | 0.007059 | 0.007257 | 0.000836 | 0.014985 | 0.032491 | 48.69 |
| 1m | rudb-parquet | q17 | 0.043720 | 0.045198 | 0.002893 | 0.054220 | 0.190581 | 123.23 |
| 1m | rudb-parquet | q18 | 0.016089 | 0.014964 | 0.001770 | 0.023442 | 0.064891 | 47.44 |
| 1m | rudb-parquet | q19 | 0.062359 | 0.058074 | 0.006046 | 0.068294 | 0.242721 | 152.17 |
| 1m | rudb-parquet | q20 | 0.003571 | 0.003549 | 0.000192 | 0.011036 | 0.014801 | 31.75 |
| 1m | rudb-parquet | q21 | 0.053888 | 0.059613 | 0.004776 | 0.071725 | 0.237302 | 189.05 |
| 1m | rudb-parquet | q22 | 0.057620 | 0.057357 | 0.000660 | 0.067436 | 0.227475 | 176.08 |
| 1m | rudb-parquet | q23 | 0.116807 | 0.119797 | 0.005088 | 0.132856 | 0.496824 | 275.67 |
| 1m | rudb-parquet | q24 | 0.127617 | 0.122342 | 0.004370 | 0.137379 | 0.694170 | 334.78 |
| 1m | rudb-parquet | q25 | 0.014551 | 0.014711 | 0.000762 | 0.023025 | 0.060340 | 48.84 |
| 1m | rudb-parquet | q26 | 0.011270 | 0.010869 | 0.001208 | 0.018191 | 0.047117 | 31.91 |
| 1m | rudb-parquet | q27 | 0.013717 | 0.014044 | 0.002291 | 0.021296 | 0.057154 | 47.75 |
| 1m | rudb-parquet | q28 | 0.051249 | 0.048399 | 0.003359 | 0.059047 | 0.197818 | 168.38 |
| 1m | rudb-parquet | q29 | 0.098105 | 0.104393 | 0.004647 | 0.115910 | 0.426591 | 192.89 |
| 1m | rudb-parquet | q30 | 0.006055 | 0.006075 | 0.001029 | 0.013027 | 0.016492 | 18.42 |
| 1m | rudb-parquet | q31 | 0.014394 | 0.013948 | 0.003242 | 0.021757 | 0.058821 | 55.53 |
| 1m | rudb-parquet | q32 | 0.014623 | 0.016679 | 0.001812 | 0.026966 | 0.067108 | 70.19 |
| 1m | rudb-parquet | q33 | 0.012920 | 0.013636 | 0.001999 | 0.020962 | 0.059815 | 69.39 |
| 1m | rudb-parquet | q34 | 0.087410 | 0.084507 | 0.007862 | 0.098314 | 0.357450 | 276.81 |
| 1m | rudb-parquet | q35 | 0.089834 | 0.097758 | 0.010696 | 0.112456 | 0.385068 | 269.89 |
| 1m | rudb-parquet | q36 | 0.007231 | 0.006393 | 0.000738 | 0.013267 | 0.030590 | 40.97 |
| 1m | rudb-parquet | q37 | 0.030411 | 0.023072 | 0.003918 | 0.029328 | 0.042086 | 64.62 |
| 1m | rudb-parquet | q38 | 0.028136 | 0.027141 | 0.000769 | 0.032083 | 0.051521 | 58.97 |
| 1m | rudb-parquet | q39 | 0.020210 | 0.020231 | 0.000441 | 0.025365 | 0.036736 | 62.69 |
| 1m | rudb-parquet | q40 | 0.033321 | 0.034278 | 0.000538 | 0.039688 | 0.067405 | 98.19 |
| 1m | rudb-parquet | q41 | 0.003517 | 0.003903 | 0.000529 | 0.009460 | 0.010713 | 28.11 |
| 1m | rudb-parquet | q42 | 0.003152 | 0.003275 | 0.000235 | 0.008339 | 0.009426 | 25.44 |
| 1m | rudb-parquet | q43 | 0.003722 | 0.003685 | 0.000123 | 0.008618 | 0.009998 | 22.70 |
| 10m | duckdb-native | q1 | 0.001000 | 0.002000 | 0.000000 | 0.022830 | 0.022476 | 21.02 |
| 10m | duckdb-native | q2 | 0.015000 | 0.004000 | 0.000000 | 0.026048 | 0.029962 | 27.77 |
| 10m | duckdb-native | q3 | 0.019000 | 0.012000 | 0.000000 | 0.035993 | 0.074464 | 35.16 |
| 10m | duckdb-native | q4 | 0.027000 | 0.012000 | 0.001000 | 0.036358 | 0.073637 | 41.75 |
| 10m | duckdb-native | q5 | 0.057000 | 0.049000 | 0.004000 | 0.076408 | 0.285686 | 103.45 |
| 10m | duckdb-native | q6 | 0.117000 | 0.089000 | 0.003000 | 0.124460 | 0.508745 | 249.67 |
| 10m | duckdb-native | q7 | 0.004000 | 0.003000 | 0.001000 | 0.026107 | 0.024971 | 21.38 |
| 10m | duckdb-native | q8 | 0.007000 | 0.005000 | 0.000000 | 0.026449 | 0.035272 | 28.97 |
| 10m | duckdb-native | q9 | 0.063000 | 0.064000 | 0.003000 | 0.092777 | 0.374644 | 127.53 |
| 10m | duckdb-native | q10 | 0.120000 | 0.126000 | 0.010000 | 0.161225 | 0.680372 | 145.05 |
| 10m | duckdb-native | q11 | 0.026000 | 0.020000 | 0.001000 | 0.046269 | 0.123652 | 72.05 |
| 10m | duckdb-native | q12 | 0.024000 | 0.023000 | 0.002000 | 0.049704 | 0.134238 | 77.70 |
| 10m | duckdb-native | q13 | 0.081000 | 0.080000 | 0.004000 | 0.116674 | 0.460631 | 250.52 |
| 10m | duckdb-native | q14 | 0.128000 | 0.129000 | 0.007000 | 0.168748 | 0.725618 | 364.28 |
| 10m | duckdb-native | q15 | 0.102000 | 0.083000 | 0.012000 | 0.114877 | 0.469431 | 259.94 |
| 10m | duckdb-native | q16 | 0.061000 | 0.056000 | 0.004000 | 0.081602 | 0.322751 | 122.05 |
| 10m | duckdb-native | q17 | 0.130000 | 0.135000 | 0.032000 | 0.169059 | 0.760450 | 340.75 |
| 10m | duckdb-native | q18 | 0.123000 | 0.130000 | 0.071000 | 0.172723 | 0.691954 | 305.42 |
| 10m | duckdb-native | q19 | 0.597000 | 0.381000 | 0.109000 | 0.462790 | 1.618366 | 710.06 |
| 10m | duckdb-native | q20 | 0.006000 | 0.005000 | 0.003000 | 0.040601 | 0.040911 | 34.12 |
| 10m | duckdb-native | q21 | 0.438000 | 0.199000 | 0.034000 | 0.290808 | 0.885319 | 444.09 |
| 10m | duckdb-native | q22 | 0.136000 | 0.162000 | 0.045000 | 0.298925 | 0.738353 | 493.92 |
| 10m | duckdb-native | q23 | 0.506000 | 0.444000 | 0.239000 | 0.626279 | 1.386095 | 864.81 |
| 10m | duckdb-native | q24 | 0.181000 | 0.154000 | 0.040000 | 0.234362 | 0.736946 | 512.56 |
| 10m | duckdb-native | q25 | 0.011000 | 0.009000 | 0.001000 | 0.032671 | 0.054335 | 41.38 |
| 10m | duckdb-native | q26 | 0.036000 | 0.036000 | 0.002000 | 0.061600 | 0.207019 | 76.22 |
| 10m | duckdb-native | q27 | 0.009000 | 0.009000 | 0.000000 | 0.032530 | 0.053719 | 39.97 |
| 10m | duckdb-native | q28 | 0.172000 | 0.127000 | 0.048000 | 0.210970 | 0.637797 | 453.22 |
| 10m | duckdb-native | q29 | 2.247000 | 2.305000 | 0.769000 | 2.409693 | 11.068508 | 676.91 |
| 10m | duckdb-native | q30 | 0.027000 | 0.020000 | 0.001000 | 0.055660 | 0.082924 | 41.62 |
| 10m | duckdb-native | q31 | 0.115000 | 0.089000 | 0.020000 | 0.140496 | 0.467083 | 190.69 |
| 10m | duckdb-native | q32 | 0.136000 | 0.120000 | 0.011000 | 0.183520 | 0.594655 | 321.47 |
| 10m | duckdb-native | q33 | 0.601000 | 0.546000 | 0.136000 | 0.668552 | 1.980674 | 840.73 |
| 10m | duckdb-native | q34 | 1.310000 | 0.525000 | 0.039000 | 0.701430 | 2.102596 | 1100.91 |
| 10m | duckdb-native | q35 | 0.791000 | 0.587000 | 0.244000 | 0.757822 | 2.280913 | 1144.39 |
| 10m | duckdb-native | q36 | 0.087000 | 0.090000 | 0.022000 | 0.122674 | 0.493345 | 116.09 |
| 10m | duckdb-native | q37 | 0.034000 | 0.044000 | 0.018000 | 0.082009 | 0.187852 | 135.73 |
| 10m | duckdb-native | q38 | 0.014000 | 0.011000 | 0.004000 | 0.048455 | 0.066944 | 48.48 |
| 10m | duckdb-native | q39 | 0.024000 | 0.016000 | 0.012000 | 0.048545 | 0.081173 | 68.67 |
| 10m | duckdb-native | q40 | 0.081000 | 0.079000 | 0.027000 | 0.123326 | 0.361807 | 240.70 |
| 10m | duckdb-native | q41 | 0.015000 | 0.009000 | 0.000000 | 0.038261 | 0.054449 | 46.34 |
| 10m | duckdb-native | q42 | 0.008000 | 0.009000 | 0.001000 | 0.039016 | 0.053476 | 41.44 |
| 10m | duckdb-native | q43 | 0.012000 | 0.011000 | 0.002000 | 0.040840 | 0.063372 | 37.28 |
| 10m | rudb-native | q1 | 0.001750 | 0.000817 | 0.000235 | 0.008951 | 0.007771 | 12.28 |
| 10m | rudb-native | q2 | 0.001836 | 0.000938 | 0.000099 | 0.009224 | 0.008025 | 12.19 |
| 10m | rudb-native | q3 | 0.001254 | 0.000921 | 0.000147 | 0.009658 | 0.008534 | 12.41 |
| 10m | rudb-native | q4 | 0.000981 | 0.000877 | 0.000076 | 0.009568 | 0.008448 | 11.75 |
| 10m | rudb-native | q5 | 0.000782 | 0.000893 | 0.000103 | 0.010060 | 0.008821 | 10.86 |
| 10m | rudb-native | q6 | 0.000894 | 0.000967 | 0.000212 | 0.010455 | 0.009254 | 11.81 |
| 10m | rudb-native | q7 | 0.000860 | 0.000890 | 0.000140 | 0.009482 | 0.008228 | 11.31 |
| 10m | rudb-native | q8 | 0.005465 | 0.003422 | 0.002429 | 0.013500 | 0.019985 | 27.12 |
| 10m | rudb-native | q9 | 0.024196 | 0.024274 | 0.001157 | 0.036482 | 0.130534 | 73.58 |
| 10m | rudb-native | q10 | 0.033735 | 0.035082 | 0.004027 | 0.049223 | 0.179617 | 82.45 |
| 10m | rudb-native | q11 | 0.007462 | 0.007817 | 0.000504 | 0.018631 | 0.046154 | 36.67 |
| 10m | rudb-native | q12 | 0.008958 | 0.009371 | 0.000826 | 0.019844 | 0.053397 | 38.33 |
| 10m | rudb-native | q13 | 0.009631 | 0.009508 | 0.000290 | 0.020640 | 0.048593 | 37.72 |
| 10m | rudb-native | q14 | 0.027446 | 0.025470 | 0.003669 | 0.038264 | 0.138857 | 85.09 |
| 10m | rudb-native | q15 | 0.019772 | 0.020227 | 0.003246 | 0.031811 | 0.107709 | 80.77 |
| 10m | rudb-native | q16 | 0.001176 | 0.001053 | 0.000117 | 0.009202 | 0.008165 | 12.00 |
| 10m | rudb-native | q17 | 0.032666 | 0.033843 | 0.003289 | 0.046322 | 0.192353 | 124.80 |
| 10m | rudb-native | q18 | 0.004578 | 0.004690 | 0.001910 | 0.014941 | 0.013328 | 21.69 |
| 10m | rudb-native | q19 | 0.092749 | 0.081280 | 0.020140 | 0.102938 | 0.425895 | 225.97 |
| 10m | rudb-native | q20 | 0.005029 | 0.001806 | 0.001075 | 0.017316 | 0.015121 | 14.84 |
| 10m | rudb-native | q21 | 0.086853 | 0.069554 | 0.007838 | 0.086657 | 0.237406 | 78.02 |
| 10m | rudb-native | q22 | 0.085276 | 0.083913 | 0.019194 | 0.101132 | 0.257097 | 99.59 |
| 10m | rudb-native | q23 | 0.289652 | 0.190181 | 0.038469 | 0.204347 | 0.467976 | 188.50 |
| 10m | rudb-native | q24 | 0.117377 | 0.066596 | 0.003196 | 0.080322 | 0.217028 | 110.53 |
| 10m | rudb-native | q25 | 0.008459 | 0.003631 | 0.000231 | 0.013258 | 0.015563 | 21.28 |
| 10m | rudb-native | q26 | 0.076144 | 0.012560 | 0.000356 | 0.022853 | 0.052537 | 34.83 |
| 10m | rudb-native | q27 | 0.006827 | 0.006670 | 0.000712 | 0.015934 | 0.018347 | 24.06 |
| 10m | rudb-native | q28 | 0.047552 | 0.027015 | 0.009631 | 0.039761 | 0.107824 | 69.11 |
| 10m | rudb-native | q29 | 0.651838 | 0.557844 | 0.357409 | 0.587801 | 1.935627 | 286.94 |
| 10m | rudb-native | q30 | 0.005260 | 0.004896 | 0.000447 | 0.017381 | 0.015395 | 14.38 |
| 10m | rudb-native | q31 | 0.075090 | 0.034624 | 0.001474 | 0.050477 | 0.182449 | 96.03 |
| 10m | rudb-native | q32 | 0.002556 | 0.002078 | 0.000285 | 0.013790 | 0.011886 | 15.86 |
| 10m | rudb-native | q33 | 0.001391 | 0.001445 | 0.000141 | 0.016015 | 0.012727 | 12.58 |
| 10m | rudb-native | q34 | 0.001786 | 0.001849 | 0.000307 | 0.016515 | 0.013073 | 13.30 |
| 10m | rudb-native | q35 | 0.073225 | 0.032635 | 0.011778 | 0.052022 | 0.129025 | 96.72 |
| 10m | rudb-native | q36 | 0.002307 | 0.001435 | 0.000306 | 0.013969 | 0.011880 | 13.31 |
| 10m | rudb-native | q37 | 0.018330 | 0.012869 | 0.002222 | 0.028639 | 0.035167 | 42.11 |
| 10m | rudb-native | q38 | 0.020643 | 0.009026 | 0.000778 | 0.025162 | 0.035710 | 36.36 |
| 10m | rudb-native | q39 | 0.010463 | 0.008719 | 0.003487 | 0.024997 | 0.025279 | 28.95 |
| 10m | rudb-native | q40 | 0.045329 | 0.045666 | 0.008784 | 0.064617 | 0.175863 | 92.22 |
| 10m | rudb-native | q41 | 0.016823 | 0.008618 | 0.001287 | 0.022275 | 0.031142 | 41.02 |
| 10m | rudb-native | q42 | 0.009677 | 0.007629 | 0.001618 | 0.022038 | 0.028306 | 31.36 |
| 10m | rudb-native | q43 | 0.007327 | 0.006420 | 0.001561 | 0.019170 | 0.030962 | 26.72 |
| 10m | duckdb-parquet | q1 | 0.016000 | 0.017000 | 0.001000 | 0.049099 | 0.048586 | 34.56 |
| 10m | duckdb-parquet | q2 | 0.025000 | 0.024000 | 0.001000 | 0.057343 | 0.082395 | 37.81 |
| 10m | duckdb-parquet | q3 | 0.030000 | 0.030000 | 0.002000 | 0.066233 | 0.112142 | 36.42 |
| 10m | duckdb-parquet | q4 | 0.031000 | 0.030000 | 0.001000 | 0.067797 | 0.111462 | 55.98 |
| 10m | duckdb-parquet | q5 | 0.058000 | 0.060000 | 0.004000 | 0.100017 | 0.271705 | 115.09 |
| 10m | duckdb-parquet | q6 | 0.132000 | 0.127000 | 0.014000 | 0.174956 | 0.609031 | 253.98 |
| 10m | duckdb-parquet | q7 | 0.020000 | 0.019000 | 0.003000 | 0.054854 | 0.054001 | 36.45 |
| 10m | duckdb-parquet | q8 | 0.023000 | 0.025000 | 0.001000 | 0.056764 | 0.084498 | 39.91 |
| 10m | duckdb-parquet | q9 | 0.077000 | 0.084000 | 0.005000 | 0.124657 | 0.416236 | 126.81 |
| 10m | duckdb-parquet | q10 | 0.120000 | 0.134000 | 0.005000 | 0.177088 | 0.648040 | 142.27 |
| 10m | duckdb-parquet | q11 | 0.041000 | 0.039000 | 0.009000 | 0.079419 | 0.160758 | 74.52 |
| 10m | duckdb-parquet | q12 | 0.042000 | 0.042000 | 0.005000 | 0.081658 | 0.182832 | 75.30 |
| 10m | duckdb-parquet | q13 | 0.146000 | 0.128000 | 0.006000 | 0.172510 | 0.638509 | 275.03 |
| 10m | duckdb-parquet | q14 | 0.180000 | 0.178000 | 0.016000 | 0.223719 | 0.892724 | 339.88 |
| 10m | duckdb-parquet | q15 | 0.134000 | 0.139000 | 0.006000 | 0.183719 | 0.671835 | 266.00 |
| 10m | duckdb-parquet | q16 | 0.063000 | 0.064000 | 0.004000 | 0.101436 | 0.302913 | 131.92 |
| 10m | duckdb-parquet | q17 | 0.180000 | 0.177000 | 0.028000 | 0.218833 | 0.910345 | 331.34 |
| 10m | duckdb-parquet | q18 | 0.167000 | 0.194000 | 0.069000 | 0.242810 | 0.860457 | 318.62 |
| 10m | duckdb-parquet | q19 | 0.708000 | 0.409000 | 0.048000 | 0.484315 | 1.871152 | 658.61 |
| 10m | duckdb-parquet | q20 | 0.042000 | 0.036000 | 0.010000 | 0.088271 | 0.088495 | 50.25 |
| 10m | duckdb-parquet | q21 | 0.669000 | 0.323000 | 0.148000 | 0.377174 | 1.280961 | 319.81 |
| 10m | duckdb-parquet | q22 | 0.217000 | 0.264000 | 0.099000 | 0.321381 | 1.133925 | 405.83 |
| 10m | duckdb-parquet | q23 | 0.523000 | 0.463000 | 0.090000 | 0.523368 | 2.325865 | 652.59 |
| 10m | duckdb-parquet | q24 | 0.279000 | 0.302000 | 0.035000 | 0.358918 | 1.469837 | 458.59 |
| 10m | duckdb-parquet | q25 | 0.079000 | 0.077000 | 0.003000 | 0.111959 | 0.298782 | 115.72 |
| 10m | duckdb-parquet | q26 | 0.079000 | 0.080000 | 0.006000 | 0.115755 | 0.373764 | 99.23 |
| 10m | duckdb-parquet | q27 | 0.068000 | 0.063000 | 0.003000 | 0.099703 | 0.288027 | 108.30 |
| 10m | duckdb-parquet | q28 | 0.165000 | 0.243000 | 0.219000 | 0.291326 | 1.079914 | 402.66 |
| 10m | duckdb-parquet | q29 | 6.000000 | 3.793000 | 1.355000 | 3.856998 | 13.961153 | 605.53 |
| 10m | duckdb-parquet | q30 | 0.047000 | 0.042000 | 0.009000 | 0.090966 | 0.120924 | 52.17 |
| 10m | duckdb-parquet | q31 | 0.174000 | 0.162000 | 0.008000 | 0.214671 | 0.714533 | 172.31 |
| 10m | duckdb-parquet | q32 | 0.184000 | 0.174000 | 0.027000 | 0.223682 | 0.802094 | 230.05 |
| 10m | duckdb-parquet | q33 | 0.546000 | 0.514000 | 0.060000 | 0.593072 | 1.981994 | 755.50 |
| 10m | duckdb-parquet | q34 | 0.632000 | 0.631000 | 0.100000 | 0.697050 | 2.511236 | 952.33 |
| 10m | duckdb-parquet | q35 | 0.778000 | 0.650000 | 0.227000 | 0.735384 | 2.656746 | 1013.48 |
| 10m | duckdb-parquet | q36 | 0.117000 | 0.121000 | 0.013000 | 0.172812 | 0.547192 | 118.88 |
| 10m | duckdb-parquet | q37 | 0.092000 | 0.097000 | 0.027000 | 0.154824 | 0.278485 | 203.72 |
| 10m | duckdb-parquet | q38 | 0.064000 | 0.056000 | 0.024000 | 0.115870 | 0.127298 | 60.66 |
| 10m | duckdb-parquet | q39 | 0.080000 | 0.053000 | 0.016000 | 0.106564 | 0.168376 | 139.34 |
| 10m | duckdb-parquet | q40 | 0.118000 | 0.117000 | 0.019000 | 0.176804 | 0.457130 | 316.34 |
| 10m | duckdb-parquet | q41 | 0.044000 | 0.035000 | 0.004000 | 0.083392 | 0.102344 | 61.69 |
| 10m | duckdb-parquet | q42 | 0.053000 | 0.038000 | 0.010000 | 0.082308 | 0.101111 | 54.97 |
| 10m | duckdb-parquet | q43 | 0.038000 | 0.045000 | 0.011000 | 0.093681 | 0.123427 | 53.36 |
| 10m | rudb-parquet | q1 | 0.012777 | 0.011261 | 0.000903 | 0.017299 | 0.019326 | 20.98 |
| 10m | rudb-parquet | q2 | 0.019069 | 0.018165 | 0.000339 | 0.024826 | 0.057687 | 28.20 |
| 10m | rudb-parquet | q3 | 0.021782 | 0.020187 | 0.000495 | 0.027952 | 0.065146 | 35.62 |
| 10m | rudb-parquet | q4 | 0.022066 | 0.021511 | 0.001181 | 0.029696 | 0.071161 | 47.17 |
| 10m | rudb-parquet | q5 | 0.061087 | 0.063529 | 0.007306 | 0.074687 | 0.295558 | 151.34 |
| 10m | rudb-parquet | q6 | 0.222389 | 0.200319 | 0.008356 | 0.211506 | 0.866837 | 203.41 |
| 10m | rudb-parquet | q7 | 0.006697 | 0.006057 | 0.000610 | 0.012845 | 0.011644 | 15.00 |
| 10m | rudb-parquet | q8 | 0.018591 | 0.018462 | 0.001432 | 0.025864 | 0.058692 | 28.12 |
| 10m | rudb-parquet | q9 | 0.044379 | 0.048320 | 0.004197 | 0.057056 | 0.213471 | 85.03 |
| 10m | rudb-parquet | q10 | 0.063629 | 0.071130 | 0.003610 | 0.080555 | 0.323964 | 95.66 |
| 10m | rudb-parquet | q11 | 0.052251 | 0.047415 | 0.001887 | 0.055034 | 0.212137 | 55.38 |
| 10m | rudb-parquet | q12 | 0.052325 | 0.052523 | 0.001062 | 0.061143 | 0.239894 | 61.56 |
| 10m | rudb-parquet | q13 | 0.142223 | 0.146022 | 0.006151 | 0.157847 | 0.719556 | 212.97 |
| 10m | rudb-parquet | q14 | 0.175553 | 0.180244 | 0.003422 | 0.195687 | 0.902231 | 295.47 |
| 10m | rudb-parquet | q15 | 0.159767 | 0.153666 | 0.004728 | 0.167385 | 0.776008 | 215.12 |
| 10m | rudb-parquet | q16 | 0.031160 | 0.030952 | 0.002800 | 0.038874 | 0.123914 | 76.27 |
| 10m | rudb-parquet | q17 | 0.255437 | 0.261921 | 0.010121 | 0.279949 | 1.240335 | 400.50 |
| 10m | rudb-parquet | q18 | 0.110915 | 0.139319 | 0.040618 | 0.151089 | 0.594411 | 121.38 |
| 10m | rudb-parquet | q19 | 0.616945 | 0.527106 | 0.164979 | 0.560486 | 2.286942 | 668.25 |
| 10m | rudb-parquet | q20 | 0.021345 | 0.025819 | 0.010753 | 0.037791 | 0.053792 | 46.69 |
| 10m | rudb-parquet | q21 | 0.686413 | 0.395368 | 0.107571 | 0.411819 | 1.753165 | 323.22 |
| 10m | rudb-parquet | q22 | 0.343998 | 0.474797 | 0.229532 | 0.496417 | 1.863024 | 348.78 |
| 10m | rudb-parquet | q23 | 1.416629 | 0.838173 | 0.128898 | 0.860971 | 4.341839 | 494.39 |
| 10m | rudb-parquet | q24 | 0.388696 | 0.435189 | 0.066385 | 0.458608 | 2.401414 | 366.33 |
| 10m | rudb-parquet | q25 | 0.111778 | 0.113416 | 0.009483 | 0.122253 | 0.556109 | 148.02 |
| 10m | rudb-parquet | q26 | 0.085532 | 0.089178 | 0.008891 | 0.097545 | 0.423099 | 138.95 |
| 10m | rudb-parquet | q27 | 0.119580 | 0.109728 | 0.004561 | 0.119093 | 0.550695 | 150.41 |
| 10m | rudb-parquet | q28 | 0.232289 | 0.279310 | 0.193201 | 0.294855 | 1.424698 | 306.14 |
| 10m | rudb-parquet | q29 | 1.506035 | 1.044998 | 0.448941 | 1.080966 | 4.193609 | 755.78 |
| 10m | rudb-parquet | q30 | 0.025440 | 0.029248 | 0.006056 | 0.041921 | 0.064404 | 31.48 |
| 10m | rudb-parquet | q31 | 0.184530 | 0.145270 | 0.025336 | 0.161465 | 0.698714 | 156.11 |
| 10m | rudb-parquet | q32 | 0.144252 | 0.145270 | 0.008601 | 0.160940 | 0.714490 | 172.31 |
| 10m | rudb-parquet | q33 | 0.117156 | 0.130147 | 0.008774 | 0.150226 | 0.602540 | 238.20 |
| 10m | rudb-parquet | q34 | 1.059560 | 0.986450 | 0.342598 | 1.021480 | 3.526245 | 813.58 |
| 10m | rudb-parquet | q35 | 1.110732 | 0.887418 | 0.064624 | 0.926782 | 3.407502 | 816.97 |
| 10m | rudb-parquet | q36 | 0.044598 | 0.037638 | 0.002921 | 0.049065 | 0.146702 | 71.25 |
| 10m | rudb-parquet | q37 | 0.080816 | 0.100723 | 0.033108 | 0.120778 | 0.303828 | 196.62 |
| 10m | rudb-parquet | q38 | 0.037217 | 0.040477 | 0.009234 | 0.055993 | 0.121851 | 60.27 |
| 10m | rudb-parquet | q39 | 0.072546 | 0.065671 | 0.029146 | 0.084345 | 0.180304 | 150.14 |
| 10m | rudb-parquet | q40 | 0.200169 | 0.173128 | 0.029205 | 0.196121 | 0.653242 | 352.80 |
| 10m | rudb-parquet | q41 | 0.026361 | 0.025708 | 0.003542 | 0.037022 | 0.064721 | 64.61 |
| 10m | rudb-parquet | q42 | 0.026694 | 0.028354 | 0.008735 | 0.039533 | 0.060611 | 58.03 |
| 10m | rudb-parquet | q43 | 0.027480 | 0.031454 | 0.007532 | 0.041717 | 0.076100 | 54.11 |
