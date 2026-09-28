# ClickBench measurement audit

All 43 SQL queries are attempted on DuckDB native, rudb native, DuckDB Parquet, and rudb Parquet. 7 hot repetitions are required for a complete row. A failed repetition invalidates that query; successful fragments are never averaged into a result.

First execution is not disk-cold: the page cache is not flushed. Each repetition uses a fresh process. Query seconds are the CLI timer, including result rendering; wall and CPU seconds and peak RSS cover the whole child process. CPU and RSS come from wait4 for that child. RSS is the maximum resident set, not allocated bytes or an incremental memory delta. Both native rows open a loaded single-file database. Both Parquet rows query the same source file with native mirroring disabled; metadata-only paths remain available. These are sample results, not official ClickBench scores.

| Size | Engine | Load wall (s) | Load CPU (s) | Load peak RSS (MiB) | Native bytes |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 0.082044 | 0.068428 | 59.90 | 1060864 |
| 1k | rudb-native | 0.133508 | 0.154316 | 41.29 | 1876635 |
| 10k | duckdb-native | 0.105717 | 0.158980 | 72.50 | 4468736 |
| 10k | rudb-native | 0.302277 | 0.435624 | 77.46 | 7732602 |
| 1m | duckdb-native | 2.806568 | 10.313536 | 1810.40 | 603729920 |
| 1m | rudb-native | 2.366018 | 12.988914 | 1132.38 | 246881101 |
| 10m | duckdb-native | 22.979539 | 64.130999 | 4637.11 | 2906402816 |
| 10m | rudb-native | 13.737715 | 105.100684 | 5083.52 | 2029080948 |

| Size | Engine | Complete / 43 | Query median sum (s) | Process wall median sum (s) | CPU median sum (s) | Peak RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 43 | 0.056000 | 0.461650 | 0.568834 | 33.30 |
| 1k | rudb-native | 43 | 0.023487 | 0.090980 | 0.090240 | 14.27 |
| 1k | duckdb-parquet | 43 | 0.079000 | 0.480923 | 0.590043 | 32.93 |
| 1k | rudb-parquet | 43 | 0.037372 | 0.090082 | 0.088814 | 13.80 |
| 10k | duckdb-native | 43 | 0.083000 | 0.507366 | 0.647796 | 36.70 |
| 10k | rudb-native | 43 | 0.033919 | 0.124443 | 0.126664 | 15.86 |
| 10k | duckdb-parquet | 43 | 0.118000 | 0.544605 | 0.692869 | 37.39 |
| 10k | rudb-parquet | 43 | 0.093617 | 0.149629 | 0.148673 | 15.68 |
| 1m | duckdb-native | 43 | 0.622000 | 1.082505 | 3.194817 | 219.51 |
| 1m | rudb-native | 43 | 0.171163 | 0.271539 | 0.651989 | 115.47 |
| 1m | duckdb-parquet | 43 | 0.897000 | 1.374676 | 4.214148 | 360.91 |
| 1m | rudb-parquet | 43 | 1.423167 | 1.518593 | 5.771202 | 310.05 |
| 10m | duckdb-native | 43 | 3.883000 | 4.564647 | 22.570987 | 1499.73 |
| 10m | rudb-native | 43 | 0.903933 | 1.101479 | 4.292803 | 293.71 |
| 10m | duckdb-parquet | 43 | 10.317000 | 16.008584 | 37.550136 | 1238.29 |
| 10m | rudb-parquet | 43 | 9.051918 | 9.322352 | 27.522842 | 1087.82 |

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
| 10m | q19 | original selections differ; deterministic retest match |
| 10m | q20 | original match |
| 10m | q21 | original match |
| 10m | q22 | original selections differ; deterministic retest match |
| 10m | q23 | original selections differ; deterministic retest match |
| 10m | q24 | original match |
| 10m | q25 | same rows; different order |
| 10m | q26 | original match |
| 10m | q27 | original match |
| 10m | q28 | original match |
| 10m | q29 | original match |
| 10m | q30 | original match |
| 10m | q31 | original selections differ; deterministic retest match |
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
| 1k | duckdb-native | q1 | 0.000000 | 0.000000 | 0.000000 | 0.011158 | 0.013034 | 26.41 |
| 1k | duckdb-native | q2 | 0.000000 | 0.001000 | 0.001000 | 0.010410 | 0.011949 | 27.38 |
| 1k | duckdb-native | q3 | 0.001000 | 0.001000 | 0.001000 | 0.009566 | 0.010875 | 27.41 |
| 1k | duckdb-native | q4 | 0.001000 | 0.000000 | 0.000000 | 0.009357 | 0.010758 | 26.90 |
| 1k | duckdb-native | q5 | 0.001000 | 0.001000 | 0.000000 | 0.010023 | 0.012371 | 28.63 |
| 1k | duckdb-native | q6 | 0.001000 | 0.001000 | 0.000000 | 0.009924 | 0.012177 | 28.58 |
| 1k | duckdb-native | q7 | 0.000000 | 0.000000 | 0.000000 | 0.009246 | 0.010585 | 26.91 |
| 1k | duckdb-native | q8 | 0.001000 | 0.001000 | 0.000000 | 0.009758 | 0.011543 | 28.92 |
| 1k | duckdb-native | q9 | 0.001000 | 0.001000 | 0.001000 | 0.010425 | 0.013993 | 32.17 |
| 1k | duckdb-native | q10 | 0.002000 | 0.002000 | 0.001000 | 0.010919 | 0.014905 | 33.30 |
| 1k | duckdb-native | q11 | 0.001000 | 0.001000 | 0.001000 | 0.010607 | 0.014121 | 32.42 |
| 1k | duckdb-native | q12 | 0.001000 | 0.002000 | 0.001000 | 0.010761 | 0.014337 | 32.67 |
| 1k | duckdb-native | q13 | 0.001000 | 0.001000 | 0.000000 | 0.010491 | 0.012821 | 29.67 |
| 1k | duckdb-native | q14 | 0.002000 | 0.002000 | 0.001000 | 0.010856 | 0.014897 | 32.93 |
| 1k | duckdb-native | q15 | 0.002000 | 0.001000 | 0.000000 | 0.010191 | 0.012652 | 30.31 |
| 1k | duckdb-native | q16 | 0.001000 | 0.001000 | 0.001000 | 0.010192 | 0.012536 | 31.01 |
| 1k | duckdb-native | q17 | 0.002000 | 0.001000 | 0.001000 | 0.010331 | 0.012948 | 31.73 |
| 1k | duckdb-native | q18 | 0.001000 | 0.001000 | 0.001000 | 0.010257 | 0.012954 | 31.16 |
| 1k | duckdb-native | q19 | 0.004000 | 0.003000 | 0.001000 | 0.021311 | 0.024305 | 32.61 |
| 1k | duckdb-native | q20 | 0.001000 | 0.000000 | 0.001000 | 0.010020 | 0.011366 | 26.91 |
| 1k | duckdb-native | q21 | 0.001000 | 0.001000 | 0.001000 | 0.009653 | 0.011239 | 27.36 |
| 1k | duckdb-native | q22 | 0.001000 | 0.001000 | 0.000000 | 0.010279 | 0.012400 | 28.67 |
| 1k | duckdb-native | q23 | 0.001000 | 0.001000 | 0.001000 | 0.010273 | 0.012278 | 28.92 |
| 1k | duckdb-native | q24 | 0.004000 | 0.004000 | 0.001000 | 0.013467 | 0.017445 | 33.19 |
| 1k | duckdb-native | q25 | 0.002000 | 0.001000 | 0.001000 | 0.010564 | 0.012762 | 29.54 |
| 1k | duckdb-native | q26 | 0.001000 | 0.000000 | 0.001000 | 0.009911 | 0.011138 | 27.41 |
| 1k | duckdb-native | q27 | 0.000000 | 0.001000 | 0.000000 | 0.009861 | 0.011464 | 27.66 |
| 1k | duckdb-native | q28 | 0.001000 | 0.001000 | 0.001000 | 0.010673 | 0.013622 | 30.17 |
| 1k | duckdb-native | q29 | 0.001000 | 0.002000 | 0.000000 | 0.010981 | 0.014427 | 30.53 |
| 1k | duckdb-native | q30 | 0.003000 | 0.004000 | 0.001000 | 0.012859 | 0.015050 | 30.79 |
| 1k | duckdb-native | q31 | 0.001000 | 0.002000 | 0.001000 | 0.010779 | 0.013619 | 32.29 |
| 1k | duckdb-native | q32 | 0.001000 | 0.002000 | 0.001000 | 0.010798 | 0.013628 | 31.75 |
| 1k | duckdb-native | q33 | 0.001000 | 0.001000 | 0.000000 | 0.010633 | 0.013051 | 31.97 |
| 1k | duckdb-native | q34 | 0.001000 | 0.001000 | 0.001000 | 0.010509 | 0.013270 | 30.42 |
| 1k | duckdb-native | q35 | 0.002000 | 0.002000 | 0.001000 | 0.010460 | 0.013506 | 30.67 |
| 1k | duckdb-native | q36 | 0.001000 | 0.001000 | 0.001000 | 0.010563 | 0.013284 | 31.92 |
| 1k | duckdb-native | q37 | 0.002000 | 0.001000 | 0.001000 | 0.010493 | 0.013072 | 30.31 |
| 1k | duckdb-native | q38 | 0.002000 | 0.001000 | 0.001000 | 0.010619 | 0.013105 | 29.82 |
| 1k | duckdb-native | q39 | 0.001000 | 0.001000 | 0.000000 | 0.010027 | 0.011759 | 28.29 |
| 1k | duckdb-native | q40 | 0.002000 | 0.002000 | 0.001000 | 0.010928 | 0.014326 | 30.68 |
| 1k | duckdb-native | q41 | 0.002000 | 0.001000 | 0.001000 | 0.010454 | 0.013070 | 31.10 |
| 1k | duckdb-native | q42 | 0.001000 | 0.002000 | 0.001000 | 0.010558 | 0.013181 | 29.68 |
| 1k | duckdb-native | q43 | 0.002000 | 0.002000 | 0.001000 | 0.010505 | 0.013011 | 29.40 |
| 1k | rudb-native | q1 | 0.000481 | 0.000294 | 0.000069 | 0.001972 | 0.001943 | 11.73 |
| 1k | rudb-native | q2 | 0.000524 | 0.000316 | 0.000023 | 0.001899 | 0.001875 | 11.99 |
| 1k | rudb-native | q3 | 0.000314 | 0.000304 | 0.000019 | 0.001813 | 0.001788 | 12.18 |
| 1k | rudb-native | q4 | 0.000300 | 0.000274 | 0.000025 | 0.001774 | 0.001722 | 11.89 |
| 1k | rudb-native | q5 | 0.000238 | 0.000279 | 0.000017 | 0.001799 | 0.001776 | 11.74 |
| 1k | rudb-native | q6 | 0.000286 | 0.000279 | 0.000014 | 0.001767 | 0.001741 | 11.86 |
| 1k | rudb-native | q7 | 0.000273 | 0.000280 | 0.000010 | 0.001763 | 0.001742 | 12.09 |
| 1k | rudb-native | q8 | 0.000512 | 0.000457 | 0.000051 | 0.001996 | 0.001974 | 13.57 |
| 1k | rudb-native | q9 | 0.000463 | 0.000492 | 0.000050 | 0.002007 | 0.001985 | 13.35 |
| 1k | rudb-native | q10 | 0.000745 | 0.000744 | 0.000045 | 0.002254 | 0.002233 | 13.90 |
| 1k | rudb-native | q11 | 0.000566 | 0.000515 | 0.000061 | 0.002029 | 0.002007 | 13.43 |
| 1k | rudb-native | q12 | 0.000540 | 0.000539 | 0.000022 | 0.002051 | 0.002029 | 13.51 |
| 1k | rudb-native | q13 | 0.000570 | 0.000610 | 0.000013 | 0.002208 | 0.002288 | 13.65 |
| 1k | rudb-native | q14 | 0.000547 | 0.000559 | 0.000038 | 0.002079 | 0.002057 | 13.52 |
| 1k | rudb-native | q15 | 0.000776 | 0.000699 | 0.000005 | 0.002231 | 0.002207 | 13.78 |
| 1k | rudb-native | q16 | 0.000341 | 0.000332 | 0.000011 | 0.001868 | 0.001847 | 12.37 |
| 1k | rudb-native | q17 | 0.000323 | 0.000357 | 0.000021 | 0.001851 | 0.001830 | 12.39 |
| 1k | rudb-native | q18 | 0.000355 | 0.000355 | 0.000081 | 0.001889 | 0.001866 | 12.38 |
| 1k | rudb-native | q19 | 0.000750 | 0.000734 | 0.000092 | 0.003375 | 0.003310 | 12.95 |
| 1k | rudb-native | q20 | 0.000617 | 0.000338 | 0.000075 | 0.001909 | 0.001882 | 12.18 |
| 1k | rudb-native | q21 | 0.000411 | 0.000432 | 0.000035 | 0.001940 | 0.001916 | 13.32 |
| 1k | rudb-native | q22 | 0.000528 | 0.000531 | 0.000091 | 0.002063 | 0.002041 | 13.40 |
| 1k | rudb-native | q23 | 0.000574 | 0.000600 | 0.000036 | 0.002159 | 0.002135 | 13.60 |
| 1k | rudb-native | q24 | 0.000499 | 0.000497 | 0.000018 | 0.002055 | 0.002034 | 12.68 |
| 1k | rudb-native | q25 | 0.000400 | 0.000422 | 0.000031 | 0.001995 | 0.001913 | 12.66 |
| 1k | rudb-native | q26 | 0.000408 | 0.000403 | 0.000014 | 0.001938 | 0.001912 | 12.70 |
| 1k | rudb-native | q27 | 0.000414 | 0.000422 | 0.000016 | 0.001936 | 0.001910 | 12.66 |
| 1k | rudb-native | q28 | 0.000657 | 0.000630 | 0.000029 | 0.002165 | 0.002135 | 14.11 |
| 1k | rudb-native | q29 | 0.000705 | 0.000792 | 0.000147 | 0.002339 | 0.002315 | 14.10 |
| 1k | rudb-native | q30 | 0.001769 | 0.001814 | 0.000060 | 0.003422 | 0.003398 | 13.36 |
| 1k | rudb-native | q31 | 0.000786 | 0.000744 | 0.000031 | 0.002282 | 0.002257 | 13.78 |
| 1k | rudb-native | q32 | 0.000477 | 0.000482 | 0.000023 | 0.002004 | 0.001979 | 12.84 |
| 1k | rudb-native | q33 | 0.000356 | 0.000394 | 0.000020 | 0.001910 | 0.001887 | 12.39 |
| 1k | rudb-native | q34 | 0.000449 | 0.000448 | 0.000026 | 0.001968 | 0.001947 | 12.85 |
| 1k | rudb-native | q35 | 0.000729 | 0.000677 | 0.000093 | 0.002280 | 0.002366 | 13.74 |
| 1k | rudb-native | q36 | 0.000646 | 0.000648 | 0.000081 | 0.002161 | 0.002137 | 13.51 |
| 1k | rudb-native | q37 | 0.000925 | 0.000861 | 0.000041 | 0.002486 | 0.002566 | 14.23 |
| 1k | rudb-native | q38 | 0.000894 | 0.000890 | 0.000040 | 0.002537 | 0.002607 | 14.24 |
| 1k | rudb-native | q39 | 0.000540 | 0.000557 | 0.000022 | 0.002094 | 0.002070 | 13.82 |
| 1k | rudb-native | q40 | 0.000693 | 0.000656 | 0.000009 | 0.002202 | 0.002177 | 14.27 |
| 1k | rudb-native | q41 | 0.000563 | 0.000612 | 0.000011 | 0.002186 | 0.002162 | 14.07 |
| 1k | rudb-native | q42 | 0.000642 | 0.000638 | 0.000067 | 0.002169 | 0.002146 | 13.68 |
| 1k | rudb-native | q43 | 0.000565 | 0.000578 | 0.000076 | 0.002152 | 0.002128 | 13.95 |
| 1k | duckdb-parquet | q1 | 0.001000 | 0.001000 | 0.000000 | 0.013910 | 0.015642 | 26.40 |
| 1k | duckdb-parquet | q2 | 0.002000 | 0.001000 | 0.000000 | 0.010350 | 0.011867 | 27.45 |
| 1k | duckdb-parquet | q3 | 0.001000 | 0.001000 | 0.001000 | 0.009875 | 0.011046 | 27.43 |
| 1k | duckdb-parquet | q4 | 0.001000 | 0.001000 | 0.000000 | 0.009785 | 0.011203 | 26.67 |
| 1k | duckdb-parquet | q5 | 0.002000 | 0.001000 | 0.001000 | 0.010514 | 0.012544 | 28.29 |
| 1k | duckdb-parquet | q6 | 0.002000 | 0.001000 | 0.001000 | 0.010417 | 0.012507 | 28.83 |
| 1k | duckdb-parquet | q7 | 0.001000 | 0.001000 | 0.000000 | 0.009946 | 0.011310 | 27.41 |
| 1k | duckdb-parquet | q8 | 0.002000 | 0.001000 | 0.000000 | 0.010335 | 0.012252 | 28.93 |
| 1k | duckdb-parquet | q9 | 0.002000 | 0.002000 | 0.000000 | 0.011042 | 0.015005 | 32.07 |
| 1k | duckdb-parquet | q10 | 0.003000 | 0.002000 | 0.000000 | 0.011476 | 0.015584 | 32.93 |
| 1k | duckdb-parquet | q11 | 0.002000 | 0.002000 | 0.000000 | 0.010989 | 0.014317 | 31.42 |
| 1k | duckdb-parquet | q12 | 0.002000 | 0.002000 | 0.000000 | 0.011142 | 0.014914 | 32.18 |
| 1k | duckdb-parquet | q13 | 0.002000 | 0.002000 | 0.000000 | 0.010606 | 0.013068 | 29.18 |
| 1k | duckdb-parquet | q14 | 0.003000 | 0.002000 | 0.000000 | 0.011416 | 0.015252 | 32.43 |
| 1k | duckdb-parquet | q15 | 0.002000 | 0.002000 | 0.000000 | 0.010749 | 0.013610 | 29.93 |
| 1k | duckdb-parquet | q16 | 0.002000 | 0.002000 | 0.001000 | 0.010622 | 0.013074 | 30.61 |
| 1k | duckdb-parquet | q17 | 0.002000 | 0.002000 | 0.001000 | 0.010851 | 0.013374 | 31.48 |
| 1k | duckdb-parquet | q18 | 0.002000 | 0.002000 | 0.000000 | 0.010666 | 0.012872 | 30.48 |
| 1k | duckdb-parquet | q19 | 0.003000 | 0.004000 | 0.001000 | 0.019546 | 0.023754 | 31.67 |
| 1k | duckdb-parquet | q20 | 0.001000 | 0.001000 | 0.000000 | 0.010469 | 0.011909 | 27.15 |
| 1k | duckdb-parquet | q21 | 0.002000 | 0.001000 | 0.000000 | 0.009975 | 0.011431 | 27.18 |
| 1k | duckdb-parquet | q22 | 0.001000 | 0.001000 | 0.001000 | 0.010614 | 0.012339 | 28.22 |
| 1k | duckdb-parquet | q23 | 0.001000 | 0.002000 | 0.000000 | 0.010501 | 0.012438 | 28.19 |
| 1k | duckdb-parquet | q24 | 0.006000 | 0.005000 | 0.001000 | 0.014376 | 0.019565 | 31.57 |
| 1k | duckdb-parquet | q25 | 0.002000 | 0.002000 | 0.000000 | 0.011488 | 0.013956 | 29.65 |
| 1k | duckdb-parquet | q26 | 0.002000 | 0.001000 | 0.000000 | 0.010274 | 0.011806 | 27.16 |
| 1k | duckdb-parquet | q27 | 0.002000 | 0.001000 | 0.001000 | 0.010413 | 0.011808 | 27.20 |
| 1k | duckdb-parquet | q28 | 0.002000 | 0.002000 | 0.001000 | 0.010715 | 0.013641 | 29.67 |
| 1k | duckdb-parquet | q29 | 0.003000 | 0.002000 | 0.000000 | 0.011989 | 0.015129 | 30.41 |
| 1k | duckdb-parquet | q30 | 0.005000 | 0.004000 | 0.000000 | 0.013657 | 0.016121 | 29.92 |
| 1k | duckdb-parquet | q31 | 0.002000 | 0.002000 | 0.001000 | 0.011028 | 0.013672 | 31.48 |
| 1k | duckdb-parquet | q32 | 0.002000 | 0.002000 | 0.001000 | 0.011188 | 0.013737 | 31.49 |
| 1k | duckdb-parquet | q33 | 0.002000 | 0.002000 | 0.000000 | 0.010936 | 0.013620 | 31.96 |
| 1k | duckdb-parquet | q34 | 0.002000 | 0.002000 | 0.000000 | 0.010833 | 0.013598 | 29.80 |
| 1k | duckdb-parquet | q35 | 0.002000 | 0.002000 | 0.000000 | 0.010916 | 0.014074 | 30.27 |
| 1k | duckdb-parquet | q36 | 0.002000 | 0.002000 | 0.000000 | 0.010858 | 0.013901 | 31.02 |
| 1k | duckdb-parquet | q37 | 0.002000 | 0.002000 | 0.000000 | 0.010993 | 0.013664 | 29.68 |
| 1k | duckdb-parquet | q38 | 0.002000 | 0.002000 | 0.000000 | 0.011042 | 0.013703 | 29.41 |
| 1k | duckdb-parquet | q39 | 0.001000 | 0.001000 | 0.001000 | 0.010241 | 0.011816 | 28.18 |
| 1k | duckdb-parquet | q40 | 0.003000 | 0.002000 | 0.000000 | 0.011352 | 0.014784 | 30.92 |
| 1k | duckdb-parquet | q41 | 0.002000 | 0.002000 | 0.001000 | 0.011020 | 0.013558 | 30.67 |
| 1k | duckdb-parquet | q42 | 0.002000 | 0.002000 | 0.000000 | 0.010909 | 0.013110 | 29.31 |
| 1k | duckdb-parquet | q43 | 0.002000 | 0.002000 | 0.000000 | 0.010899 | 0.013468 | 29.41 |
| 1k | rudb-parquet | q1 | 0.000887 | 0.000620 | 0.000293 | 0.001916 | 0.001889 | 11.70 |
| 1k | rudb-parquet | q2 | 0.001072 | 0.000598 | 0.000097 | 0.001850 | 0.001825 | 12.50 |
| 1k | rudb-parquet | q3 | 0.000606 | 0.000571 | 0.000028 | 0.001758 | 0.001737 | 11.96 |
| 1k | rudb-parquet | q4 | 0.000514 | 0.000516 | 0.000013 | 0.001697 | 0.001617 | 12.16 |
| 1k | rudb-parquet | q5 | 0.000529 | 0.000537 | 0.000023 | 0.001709 | 0.001687 | 11.96 |
| 1k | rudb-parquet | q6 | 0.000763 | 0.000695 | 0.000045 | 0.001892 | 0.001869 | 12.12 |
| 1k | rudb-parquet | q7 | 0.000541 | 0.000556 | 0.000016 | 0.001712 | 0.001691 | 12.56 |
| 1k | rudb-parquet | q8 | 0.000683 | 0.000604 | 0.000028 | 0.001760 | 0.001734 | 12.70 |
| 1k | rudb-parquet | q9 | 0.000649 | 0.000656 | 0.000028 | 0.001869 | 0.001847 | 12.31 |
| 1k | rudb-parquet | q10 | 0.000942 | 0.000924 | 0.000029 | 0.002124 | 0.002102 | 12.88 |
| 1k | rudb-parquet | q11 | 0.000590 | 0.000637 | 0.000030 | 0.001847 | 0.001823 | 12.43 |
| 1k | rudb-parquet | q12 | 0.000683 | 0.000656 | 0.000055 | 0.001851 | 0.001826 | 12.49 |
| 1k | rudb-parquet | q13 | 0.000717 | 0.000719 | 0.000043 | 0.001934 | 0.001912 | 12.42 |
| 1k | rudb-parquet | q14 | 0.000757 | 0.000741 | 0.000065 | 0.001973 | 0.001951 | 12.48 |
| 1k | rudb-parquet | q15 | 0.000681 | 0.000725 | 0.000030 | 0.001912 | 0.001890 | 12.34 |
| 1k | rudb-parquet | q16 | 0.000662 | 0.000643 | 0.000102 | 0.001816 | 0.001794 | 12.31 |
| 1k | rudb-parquet | q17 | 0.000736 | 0.000755 | 0.000069 | 0.001927 | 0.001904 | 12.30 |
| 1k | rudb-parquet | q18 | 0.000732 | 0.000665 | 0.000419 | 0.001845 | 0.001822 | 11.92 |
| 1k | rudb-parquet | q19 | 0.001359 | 0.001384 | 0.000129 | 0.003567 | 0.003515 | 12.80 |
| 1k | rudb-parquet | q20 | 0.000651 | 0.000453 | 0.000038 | 0.001704 | 0.001611 | 12.06 |
| 1k | rudb-parquet | q21 | 0.000980 | 0.000931 | 0.000036 | 0.002093 | 0.002070 | 12.50 |
| 1k | rudb-parquet | q22 | 0.001061 | 0.001073 | 0.000056 | 0.002283 | 0.002257 | 12.46 |
| 1k | rudb-parquet | q23 | 0.001510 | 0.001507 | 0.000074 | 0.002702 | 0.002681 | 12.66 |
| 1k | rudb-parquet | q24 | 0.001180 | 0.001145 | 0.000039 | 0.002378 | 0.002353 | 12.18 |
| 1k | rudb-parquet | q25 | 0.000634 | 0.000643 | 0.000016 | 0.001846 | 0.001822 | 12.69 |
| 1k | rudb-parquet | q26 | 0.000611 | 0.000583 | 0.000013 | 0.001778 | 0.001726 | 11.93 |
| 1k | rudb-parquet | q27 | 0.000684 | 0.000656 | 0.000026 | 0.001893 | 0.001852 | 12.67 |
| 1k | rudb-parquet | q28 | 0.001055 | 0.001077 | 0.000093 | 0.002311 | 0.002289 | 12.98 |
| 1k | rudb-parquet | q29 | 0.001126 | 0.001142 | 0.000050 | 0.002377 | 0.002353 | 13.21 |
| 1k | rudb-parquet | q30 | 0.002114 | 0.002142 | 0.000091 | 0.003382 | 0.003354 | 13.23 |
| 1k | rudb-parquet | q31 | 0.000954 | 0.000939 | 0.000064 | 0.002125 | 0.002106 | 12.68 |
| 1k | rudb-parquet | q32 | 0.000961 | 0.000926 | 0.000043 | 0.002135 | 0.002115 | 12.68 |
| 1k | rudb-parquet | q33 | 0.000895 | 0.000891 | 0.000070 | 0.002060 | 0.002028 | 12.62 |
| 1k | rudb-parquet | q34 | 0.001041 | 0.001038 | 0.000035 | 0.002198 | 0.002174 | 12.55 |
| 1k | rudb-parquet | q35 | 0.001011 | 0.001033 | 0.000042 | 0.002238 | 0.002202 | 12.54 |
| 1k | rudb-parquet | q36 | 0.000702 | 0.000716 | 0.000055 | 0.001884 | 0.001852 | 12.51 |
| 1k | rudb-parquet | q37 | 0.001105 | 0.001126 | 0.000036 | 0.002361 | 0.002326 | 13.28 |
| 1k | rudb-parquet | q38 | 0.001209 | 0.001168 | 0.000030 | 0.002363 | 0.002317 | 13.28 |
| 1k | rudb-parquet | q39 | 0.001046 | 0.001055 | 0.000039 | 0.002236 | 0.002213 | 12.96 |
| 1k | rudb-parquet | q40 | 0.001632 | 0.001482 | 0.000103 | 0.002704 | 0.002679 | 13.45 |
| 1k | rudb-parquet | q41 | 0.000799 | 0.000805 | 0.000041 | 0.002020 | 0.001998 | 13.34 |
| 1k | rudb-parquet | q42 | 0.000831 | 0.000827 | 0.000036 | 0.002032 | 0.002009 | 13.37 |
| 1k | rudb-parquet | q43 | 0.000764 | 0.000813 | 0.000027 | 0.002019 | 0.001992 | 13.80 |
| 10k | duckdb-native | q1 | 0.000000 | 0.001000 | 0.001000 | 0.010194 | 0.011752 | 26.50 |
| 10k | duckdb-native | q2 | 0.003000 | 0.001000 | 0.000000 | 0.014062 | 0.016827 | 27.60 |
| 10k | duckdb-native | q3 | 0.002000 | 0.001000 | 0.000000 | 0.015163 | 0.018161 | 28.11 |
| 10k | duckdb-native | q4 | 0.000000 | 0.001000 | 0.000000 | 0.010245 | 0.011603 | 27.15 |
| 10k | duckdb-native | q5 | 0.001000 | 0.002000 | 0.001000 | 0.017586 | 0.021236 | 29.87 |
| 10k | duckdb-native | q6 | 0.002000 | 0.001000 | 0.000000 | 0.010295 | 0.012577 | 29.16 |
| 10k | duckdb-native | q7 | 0.000000 | 0.001000 | 0.001000 | 0.009195 | 0.010372 | 26.66 |
| 10k | duckdb-native | q8 | 0.001000 | 0.001000 | 0.000000 | 0.009987 | 0.011846 | 28.67 |
| 10k | duckdb-native | q9 | 0.002000 | 0.002000 | 0.001000 | 0.011025 | 0.015345 | 33.67 |
| 10k | duckdb-native | q10 | 0.003000 | 0.002000 | 0.000000 | 0.011599 | 0.016508 | 35.17 |
| 10k | duckdb-native | q11 | 0.002000 | 0.002000 | 0.001000 | 0.010666 | 0.014314 | 32.92 |
| 10k | duckdb-native | q12 | 0.002000 | 0.002000 | 0.000000 | 0.010974 | 0.014832 | 33.06 |
| 10k | duckdb-native | q13 | 0.001000 | 0.002000 | 0.001000 | 0.010457 | 0.013171 | 30.42 |
| 10k | duckdb-native | q14 | 0.002000 | 0.002000 | 0.000000 | 0.011148 | 0.015568 | 33.43 |
| 10k | duckdb-native | q15 | 0.001000 | 0.001000 | 0.000000 | 0.010468 | 0.013546 | 30.92 |
| 10k | duckdb-native | q16 | 0.002000 | 0.001000 | 0.001000 | 0.010591 | 0.013804 | 32.26 |
| 10k | duckdb-native | q17 | 0.002000 | 0.002000 | 0.000000 | 0.011081 | 0.014666 | 34.17 |
| 10k | duckdb-native | q18 | 0.001000 | 0.002000 | 0.001000 | 0.010835 | 0.013260 | 33.22 |
| 10k | duckdb-native | q19 | 0.003000 | 0.002000 | 0.001000 | 0.011488 | 0.015666 | 35.10 |
| 10k | duckdb-native | q20 | 0.000000 | 0.000000 | 0.001000 | 0.009586 | 0.010927 | 26.91 |
| 10k | duckdb-native | q21 | 0.001000 | 0.001000 | 0.001000 | 0.010382 | 0.011861 | 28.27 |
| 10k | duckdb-native | q22 | 0.002000 | 0.002000 | 0.001000 | 0.010875 | 0.013225 | 29.42 |
| 10k | duckdb-native | q23 | 0.002000 | 0.003000 | 0.001000 | 0.012001 | 0.016001 | 32.43 |
| 10k | duckdb-native | q24 | 0.005000 | 0.005000 | 0.001000 | 0.015030 | 0.020764 | 36.70 |
| 10k | duckdb-native | q25 | 0.002000 | 0.002000 | 0.001000 | 0.010656 | 0.013190 | 29.91 |
| 10k | duckdb-native | q26 | 0.001000 | 0.001000 | 0.000000 | 0.009758 | 0.011111 | 27.35 |
| 10k | duckdb-native | q27 | 0.001000 | 0.001000 | 0.000000 | 0.010094 | 0.011500 | 28.16 |
| 10k | duckdb-native | q28 | 0.002000 | 0.002000 | 0.000000 | 0.011235 | 0.014787 | 31.17 |
| 10k | duckdb-native | q29 | 0.006000 | 0.006000 | 0.000000 | 0.015354 | 0.023735 | 31.91 |
| 10k | duckdb-native | q30 | 0.003000 | 0.004000 | 0.004000 | 0.013268 | 0.015996 | 30.91 |
| 10k | duckdb-native | q31 | 0.003000 | 0.003000 | 0.001000 | 0.018508 | 0.022065 | 33.57 |
| 10k | duckdb-native | q32 | 0.002000 | 0.002000 | 0.000000 | 0.011269 | 0.014800 | 32.71 |
| 10k | duckdb-native | q33 | 0.004000 | 0.003000 | 0.001000 | 0.021125 | 0.027308 | 33.98 |
| 10k | duckdb-native | q34 | 0.004000 | 0.003000 | 0.000000 | 0.011801 | 0.016038 | 33.67 |
| 10k | duckdb-native | q35 | 0.003000 | 0.002000 | 0.001000 | 0.011889 | 0.016658 | 34.06 |
| 10k | duckdb-native | q36 | 0.002000 | 0.002000 | 0.000000 | 0.011047 | 0.014674 | 33.22 |
| 10k | duckdb-native | q37 | 0.002000 | 0.002000 | 0.001000 | 0.011058 | 0.014354 | 31.43 |
| 10k | duckdb-native | q38 | 0.002000 | 0.002000 | 0.000000 | 0.010996 | 0.014091 | 31.37 |
| 10k | duckdb-native | q39 | 0.002000 | 0.002000 | 0.001000 | 0.010800 | 0.013752 | 30.92 |
| 10k | duckdb-native | q40 | 0.002000 | 0.002000 | 0.001000 | 0.011559 | 0.015376 | 33.17 |
| 10k | duckdb-native | q41 | 0.002000 | 0.002000 | 0.000000 | 0.010777 | 0.013666 | 32.35 |
| 10k | duckdb-native | q42 | 0.002000 | 0.001000 | 0.001000 | 0.010606 | 0.013569 | 31.30 |
| 10k | duckdb-native | q43 | 0.002000 | 0.001000 | 0.000000 | 0.010634 | 0.013294 | 30.41 |
| 10k | rudb-native | q1 | 0.000278 | 0.000320 | 0.000209 | 0.002270 | 0.002245 | 11.71 |
| 10k | rudb-native | q2 | 0.000473 | 0.000350 | 0.000164 | 0.002311 | 0.002272 | 11.95 |
| 10k | rudb-native | q3 | 0.000519 | 0.000355 | 0.000197 | 0.002400 | 0.002376 | 12.17 |
| 10k | rudb-native | q4 | 0.000366 | 0.000317 | 0.000082 | 0.002255 | 0.002232 | 11.93 |
| 10k | rudb-native | q5 | 0.000551 | 0.000528 | 0.000071 | 0.004034 | 0.003981 | 11.73 |
| 10k | rudb-native | q6 | 0.000493 | 0.000311 | 0.000043 | 0.002261 | 0.002223 | 11.83 |
| 10k | rudb-native | q7 | 0.000297 | 0.000302 | 0.000039 | 0.002191 | 0.002165 | 12.11 |
| 10k | rudb-native | q8 | 0.000639 | 0.000596 | 0.000048 | 0.002552 | 0.002606 | 13.77 |
| 10k | rudb-native | q9 | 0.000704 | 0.000764 | 0.000054 | 0.002630 | 0.002608 | 14.12 |
| 10k | rudb-native | q10 | 0.001111 | 0.001100 | 0.000066 | 0.003016 | 0.002994 | 14.42 |
| 10k | rudb-native | q11 | 0.000619 | 0.000628 | 0.000056 | 0.002587 | 0.002680 | 14.04 |
| 10k | rudb-native | q12 | 0.000749 | 0.000682 | 0.000046 | 0.002663 | 0.002767 | 14.08 |
| 10k | rudb-native | q13 | 0.000764 | 0.000785 | 0.000009 | 0.002766 | 0.002935 | 14.31 |
| 10k | rudb-native | q14 | 0.000831 | 0.000845 | 0.000030 | 0.002842 | 0.002952 | 14.23 |
| 10k | rudb-native | q15 | 0.001052 | 0.001066 | 0.000056 | 0.003058 | 0.003173 | 14.71 |
| 10k | rudb-native | q16 | 0.000795 | 0.000789 | 0.000053 | 0.002758 | 0.002790 | 13.93 |
| 10k | rudb-native | q17 | 0.000959 | 0.001035 | 0.000074 | 0.003021 | 0.003139 | 14.38 |
| 10k | rudb-native | q18 | 0.000627 | 0.000654 | 0.000052 | 0.002588 | 0.002555 | 13.27 |
| 10k | rudb-native | q19 | 0.001258 | 0.001166 | 0.000075 | 0.003175 | 0.003336 | 14.97 |
| 10k | rudb-native | q20 | 0.000351 | 0.000346 | 0.000026 | 0.002212 | 0.002187 | 12.18 |
| 10k | rudb-native | q21 | 0.000476 | 0.000514 | 0.000054 | 0.002428 | 0.002397 | 13.43 |
| 10k | rudb-native | q22 | 0.000744 | 0.000755 | 0.000089 | 0.002801 | 0.002893 | 13.86 |
| 10k | rudb-native | q23 | 0.001662 | 0.001659 | 0.000086 | 0.003720 | 0.004412 | 15.30 |
| 10k | rudb-native | q24 | 0.000613 | 0.000596 | 0.000067 | 0.002547 | 0.002522 | 13.20 |
| 10k | rudb-native | q25 | 0.000605 | 0.000660 | 0.000023 | 0.002559 | 0.002531 | 13.89 |
| 10k | rudb-native | q26 | 0.000624 | 0.000638 | 0.000127 | 0.002488 | 0.002465 | 13.41 |
| 10k | rudb-native | q27 | 0.000656 | 0.000688 | 0.000086 | 0.002576 | 0.002553 | 13.88 |
| 10k | rudb-native | q28 | 0.001010 | 0.001037 | 0.000074 | 0.003021 | 0.003206 | 15.02 |
| 10k | rudb-native | q29 | 0.001721 | 0.001756 | 0.000152 | 0.003826 | 0.004718 | 15.82 |
| 10k | rudb-native | q30 | 0.001806 | 0.001946 | 0.001560 | 0.005447 | 0.005396 | 13.59 |
| 10k | rudb-native | q31 | 0.001109 | 0.001476 | 0.000225 | 0.004978 | 0.004235 | 14.86 |
| 10k | rudb-native | q32 | 0.000646 | 0.000593 | 0.000025 | 0.002552 | 0.002529 | 13.55 |
| 10k | rudb-native | q33 | 0.000753 | 0.000827 | 0.000125 | 0.004230 | 0.004171 | 13.05 |
| 10k | rudb-native | q34 | 0.000505 | 0.000468 | 0.000024 | 0.002367 | 0.002343 | 12.85 |
| 10k | rudb-native | q35 | 0.001080 | 0.001051 | 0.000038 | 0.003044 | 0.003204 | 14.47 |
| 10k | rudb-native | q36 | 0.000524 | 0.000488 | 0.000031 | 0.002394 | 0.002368 | 13.08 |
| 10k | rudb-native | q37 | 0.000894 | 0.000923 | 0.000014 | 0.002934 | 0.003070 | 15.52 |
| 10k | rudb-native | q38 | 0.001099 | 0.001069 | 0.000028 | 0.003076 | 0.003215 | 15.86 |
| 10k | rudb-native | q39 | 0.000742 | 0.000768 | 0.000064 | 0.002747 | 0.002878 | 15.04 |
| 10k | rudb-native | q40 | 0.000836 | 0.000816 | 0.000081 | 0.002848 | 0.002905 | 14.86 |
| 10k | rudb-native | q41 | 0.000952 | 0.000769 | 0.000064 | 0.002781 | 0.002844 | 15.59 |
| 10k | rudb-native | q42 | 0.000811 | 0.000758 | 0.000055 | 0.002751 | 0.002803 | 15.28 |
| 10k | rudb-native | q43 | 0.000731 | 0.000723 | 0.000044 | 0.002737 | 0.002790 | 15.36 |
| 10k | duckdb-parquet | q1 | 0.001000 | 0.001000 | 0.001000 | 0.010076 | 0.011582 | 26.44 |
| 10k | duckdb-parquet | q2 | 0.002000 | 0.001000 | 0.001000 | 0.011983 | 0.013304 | 27.44 |
| 10k | duckdb-parquet | q3 | 0.002000 | 0.002000 | 0.001000 | 0.015907 | 0.018009 | 27.66 |
| 10k | duckdb-parquet | q4 | 0.001000 | 0.001000 | 0.000000 | 0.010379 | 0.012061 | 27.41 |
| 10k | duckdb-parquet | q5 | 0.003000 | 0.003000 | 0.000000 | 0.019686 | 0.022644 | 29.66 |
| 10k | duckdb-parquet | q6 | 0.002000 | 0.002000 | 0.000000 | 0.011530 | 0.013700 | 29.17 |
| 10k | duckdb-parquet | q7 | 0.001000 | 0.001000 | 0.000000 | 0.009884 | 0.011835 | 27.16 |
| 10k | duckdb-parquet | q8 | 0.002000 | 0.001000 | 0.001000 | 0.010644 | 0.012134 | 28.68 |
| 10k | duckdb-parquet | q9 | 0.002000 | 0.002000 | 0.001000 | 0.011452 | 0.015885 | 33.43 |
| 10k | duckdb-parquet | q10 | 0.002000 | 0.003000 | 0.001000 | 0.011852 | 0.016648 | 34.43 |
| 10k | duckdb-parquet | q11 | 0.002000 | 0.002000 | 0.000000 | 0.011187 | 0.014912 | 32.18 |
| 10k | duckdb-parquet | q12 | 0.002000 | 0.002000 | 0.001000 | 0.011380 | 0.015684 | 32.88 |
| 10k | duckdb-parquet | q13 | 0.002000 | 0.002000 | 0.000000 | 0.011069 | 0.013958 | 29.82 |
| 10k | duckdb-parquet | q14 | 0.002000 | 0.003000 | 0.001000 | 0.011984 | 0.016205 | 33.18 |
| 10k | duckdb-parquet | q15 | 0.002000 | 0.002000 | 0.000000 | 0.011380 | 0.014397 | 30.68 |
| 10k | duckdb-parquet | q16 | 0.002000 | 0.002000 | 0.000000 | 0.011204 | 0.014207 | 32.48 |
| 10k | duckdb-parquet | q17 | 0.003000 | 0.003000 | 0.001000 | 0.011602 | 0.015183 | 34.06 |
| 10k | duckdb-parquet | q18 | 0.002000 | 0.003000 | 0.001000 | 0.011508 | 0.013863 | 32.48 |
| 10k | duckdb-parquet | q19 | 0.003000 | 0.003000 | 0.000000 | 0.011982 | 0.015909 | 34.23 |
| 10k | duckdb-parquet | q20 | 0.001000 | 0.001000 | 0.000000 | 0.009990 | 0.011520 | 27.16 |
| 10k | duckdb-parquet | q21 | 0.002000 | 0.002000 | 0.001000 | 0.011351 | 0.012991 | 28.17 |
| 10k | duckdb-parquet | q22 | 0.002000 | 0.002000 | 0.001000 | 0.011549 | 0.014234 | 29.17 |
| 10k | duckdb-parquet | q23 | 0.004000 | 0.004000 | 0.000000 | 0.013207 | 0.017930 | 32.68 |
| 10k | duckdb-parquet | q24 | 0.010000 | 0.010000 | 0.000000 | 0.019682 | 0.029531 | 37.39 |
| 10k | duckdb-parquet | q25 | 0.003000 | 0.003000 | 0.001000 | 0.011759 | 0.015012 | 30.04 |
| 10k | duckdb-parquet | q26 | 0.002000 | 0.001000 | 0.001000 | 0.010407 | 0.012265 | 27.46 |
| 10k | duckdb-parquet | q27 | 0.001000 | 0.002000 | 0.000000 | 0.010617 | 0.012682 | 27.79 |
| 10k | duckdb-parquet | q28 | 0.003000 | 0.003000 | 0.001000 | 0.011846 | 0.015531 | 30.68 |
| 10k | duckdb-parquet | q29 | 0.007000 | 0.007000 | 0.000000 | 0.016537 | 0.025272 | 32.09 |
| 10k | duckdb-parquet | q30 | 0.004000 | 0.007000 | 0.003000 | 0.020304 | 0.023815 | 30.67 |
| 10k | duckdb-parquet | q31 | 0.004000 | 0.004000 | 0.001000 | 0.018977 | 0.022195 | 32.23 |
| 10k | duckdb-parquet | q32 | 0.002000 | 0.002000 | 0.001000 | 0.011782 | 0.014999 | 31.78 |
| 10k | duckdb-parquet | q33 | 0.004000 | 0.004000 | 0.001000 | 0.021638 | 0.026812 | 34.21 |
| 10k | duckdb-parquet | q34 | 0.007000 | 0.003000 | 0.001000 | 0.012646 | 0.017284 | 33.18 |
| 10k | duckdb-parquet | q35 | 0.003000 | 0.003000 | 0.001000 | 0.012557 | 0.017635 | 33.65 |
| 10k | duckdb-parquet | q36 | 0.002000 | 0.002000 | 0.001000 | 0.011492 | 0.014861 | 32.54 |
| 10k | duckdb-parquet | q37 | 0.002000 | 0.003000 | 0.001000 | 0.011699 | 0.015486 | 30.88 |
| 10k | duckdb-parquet | q38 | 0.003000 | 0.003000 | 0.001000 | 0.011738 | 0.015161 | 31.05 |
| 10k | duckdb-parquet | q39 | 0.002000 | 0.003000 | 0.001000 | 0.011827 | 0.015407 | 30.55 |
| 10k | duckdb-parquet | q40 | 0.003000 | 0.004000 | 0.000000 | 0.012858 | 0.017757 | 32.68 |
| 10k | duckdb-parquet | q41 | 0.002000 | 0.002000 | 0.000000 | 0.011225 | 0.014112 | 31.72 |
| 10k | duckdb-parquet | q42 | 0.002000 | 0.002000 | 0.000000 | 0.011160 | 0.014353 | 30.31 |
| 10k | duckdb-parquet | q43 | 0.002000 | 0.002000 | 0.000000 | 0.011069 | 0.013904 | 29.62 |
| 10k | rudb-parquet | q1 | 0.000496 | 0.000516 | 0.000350 | 0.001794 | 0.001772 | 11.74 |
| 10k | rudb-parquet | q2 | 0.000970 | 0.000834 | 0.000398 | 0.002239 | 0.002211 | 12.55 |
| 10k | rudb-parquet | q3 | 0.001158 | 0.000699 | 0.000431 | 0.002020 | 0.001996 | 12.00 |
| 10k | rudb-parquet | q4 | 0.000600 | 0.000606 | 0.000033 | 0.001873 | 0.001839 | 12.31 |
| 10k | rudb-parquet | q5 | 0.000769 | 0.001165 | 0.000159 | 0.003383 | 0.003330 | 12.24 |
| 10k | rudb-parquet | q6 | 0.002072 | 0.001327 | 0.000238 | 0.002586 | 0.002550 | 12.81 |
| 10k | rudb-parquet | q7 | 0.000597 | 0.000626 | 0.000030 | 0.001823 | 0.001797 | 12.73 |
| 10k | rudb-parquet | q8 | 0.000631 | 0.000652 | 0.000082 | 0.001889 | 0.001777 | 12.59 |
| 10k | rudb-parquet | q9 | 0.001035 | 0.001032 | 0.000095 | 0.002229 | 0.002207 | 12.86 |
| 10k | rudb-parquet | q10 | 0.001522 | 0.001511 | 0.000061 | 0.002716 | 0.002690 | 13.43 |
| 10k | rudb-parquet | q11 | 0.000717 | 0.000797 | 0.000042 | 0.001990 | 0.001967 | 12.62 |
| 10k | rudb-parquet | q12 | 0.000846 | 0.000857 | 0.000041 | 0.002043 | 0.002017 | 12.91 |
| 10k | rudb-parquet | q13 | 0.001242 | 0.001251 | 0.000064 | 0.002413 | 0.002390 | 12.67 |
| 10k | rudb-parquet | q14 | 0.001358 | 0.001366 | 0.000050 | 0.002605 | 0.002579 | 12.99 |
| 10k | rudb-parquet | q15 | 0.001251 | 0.001311 | 0.000063 | 0.002516 | 0.002490 | 12.59 |
| 10k | rudb-parquet | q16 | 0.000914 | 0.000924 | 0.000042 | 0.002184 | 0.002221 | 12.30 |
| 10k | rudb-parquet | q17 | 0.001918 | 0.001944 | 0.000064 | 0.003151 | 0.003129 | 13.62 |
| 10k | rudb-parquet | q18 | 0.001372 | 0.001367 | 0.000066 | 0.002554 | 0.002529 | 12.43 |
| 10k | rudb-parquet | q19 | 0.002351 | 0.002420 | 0.000091 | 0.003668 | 0.003644 | 14.81 |
| 10k | rudb-parquet | q20 | 0.000569 | 0.000520 | 0.000049 | 0.001700 | 0.001677 | 12.05 |
| 10k | rudb-parquet | q21 | 0.003634 | 0.003688 | 0.000363 | 0.004955 | 0.004876 | 14.00 |
| 10k | rudb-parquet | q22 | 0.004061 | 0.004124 | 0.000146 | 0.005378 | 0.005350 | 14.16 |
| 10k | rudb-parquet | q23 | 0.007553 | 0.007545 | 0.000108 | 0.008812 | 0.008776 | 15.49 |
| 10k | rudb-parquet | q24 | 0.004294 | 0.004115 | 0.000444 | 0.005396 | 0.005358 | 13.68 |
| 10k | rudb-parquet | q25 | 0.001294 | 0.001319 | 0.000079 | 0.002510 | 0.002489 | 12.94 |
| 10k | rudb-parquet | q26 | 0.001141 | 0.001098 | 0.000020 | 0.002279 | 0.002256 | 12.25 |
| 10k | rudb-parquet | q27 | 0.001272 | 0.001313 | 0.000056 | 0.002519 | 0.002494 | 12.94 |
| 10k | rudb-parquet | q28 | 0.003691 | 0.003768 | 0.000053 | 0.005026 | 0.004988 | 14.55 |
| 10k | rudb-parquet | q29 | 0.004348 | 0.004449 | 0.000121 | 0.005739 | 0.005704 | 14.97 |
| 10k | rudb-parquet | q30 | 0.002206 | 0.002204 | 0.001514 | 0.003518 | 0.003493 | 13.24 |
| 10k | rudb-parquet | q31 | 0.001585 | 0.002338 | 0.000216 | 0.004174 | 0.004125 | 13.18 |
| 10k | rudb-parquet | q32 | 0.001648 | 0.001663 | 0.000114 | 0.002908 | 0.002886 | 13.19 |
| 10k | rudb-parquet | q33 | 0.001620 | 0.002308 | 0.000316 | 0.004715 | 0.004954 | 13.37 |
| 10k | rudb-parquet | q34 | 0.006569 | 0.004578 | 0.000101 | 0.005854 | 0.005828 | 15.36 |
| 10k | rudb-parquet | q35 | 0.004493 | 0.004502 | 0.000124 | 0.005740 | 0.005706 | 15.36 |
| 10k | rudb-parquet | q36 | 0.000960 | 0.000968 | 0.000040 | 0.002239 | 0.002269 | 12.77 |
| 10k | rudb-parquet | q37 | 0.003834 | 0.003809 | 0.000090 | 0.005044 | 0.005021 | 14.93 |
| 10k | rudb-parquet | q38 | 0.004284 | 0.004270 | 0.000147 | 0.005477 | 0.005444 | 14.91 |
| 10k | rudb-parquet | q39 | 0.003763 | 0.003824 | 0.000083 | 0.005076 | 0.005051 | 14.93 |
| 10k | rudb-parquet | q40 | 0.006351 | 0.006308 | 0.000180 | 0.007573 | 0.007547 | 15.68 |
| 10k | rudb-parquet | q41 | 0.001203 | 0.001216 | 0.000043 | 0.002412 | 0.002390 | 13.93 |
| 10k | rudb-parquet | q42 | 0.001294 | 0.001254 | 0.000043 | 0.002446 | 0.002422 | 13.94 |
| 10k | rudb-parquet | q43 | 0.001165 | 0.001231 | 0.000067 | 0.002457 | 0.002434 | 14.20 |
| 1m | duckdb-native | q1 | 0.001000 | 0.001000 | 0.001000 | 0.010054 | 0.011837 | 26.91 |
| 1m | duckdb-native | q2 | 0.001000 | 0.001000 | 0.000000 | 0.010133 | 0.012964 | 29.88 |
| 1m | duckdb-native | q3 | 0.001000 | 0.001000 | 0.000000 | 0.010565 | 0.015070 | 32.40 |
| 1m | duckdb-native | q4 | 0.002000 | 0.002000 | 0.001000 | 0.011103 | 0.016847 | 36.64 |
| 1m | duckdb-native | q5 | 0.011000 | 0.011000 | 0.000000 | 0.020817 | 0.062068 | 66.61 |
| 1m | duckdb-native | q6 | 0.009000 | 0.009000 | 0.000000 | 0.018875 | 0.048413 | 58.26 |
| 1m | duckdb-native | q7 | 0.001000 | 0.000000 | 0.001000 | 0.009726 | 0.011236 | 26.87 |
| 1m | duckdb-native | q8 | 0.001000 | 0.001000 | 0.000000 | 0.010479 | 0.013671 | 31.41 |
| 1m | duckdb-native | q9 | 0.014000 | 0.014000 | 0.001000 | 0.024669 | 0.079601 | 76.12 |
| 1m | duckdb-native | q10 | 0.020000 | 0.019000 | 0.001000 | 0.029957 | 0.102377 | 83.66 |
| 1m | duckdb-native | q11 | 0.005000 | 0.005000 | 0.000000 | 0.014874 | 0.031084 | 51.69 |
| 1m | duckdb-native | q12 | 0.006000 | 0.006000 | 0.001000 | 0.015485 | 0.033564 | 54.33 |
| 1m | duckdb-native | q13 | 0.007000 | 0.007000 | 0.000000 | 0.016982 | 0.041565 | 61.21 |
| 1m | duckdb-native | q14 | 0.012000 | 0.011000 | 0.001000 | 0.021891 | 0.061795 | 83.21 |
| 1m | duckdb-native | q15 | 0.008000 | 0.008000 | 0.001000 | 0.017792 | 0.044245 | 65.14 |
| 1m | duckdb-native | q16 | 0.013000 | 0.013000 | 0.017000 | 0.026235 | 0.072456 | 79.85 |
| 1m | duckdb-native | q17 | 0.033000 | 0.025000 | 0.025000 | 0.037505 | 0.140795 | 129.88 |
| 1m | duckdb-native | q18 | 0.019000 | 0.019000 | 0.001000 | 0.030125 | 0.098041 | 123.54 |
| 1m | duckdb-native | q19 | 0.029000 | 0.027000 | 0.001000 | 0.038320 | 0.140130 | 142.11 |
| 1m | duckdb-native | q20 | 0.002000 | 0.002000 | 0.001000 | 0.011182 | 0.016443 | 35.97 |
| 1m | duckdb-native | q21 | 0.020000 | 0.019000 | 0.001000 | 0.029497 | 0.089609 | 91.66 |
| 1m | duckdb-native | q22 | 0.020000 | 0.019000 | 0.001000 | 0.029625 | 0.094352 | 103.41 |
| 1m | duckdb-native | q23 | 0.017000 | 0.016000 | 0.035000 | 0.027407 | 0.087940 | 123.38 |
| 1m | duckdb-native | q24 | 0.111000 | 0.046000 | 0.001000 | 0.057929 | 0.189318 | 209.45 |
| 1m | duckdb-native | q25 | 0.004000 | 0.004000 | 0.001000 | 0.013523 | 0.025934 | 41.64 |
| 1m | duckdb-native | q26 | 0.004000 | 0.004000 | 0.000000 | 0.013295 | 0.027018 | 36.91 |
| 1m | duckdb-native | q27 | 0.004000 | 0.003000 | 0.001000 | 0.012796 | 0.025335 | 39.37 |
| 1m | duckdb-native | q28 | 0.018000 | 0.018000 | 0.000000 | 0.028512 | 0.087620 | 99.66 |
| 1m | duckdb-native | q29 | 0.122000 | 0.144000 | 0.033000 | 0.160839 | 0.641708 | 142.66 |
| 1m | duckdb-native | q30 | 0.004000 | 0.004000 | 0.000000 | 0.013480 | 0.017660 | 33.73 |
| 1m | duckdb-native | q31 | 0.008000 | 0.008000 | 0.000000 | 0.018607 | 0.046188 | 69.46 |
| 1m | duckdb-native | q32 | 0.009000 | 0.009000 | 0.001000 | 0.019427 | 0.050511 | 77.81 |
| 1m | duckdb-native | q33 | 0.023000 | 0.024000 | 0.000000 | 0.035237 | 0.123580 | 132.54 |
| 1m | duckdb-native | q34 | 0.039000 | 0.039000 | 0.002000 | 0.050906 | 0.183560 | 216.63 |
| 1m | duckdb-native | q35 | 0.041000 | 0.040000 | 0.001000 | 0.051878 | 0.199694 | 219.51 |
| 1m | duckdb-native | q36 | 0.019000 | 0.022000 | 0.008000 | 0.038946 | 0.118101 | 78.98 |
| 1m | duckdb-native | q37 | 0.003000 | 0.004000 | 0.002000 | 0.019759 | 0.027588 | 36.18 |
| 1m | duckdb-native | q38 | 0.003000 | 0.003000 | 0.001000 | 0.012187 | 0.016938 | 34.62 |
| 1m | duckdb-native | q39 | 0.002000 | 0.003000 | 0.001000 | 0.012536 | 0.017578 | 34.11 |
| 1m | duckdb-native | q40 | 0.004000 | 0.005000 | 0.000000 | 0.014782 | 0.023588 | 43.19 |
| 1m | duckdb-native | q41 | 0.002000 | 0.002000 | 0.000000 | 0.011613 | 0.015415 | 35.18 |
| 1m | duckdb-native | q42 | 0.002000 | 0.002000 | 0.000000 | 0.011405 | 0.015483 | 33.91 |
| 1m | duckdb-native | q43 | 0.002000 | 0.002000 | 0.000000 | 0.011549 | 0.015897 | 33.10 |
| 1m | rudb-native | q1 | 0.000596 | 0.000293 | 0.000059 | 0.002560 | 0.002354 | 11.96 |
| 1m | rudb-native | q2 | 0.000312 | 0.000318 | 0.000040 | 0.002390 | 0.002367 | 12.24 |
| 1m | rudb-native | q3 | 0.000369 | 0.000337 | 0.000024 | 0.002440 | 0.002407 | 12.43 |
| 1m | rudb-native | q4 | 0.000310 | 0.000309 | 0.000009 | 0.002390 | 0.002366 | 12.18 |
| 1m | rudb-native | q5 | 0.000314 | 0.000310 | 0.000015 | 0.002453 | 0.002428 | 11.98 |
| 1m | rudb-native | q6 | 0.000312 | 0.000317 | 0.000038 | 0.002481 | 0.002454 | 12.09 |
| 1m | rudb-native | q7 | 0.000297 | 0.000308 | 0.000015 | 0.002383 | 0.002341 | 12.34 |
| 1m | rudb-native | q8 | 0.000842 | 0.000850 | 0.000061 | 0.003060 | 0.004347 | 26.96 |
| 1m | rudb-native | q9 | 0.005292 | 0.005340 | 0.000314 | 0.007709 | 0.025498 | 52.06 |
| 1m | rudb-native | q10 | 0.006952 | 0.007056 | 0.000163 | 0.009390 | 0.032076 | 69.82 |
| 1m | rudb-native | q11 | 0.001860 | 0.001855 | 0.000153 | 0.004140 | 0.008287 | 47.49 |
| 1m | rudb-native | q12 | 0.002084 | 0.002038 | 0.000033 | 0.004353 | 0.009037 | 48.40 |
| 1m | rudb-native | q13 | 0.001915 | 0.001916 | 0.000078 | 0.004200 | 0.006787 | 27.48 |
| 1m | rudb-native | q14 | 0.003477 | 0.003463 | 0.000123 | 0.005841 | 0.015818 | 52.78 |
| 1m | rudb-native | q15 | 0.003372 | 0.003362 | 0.000106 | 0.005689 | 0.015293 | 31.44 |
| 1m | rudb-native | q16 | 0.000436 | 0.000469 | 0.000295 | 0.002692 | 0.002667 | 13.07 |
| 1m | rudb-native | q17 | 0.016616 | 0.007150 | 0.004213 | 0.009563 | 0.032596 | 64.81 |
| 1m | rudb-native | q18 | 0.001228 | 0.001165 | 0.000043 | 0.003418 | 0.003391 | 45.92 |
| 1m | rudb-native | q19 | 0.008534 | 0.008733 | 0.000382 | 0.011151 | 0.039005 | 64.90 |
| 1m | rudb-native | q20 | 0.000808 | 0.000701 | 0.000091 | 0.002967 | 0.003940 | 22.44 |
| 1m | rudb-native | q21 | 0.013418 | 0.012930 | 0.000213 | 0.015300 | 0.044056 | 49.56 |
| 1m | rudb-native | q22 | 0.014175 | 0.014684 | 0.000551 | 0.017107 | 0.049882 | 62.00 |
| 1m | rudb-native | q23 | 0.013019 | 0.013316 | 0.000660 | 0.015698 | 0.043156 | 66.58 |
| 1m | rudb-native | q24 | 0.030382 | 0.015341 | 0.000524 | 0.017889 | 0.049921 | 115.47 |
| 1m | rudb-native | q25 | 0.001613 | 0.001574 | 0.000188 | 0.003849 | 0.005748 | 39.12 |
| 1m | rudb-native | q26 | 0.002595 | 0.002663 | 0.000139 | 0.004939 | 0.009228 | 27.70 |
| 1m | rudb-native | q27 | 0.001955 | 0.001975 | 0.000115 | 0.004300 | 0.006372 | 39.75 |
| 1m | rudb-native | q28 | 0.004283 | 0.004325 | 0.000177 | 0.006669 | 0.013208 | 52.38 |
| 1m | rudb-native | q29 | 0.035692 | 0.034702 | 0.022370 | 0.037166 | 0.140143 | 70.99 |
| 1m | rudb-native | q30 | 0.001776 | 0.001783 | 0.000076 | 0.003961 | 0.003939 | 13.86 |
| 1m | rudb-native | q31 | 0.002833 | 0.002910 | 0.000090 | 0.005242 | 0.012985 | 56.95 |
| 1m | rudb-native | q32 | 0.000720 | 0.000701 | 0.000020 | 0.002908 | 0.002885 | 19.18 |
| 1m | rudb-native | q33 | 0.000483 | 0.000474 | 0.000046 | 0.002741 | 0.002697 | 16.71 |
| 1m | rudb-native | q34 | 0.000502 | 0.000473 | 0.000032 | 0.002687 | 0.002649 | 13.11 |
| 1m | rudb-native | q35 | 0.003203 | 0.003148 | 0.000155 | 0.005561 | 0.009974 | 30.08 |
| 1m | rudb-native | q36 | 0.000805 | 0.000706 | 0.000292 | 0.003657 | 0.002817 | 13.34 |
| 1m | rudb-native | q37 | 0.002024 | 0.002710 | 0.001137 | 0.006482 | 0.007697 | 26.14 |
| 1m | rudb-native | q38 | 0.001994 | 0.001939 | 0.000103 | 0.004250 | 0.004988 | 23.54 |
| 1m | rudb-native | q39 | 0.001475 | 0.001458 | 0.000104 | 0.003811 | 0.004589 | 28.30 |
| 1m | rudb-native | q40 | 0.003494 | 0.003531 | 0.000280 | 0.005842 | 0.007092 | 32.81 |
| 1m | rudb-native | q41 | 0.001237 | 0.001095 | 0.000028 | 0.003347 | 0.004102 | 27.84 |
| 1m | rudb-native | q42 | 0.001139 | 0.001117 | 0.000042 | 0.003336 | 0.004025 | 28.84 |
| 1m | rudb-native | q43 | 0.001788 | 0.001319 | 0.000069 | 0.003526 | 0.004377 | 22.85 |
| 1m | duckdb-parquet | q1 | 0.002000 | 0.001000 | 0.001000 | 0.011575 | 0.012886 | 27.39 |
| 1m | duckdb-parquet | q2 | 0.002000 | 0.002000 | 0.000000 | 0.011697 | 0.014835 | 28.97 |
| 1m | duckdb-parquet | q3 | 0.004000 | 0.003000 | 0.000000 | 0.012543 | 0.019191 | 30.14 |
| 1m | duckdb-parquet | q4 | 0.003000 | 0.003000 | 0.000000 | 0.012774 | 0.019745 | 38.91 |
| 1m | duckdb-parquet | q5 | 0.012000 | 0.012000 | 0.001000 | 0.022121 | 0.062656 | 62.04 |
| 1m | duckdb-parquet | q6 | 0.009000 | 0.009000 | 0.001000 | 0.019605 | 0.046812 | 55.39 |
| 1m | duckdb-parquet | q7 | 0.002000 | 0.002000 | 0.001000 | 0.011887 | 0.015972 | 28.68 |
| 1m | duckdb-parquet | q8 | 0.003000 | 0.002000 | 0.001000 | 0.011898 | 0.015411 | 30.37 |
| 1m | duckdb-parquet | q9 | 0.016000 | 0.016000 | 0.000000 | 0.027000 | 0.084247 | 68.49 |
| 1m | duckdb-parquet | q10 | 0.018000 | 0.018000 | 0.000000 | 0.028960 | 0.088897 | 74.53 |
| 1m | duckdb-parquet | q11 | 0.005000 | 0.006000 | 0.001000 | 0.015904 | 0.031036 | 49.43 |
| 1m | duckdb-parquet | q12 | 0.006000 | 0.006000 | 0.001000 | 0.016405 | 0.033220 | 50.56 |
| 1m | duckdb-parquet | q13 | 0.009000 | 0.010000 | 0.001000 | 0.020350 | 0.052237 | 60.91 |
| 1m | duckdb-parquet | q14 | 0.014000 | 0.014000 | 0.001000 | 0.024223 | 0.069932 | 74.18 |
| 1m | duckdb-parquet | q15 | 0.011000 | 0.011000 | 0.001000 | 0.020985 | 0.055747 | 63.30 |
| 1m | duckdb-parquet | q16 | 0.013000 | 0.020000 | 0.016000 | 0.032630 | 0.100216 | 73.73 |
| 1m | duckdb-parquet | q17 | 0.035000 | 0.026000 | 0.022000 | 0.038182 | 0.135379 | 120.27 |
| 1m | duckdb-parquet | q18 | 0.021000 | 0.022000 | 0.001000 | 0.033024 | 0.108773 | 122.23 |
| 1m | duckdb-parquet | q19 | 0.031000 | 0.030000 | 0.002000 | 0.041973 | 0.154006 | 134.24 |
| 1m | duckdb-parquet | q20 | 0.003000 | 0.003000 | 0.000000 | 0.013089 | 0.019349 | 40.06 |
| 1m | duckdb-parquet | q21 | 0.027000 | 0.028000 | 0.001000 | 0.038839 | 0.134076 | 123.32 |
| 1m | duckdb-parquet | q22 | 0.029000 | 0.031000 | 0.002000 | 0.042289 | 0.138963 | 149.43 |
| 1m | duckdb-parquet | q23 | 0.055000 | 0.061000 | 0.029000 | 0.074221 | 0.267606 | 252.43 |
| 1m | duckdb-parquet | q24 | 0.156000 | 0.089000 | 0.012000 | 0.102835 | 0.400291 | 360.91 |
| 1m | duckdb-parquet | q25 | 0.013000 | 0.013000 | 0.002000 | 0.022706 | 0.062558 | 56.41 |
| 1m | duckdb-parquet | q26 | 0.007000 | 0.007000 | 0.001000 | 0.016585 | 0.036148 | 40.32 |
| 1m | duckdb-parquet | q27 | 0.009000 | 0.009000 | 0.000000 | 0.019122 | 0.046626 | 49.91 |
| 1m | duckdb-parquet | q28 | 0.028000 | 0.027000 | 0.003000 | 0.037883 | 0.125940 | 141.68 |
| 1m | duckdb-parquet | q29 | 0.127000 | 0.153000 | 0.066000 | 0.167214 | 0.711478 | 164.72 |
| 1m | duckdb-parquet | q30 | 0.006000 | 0.006000 | 0.000000 | 0.015660 | 0.022342 | 32.84 |
| 1m | duckdb-parquet | q31 | 0.010000 | 0.010000 | 0.000000 | 0.020698 | 0.054945 | 60.61 |
| 1m | duckdb-parquet | q32 | 0.011000 | 0.011000 | 0.000000 | 0.021487 | 0.057763 | 68.98 |
| 1m | duckdb-parquet | q33 | 0.024000 | 0.025000 | 0.002000 | 0.035546 | 0.123564 | 123.52 |
| 1m | duckdb-parquet | q34 | 0.048000 | 0.049000 | 0.004000 | 0.060637 | 0.233173 | 233.68 |
| 1m | duckdb-parquet | q35 | 0.050000 | 0.050000 | 0.002000 | 0.064279 | 0.240001 | 238.12 |
| 1m | duckdb-parquet | q36 | 0.055000 | 0.021000 | 0.003000 | 0.037857 | 0.111236 | 77.73 |
| 1m | duckdb-parquet | q37 | 0.016000 | 0.025000 | 0.016000 | 0.041305 | 0.078503 | 64.43 |
| 1m | duckdb-parquet | q38 | 0.013000 | 0.013000 | 0.000000 | 0.023590 | 0.037719 | 60.43 |
| 1m | duckdb-parquet | q39 | 0.015000 | 0.016000 | 0.002000 | 0.027056 | 0.052157 | 63.40 |
| 1m | duckdb-parquet | q40 | 0.026000 | 0.025000 | 0.001000 | 0.035826 | 0.077918 | 87.34 |
| 1m | duckdb-parquet | q41 | 0.005000 | 0.004000 | 0.001000 | 0.014370 | 0.021394 | 38.49 |
| 1m | duckdb-parquet | q42 | 0.004000 | 0.004000 | 0.000000 | 0.013939 | 0.018823 | 35.45 |
| 1m | duckdb-parquet | q43 | 0.004000 | 0.004000 | 0.001000 | 0.013906 | 0.020377 | 33.77 |
| 1m | rudb-parquet | q1 | 0.002339 | 0.001397 | 0.000459 | 0.002748 | 0.003105 | 12.55 |
| 1m | rudb-parquet | q2 | 0.002001 | 0.001956 | 0.000122 | 0.003202 | 0.005885 | 16.65 |
| 1m | rudb-parquet | q3 | 0.003572 | 0.003675 | 0.000117 | 0.004935 | 0.013093 | 20.27 |
| 1m | rudb-parquet | q4 | 0.002722 | 0.002784 | 0.000121 | 0.004049 | 0.010180 | 30.66 |
| 1m | rudb-parquet | q5 | 0.006076 | 0.006211 | 0.000147 | 0.007539 | 0.028262 | 33.49 |
| 1m | rudb-parquet | q6 | 0.018303 | 0.018779 | 0.000384 | 0.020149 | 0.072621 | 45.81 |
| 1m | rudb-parquet | q7 | 0.001923 | 0.001837 | 0.000072 | 0.003063 | 0.005411 | 16.68 |
| 1m | rudb-parquet | q8 | 0.002153 | 0.002138 | 0.000188 | 0.003422 | 0.006244 | 16.59 |
| 1m | rudb-parquet | q9 | 0.008812 | 0.008802 | 0.000280 | 0.010179 | 0.036214 | 45.56 |
| 1m | rudb-parquet | q10 | 0.012688 | 0.012674 | 0.000493 | 0.014086 | 0.052271 | 54.29 |
| 1m | rudb-parquet | q11 | 0.005950 | 0.005960 | 0.000236 | 0.007300 | 0.022624 | 33.17 |
| 1m | rudb-parquet | q12 | 0.006802 | 0.006794 | 0.000113 | 0.008134 | 0.026204 | 35.62 |
| 1m | rudb-parquet | q13 | 0.015572 | 0.015776 | 0.000850 | 0.017175 | 0.066582 | 46.16 |
| 1m | rudb-parquet | q14 | 0.021507 | 0.021768 | 0.001106 | 0.023195 | 0.088886 | 70.38 |
| 1m | rudb-parquet | q15 | 0.017408 | 0.017026 | 0.000663 | 0.018466 | 0.074400 | 49.53 |
| 1m | rudb-parquet | q16 | 0.005392 | 0.005400 | 0.003192 | 0.006817 | 0.022280 | 38.56 |
| 1m | rudb-parquet | q17 | 0.077113 | 0.056554 | 0.033017 | 0.058478 | 0.201542 | 106.88 |
| 1m | rudb-parquet | q18 | 0.014056 | 0.013118 | 0.000445 | 0.014562 | 0.057177 | 38.64 |
| 1m | rudb-parquet | q19 | 0.059000 | 0.060216 | 0.001785 | 0.065154 | 0.214591 | 131.32 |
| 1m | rudb-parquet | q20 | 0.002767 | 0.002722 | 0.000045 | 0.004042 | 0.010280 | 29.25 |
| 1m | rudb-parquet | q21 | 0.063933 | 0.066989 | 0.008696 | 0.070272 | 0.269786 | 116.12 |
| 1m | rudb-parquet | q22 | 0.082644 | 0.075340 | 0.002838 | 0.078173 | 0.302256 | 106.45 |
| 1m | rudb-parquet | q23 | 0.138711 | 0.164581 | 0.050499 | 0.170101 | 0.619841 | 208.14 |
| 1m | rudb-parquet | q24 | 0.148227 | 0.134357 | 0.005275 | 0.138312 | 1.231428 | 310.05 |
| 1m | rudb-parquet | q25 | 0.016739 | 0.016868 | 0.000261 | 0.018238 | 0.070805 | 39.84 |
| 1m | rudb-parquet | q26 | 0.010919 | 0.010831 | 0.000133 | 0.012134 | 0.046217 | 27.56 |
| 1m | rudb-parquet | q27 | 0.017101 | 0.016922 | 0.000270 | 0.018339 | 0.071573 | 39.85 |
| 1m | rudb-parquet | q28 | 0.062047 | 0.065712 | 0.003639 | 0.069116 | 0.266546 | 120.85 |
| 1m | rudb-parquet | q29 | 0.094050 | 0.096047 | 0.051427 | 0.101344 | 0.388971 | 149.07 |
| 1m | rudb-parquet | q30 | 0.004930 | 0.004906 | 0.000050 | 0.006245 | 0.013329 | 18.38 |
| 1m | rudb-parquet | q31 | 0.015691 | 0.016380 | 0.000965 | 0.017817 | 0.067638 | 46.65 |
| 1m | rudb-parquet | q32 | 0.015703 | 0.015637 | 0.000893 | 0.017011 | 0.068124 | 55.35 |
| 1m | rudb-parquet | q33 | 0.010453 | 0.010267 | 0.000150 | 0.011733 | 0.044613 | 56.53 |
| 1m | rudb-parquet | q34 | 0.099315 | 0.101383 | 0.005211 | 0.108344 | 0.397397 | 233.46 |
| 1m | rudb-parquet | q35 | 0.114324 | 0.102331 | 0.004849 | 0.109792 | 0.405314 | 233.24 |
| 1m | rudb-parquet | q36 | 0.011120 | 0.007411 | 0.004121 | 0.009464 | 0.029938 | 33.36 |
| 1m | rudb-parquet | q37 | 0.047920 | 0.058118 | 0.027970 | 0.060767 | 0.103865 | 55.84 |
| 1m | rudb-parquet | q38 | 0.047546 | 0.046624 | 0.001484 | 0.048682 | 0.083716 | 49.81 |
| 1m | rudb-parquet | q39 | 0.047640 | 0.047863 | 0.002894 | 0.050775 | 0.081381 | 55.95 |
| 1m | rudb-parquet | q40 | 0.077751 | 0.079582 | 0.005137 | 0.082021 | 0.152496 | 80.88 |
| 1m | rudb-parquet | q41 | 0.006473 | 0.006460 | 0.000156 | 0.007704 | 0.012535 | 28.49 |
| 1m | rudb-parquet | q42 | 0.006473 | 0.006653 | 0.000399 | 0.007946 | 0.013145 | 26.56 |
| 1m | rudb-parquet | q43 | 0.006236 | 0.006322 | 0.000096 | 0.007567 | 0.012436 | 23.26 |
| 10m | duckdb-native | q1 | 0.001000 | 0.001000 | 0.001000 | 0.010906 | 0.012421 | 27.14 |
| 10m | duckdb-native | q2 | 0.009000 | 0.004000 | 0.000000 | 0.014278 | 0.026294 | 50.66 |
| 10m | duckdb-native | q3 | 0.013000 | 0.008000 | 0.008000 | 0.019357 | 0.049099 | 72.61 |
| 10m | duckdb-native | q4 | 0.033000 | 0.012000 | 0.002000 | 0.023761 | 0.077741 | 98.39 |
| 10m | duckdb-native | q5 | 0.064000 | 0.064000 | 0.008000 | 0.076977 | 0.380244 | 219.28 |
| 10m | duckdb-native | q6 | 0.070000 | 0.061000 | 0.003000 | 0.074312 | 0.360425 | 227.28 |
| 10m | duckdb-native | q7 | 0.002000 | 0.002000 | 0.000000 | 0.012144 | 0.013851 | 28.92 |
| 10m | duckdb-native | q8 | 0.004000 | 0.004000 | 0.001000 | 0.014744 | 0.027978 | 52.67 |
| 10m | duckdb-native | q9 | 0.087000 | 0.104000 | 0.038000 | 0.118614 | 0.599403 | 268.56 |
| 10m | duckdb-native | q10 | 0.113000 | 0.109000 | 0.018000 | 0.124039 | 0.637583 | 315.09 |
| 10m | duckdb-native | q11 | 0.032000 | 0.026000 | 0.001000 | 0.037945 | 0.152007 | 160.14 |
| 10m | duckdb-native | q12 | 0.031000 | 0.029000 | 0.006000 | 0.040750 | 0.165740 | 169.10 |
| 10m | duckdb-native | q13 | 0.074000 | 0.055000 | 0.031000 | 0.068915 | 0.300048 | 231.72 |
| 10m | duckdb-native | q14 | 0.101000 | 0.087000 | 0.002000 | 0.102068 | 0.507815 | 381.61 |
| 10m | duckdb-native | q15 | 0.056000 | 0.053000 | 0.001000 | 0.066118 | 0.313887 | 251.63 |
| 10m | duckdb-native | q16 | 0.075000 | 0.074000 | 0.041000 | 0.089305 | 0.447839 | 272.42 |
| 10m | duckdb-native | q17 | 0.148000 | 0.150000 | 0.035000 | 0.169711 | 0.905841 | 667.00 |
| 10m | duckdb-native | q18 | 0.188000 | 0.125000 | 0.042000 | 0.145422 | 0.746967 | 665.23 |
| 10m | duckdb-native | q19 | 0.230000 | 0.216000 | 0.007000 | 0.244585 | 1.307776 | 941.46 |
| 10m | duckdb-native | q20 | 0.008000 | 0.008000 | 0.001000 | 0.019221 | 0.051411 | 96.57 |
| 10m | duckdb-native | q21 | 0.394000 | 0.156000 | 0.016000 | 0.177477 | 0.905011 | 621.86 |
| 10m | duckdb-native | q22 | 0.129000 | 0.128000 | 0.009000 | 0.149808 | 0.759041 | 683.48 |
| 10m | duckdb-native | q23 | 0.198000 | 0.142000 | 0.021000 | 0.162240 | 0.810965 | 859.86 |
| 10m | duckdb-native | q24 | 0.141000 | 0.139000 | 0.063000 | 0.162781 | 0.649025 | 558.48 |
| 10m | duckdb-native | q25 | 0.010000 | 0.011000 | 0.000000 | 0.021836 | 0.057883 | 64.35 |
| 10m | duckdb-native | q26 | 0.055000 | 0.024000 | 0.025000 | 0.034846 | 0.137056 | 88.90 |
| 10m | duckdb-native | q27 | 0.010000 | 0.011000 | 0.011000 | 0.022772 | 0.061195 | 64.72 |
| 10m | duckdb-native | q28 | 0.130000 | 0.114000 | 0.016000 | 0.130024 | 0.641003 | 643.01 |
| 10m | duckdb-native | q29 | 0.962000 | 0.931000 | 0.013000 | 0.961846 | 5.339800 | 763.93 |
| 10m | duckdb-native | q30 | 0.009000 | 0.009000 | 0.000000 | 0.019772 | 0.043364 | 54.70 |
| 10m | duckdb-native | q31 | 0.061000 | 0.052000 | 0.002000 | 0.066190 | 0.305408 | 269.09 |
| 10m | duckdb-native | q32 | 0.081000 | 0.065000 | 0.006000 | 0.080498 | 0.378785 | 369.57 |
| 10m | duckdb-native | q33 | 0.207000 | 0.215000 | 0.029000 | 0.237719 | 1.287603 | 863.43 |
| 10m | duckdb-native | q34 | 0.544000 | 0.265000 | 0.035000 | 0.302731 | 1.610106 | 1472.24 |
| 10m | duckdb-native | q35 | 0.281000 | 0.283000 | 0.010000 | 0.326308 | 1.743969 | 1499.73 |
| 10m | duckdb-native | q36 | 0.084000 | 0.090000 | 0.001000 | 0.104418 | 0.531374 | 301.12 |
| 10m | duckdb-native | q37 | 0.012000 | 0.011000 | 0.001000 | 0.021828 | 0.038525 | 53.59 |
| 10m | duckdb-native | q38 | 0.008000 | 0.008000 | 0.001000 | 0.018352 | 0.030779 | 42.62 |
| 10m | duckdb-native | q39 | 0.005000 | 0.006000 | 0.001000 | 0.016172 | 0.026024 | 42.06 |
| 10m | duckdb-native | q40 | 0.018000 | 0.019000 | 0.001000 | 0.029626 | 0.061125 | 75.98 |
| 10m | duckdb-native | q41 | 0.007000 | 0.004000 | 0.000000 | 0.014640 | 0.022508 | 40.96 |
| 10m | duckdb-native | q42 | 0.005000 | 0.004000 | 0.000000 | 0.014777 | 0.022756 | 39.70 |
| 10m | duckdb-native | q43 | 0.004000 | 0.004000 | 0.000000 | 0.014813 | 0.023312 | 36.34 |
| 10m | rudb-native | q1 | 0.000329 | 0.000339 | 0.000049 | 0.003859 | 0.003829 | 13.26 |
| 10m | rudb-native | q2 | 0.000354 | 0.000363 | 0.000021 | 0.003970 | 0.003930 | 13.45 |
| 10m | rudb-native | q3 | 0.000383 | 0.000386 | 0.000040 | 0.004042 | 0.003999 | 13.56 |
| 10m | rudb-native | q4 | 0.000366 | 0.000348 | 0.000016 | 0.004033 | 0.003996 | 13.30 |
| 10m | rudb-native | q5 | 0.000335 | 0.000338 | 0.000125 | 0.004056 | 0.004025 | 13.26 |
| 10m | rudb-native | q6 | 0.000357 | 0.000338 | 0.000021 | 0.003910 | 0.003883 | 13.32 |
| 10m | rudb-native | q7 | 0.000323 | 0.000347 | 0.000041 | 0.003957 | 0.003932 | 13.34 |
| 10m | rudb-native | q8 | 0.001922 | 0.001877 | 0.000063 | 0.005582 | 0.011759 | 34.84 |
| 10m | rudb-native | q9 | 0.033531 | 0.033493 | 0.022434 | 0.038017 | 0.194360 | 138.94 |
| 10m | rudb-native | q10 | 0.044234 | 0.042941 | 0.000928 | 0.046996 | 0.245449 | 163.23 |
| 10m | rudb-native | q11 | 0.008198 | 0.008431 | 0.000459 | 0.012261 | 0.047858 | 52.46 |
| 10m | rudb-native | q12 | 0.010098 | 0.010199 | 0.000432 | 0.014064 | 0.057335 | 55.29 |
| 10m | rudb-native | q13 | 0.006311 | 0.006565 | 0.000788 | 0.010587 | 0.029299 | 40.78 |
| 10m | rudb-native | q14 | 0.022326 | 0.019930 | 0.002426 | 0.024071 | 0.103951 | 78.80 |
| 10m | rudb-native | q15 | 0.014570 | 0.014774 | 0.001088 | 0.018793 | 0.080073 | 67.08 |
| 10m | rudb-native | q16 | 0.000468 | 0.000488 | 0.000130 | 0.004268 | 0.004216 | 14.24 |
| 10m | rudb-native | q17 | 0.080826 | 0.055330 | 0.001421 | 0.059273 | 0.291268 | 220.12 |
| 10m | rudb-native | q18 | 0.016813 | 0.003099 | 0.000089 | 0.006928 | 0.006898 | 71.18 |
| 10m | rudb-native | q19 | 0.072655 | 0.072392 | 0.006055 | 0.087016 | 0.424895 | 293.71 |
| 10m | rudb-native | q20 | 0.006405 | 0.001479 | 0.000166 | 0.005437 | 0.009534 | 35.45 |
| 10m | rudb-native | q21 | 0.220419 | 0.069795 | 0.006389 | 0.076375 | 0.367779 | 256.54 |
| 10m | rudb-native | q22 | 0.080778 | 0.080682 | 0.006390 | 0.086724 | 0.383489 | 230.62 |
| 10m | rudb-native | q23 | 0.084218 | 0.070022 | 0.001111 | 0.075458 | 0.225708 | 154.09 |
| 10m | rudb-native | q24 | 0.061603 | 0.058599 | 0.022729 | 0.063809 | 0.202969 | 284.18 |
| 10m | rudb-native | q25 | 0.003473 | 0.003490 | 0.002717 | 0.007790 | 0.011641 | 97.08 |
| 10m | rudb-native | q26 | 0.052610 | 0.011490 | 0.011502 | 0.015769 | 0.039846 | 43.76 |
| 10m | rudb-native | q27 | 0.006984 | 0.007624 | 0.003621 | 0.011908 | 0.016261 | 100.36 |
| 10m | rudb-native | q28 | 0.066557 | 0.021489 | 0.002060 | 0.025893 | 0.062487 | 56.55 |
| 10m | rudb-native | q29 | 0.234216 | 0.226833 | 0.009867 | 0.237943 | 1.151421 | 276.93 |
| 10m | rudb-native | q30 | 0.001936 | 0.001987 | 0.000100 | 0.005969 | 0.005924 | 14.35 |
| 10m | rudb-native | q31 | 0.023386 | 0.019680 | 0.001923 | 0.024040 | 0.109681 | 66.55 |
| 10m | rudb-native | q32 | 0.001646 | 0.001663 | 0.000123 | 0.005658 | 0.005618 | 16.57 |
| 10m | rudb-native | q33 | 0.000612 | 0.000611 | 0.000053 | 0.004489 | 0.004459 | 14.06 |
| 10m | rudb-native | q34 | 0.000981 | 0.000569 | 0.000048 | 0.004431 | 0.004398 | 13.82 |
| 10m | rudb-native | q35 | 0.017872 | 0.015183 | 0.000601 | 0.019493 | 0.058554 | 80.04 |
| 10m | rudb-native | q36 | 0.000643 | 0.000635 | 0.000026 | 0.004546 | 0.004510 | 14.12 |
| 10m | rudb-native | q37 | 0.008656 | 0.006109 | 0.000264 | 0.010637 | 0.013451 | 30.07 |
| 10m | rudb-native | q38 | 0.005651 | 0.004546 | 0.000143 | 0.008657 | 0.011102 | 24.82 |
| 10m | rudb-native | q39 | 0.006121 | 0.005914 | 0.000406 | 0.010365 | 0.012093 | 27.25 |
| 10m | rudb-native | q40 | 0.017650 | 0.017362 | 0.000777 | 0.022120 | 0.040582 | 45.70 |
| 10m | rudb-native | q41 | 0.002167 | 0.001986 | 0.000399 | 0.006021 | 0.008445 | 20.58 |
| 10m | rudb-native | q42 | 0.002390 | 0.002346 | 0.000144 | 0.006351 | 0.008690 | 20.99 |
| 10m | rudb-native | q43 | 0.001837 | 0.001859 | 0.000082 | 0.005914 | 0.009206 | 20.87 |
| 10m | duckdb-parquet | q1 | 0.184000 | 0.127000 | 0.062000 | 0.248294 | 0.249451 | 150.41 |
| 10m | duckdb-parquet | q2 | 0.131000 | 0.138000 | 0.006000 | 0.267876 | 0.288597 | 159.16 |
| 10m | duckdb-parquet | q3 | 0.153000 | 0.180000 | 0.063000 | 0.341408 | 0.407975 | 174.66 |
| 10m | duckdb-parquet | q4 | 0.159000 | 0.140000 | 0.002000 | 0.263176 | 0.307594 | 159.42 |
| 10m | duckdb-parquet | q5 | 0.182000 | 0.184000 | 0.042000 | 0.313789 | 0.604560 | 278.68 |
| 10m | duckdb-parquet | q6 | 0.188000 | 0.190000 | 0.020000 | 0.312053 | 0.633117 | 316.92 |
| 10m | duckdb-parquet | q7 | 0.136000 | 0.134000 | 0.015000 | 0.253749 | 0.298602 | 162.51 |
| 10m | duckdb-parquet | q8 | 0.132000 | 0.130000 | 0.003000 | 0.247553 | 0.278038 | 160.48 |
| 10m | duckdb-parquet | q9 | 0.198000 | 0.200000 | 0.100000 | 0.347270 | 0.743331 | 317.00 |
| 10m | duckdb-parquet | q10 | 0.217000 | 0.219000 | 0.057000 | 0.344661 | 0.823530 | 344.66 |
| 10m | duckdb-parquet | q11 | 0.232000 | 0.152000 | 0.017000 | 0.290312 | 0.400280 | 206.74 |
| 10m | duckdb-parquet | q12 | 0.160000 | 0.163000 | 0.022000 | 0.300865 | 0.444081 | 219.05 |
| 10m | duckdb-parquet | q13 | 0.302000 | 0.228000 | 0.111000 | 0.380512 | 0.699074 | 323.18 |
| 10m | duckdb-parquet | q14 | 0.363000 | 0.230000 | 0.034000 | 0.360642 | 0.866475 | 411.93 |
| 10m | duckdb-parquet | q15 | 0.203000 | 0.200000 | 0.004000 | 0.328824 | 0.686167 | 336.42 |
| 10m | duckdb-parquet | q16 | 0.274000 | 0.262000 | 0.094000 | 0.395007 | 0.730238 | 329.74 |
| 10m | duckdb-parquet | q17 | 0.285000 | 0.285000 | 0.052000 | 0.415955 | 1.225560 | 685.22 |
| 10m | duckdb-parquet | q18 | 0.264000 | 0.266000 | 0.027000 | 0.399086 | 1.077621 | 685.54 |
| 10m | duckdb-parquet | q19 | 0.372000 | 0.367000 | 0.359000 | 0.507340 | 1.718111 | 934.23 |
| 10m | duckdb-parquet | q20 | 0.139000 | 0.138000 | 0.052000 | 0.263493 | 0.291968 | 156.28 |
| 10m | duckdb-parquet | q21 | 0.662000 | 0.330000 | 0.026000 | 0.478263 | 1.346501 | 217.92 |
| 10m | duckdb-parquet | q22 | 0.300000 | 0.283000 | 0.023000 | 0.413541 | 1.164255 | 251.96 |
| 10m | duckdb-parquet | q23 | 0.339000 | 0.322000 | 0.067000 | 0.444798 | 1.394277 | 240.04 |
| 10m | duckdb-parquet | q24 | 0.509000 | 0.436000 | 0.054000 | 0.569717 | 1.578193 | 360.61 |
| 10m | duckdb-parquet | q25 | 0.285000 | 0.285000 | 0.084000 | 0.421015 | 0.689001 | 291.46 |
| 10m | duckdb-parquet | q26 | 0.517000 | 0.174000 | 0.205000 | 0.304596 | 0.504266 | 171.37 |
| 10m | duckdb-parquet | q27 | 0.185000 | 0.186000 | 0.019000 | 0.316113 | 0.581531 | 178.72 |
| 10m | duckdb-parquet | q28 | 0.279000 | 0.267000 | 0.007000 | 0.394232 | 1.082986 | 259.98 |
| 10m | duckdb-parquet | q29 | 1.071000 | 1.080000 | 0.141000 | 1.221122 | 5.779400 | 508.53 |
| 10m | duckdb-parquet | q30 | 0.146000 | 0.145000 | 0.004000 | 0.274399 | 0.317874 | 166.01 |
| 10m | duckdb-parquet | q31 | 0.203000 | 0.204000 | 0.004000 | 0.334087 | 0.704629 | 294.84 |
| 10m | duckdb-parquet | q32 | 0.209000 | 0.211000 | 0.010000 | 0.339044 | 0.743036 | 318.18 |
| 10m | duckdb-parquet | q33 | 0.397000 | 0.354000 | 0.092000 | 0.503288 | 1.587863 | 850.33 |
| 10m | duckdb-parquet | q34 | 0.514000 | 0.429000 | 0.030000 | 0.570936 | 2.108052 | 1150.91 |
| 10m | duckdb-parquet | q35 | 0.451000 | 0.448000 | 0.003000 | 0.592808 | 2.220939 | 1238.29 |
| 10m | duckdb-parquet | q36 | 0.208000 | 0.209000 | 0.004000 | 0.339482 | 0.761564 | 385.09 |
| 10m | duckdb-parquet | q37 | 0.190000 | 0.147000 | 0.002000 | 0.275524 | 0.320248 | 184.81 |
| 10m | duckdb-parquet | q38 | 0.147000 | 0.146000 | 0.009000 | 0.273039 | 0.316347 | 172.75 |
| 10m | duckdb-parquet | q39 | 0.150000 | 0.146000 | 0.002000 | 0.270667 | 0.312693 | 175.25 |
| 10m | duckdb-parquet | q40 | 0.149000 | 0.148000 | 0.002000 | 0.274459 | 0.336935 | 202.11 |
| 10m | duckdb-parquet | q41 | 0.148000 | 0.142000 | 0.004000 | 0.270012 | 0.299697 | 165.56 |
| 10m | duckdb-parquet | q42 | 0.150000 | 0.147000 | 0.002000 | 0.272912 | 0.312155 | 166.93 |
| 10m | duckdb-parquet | q43 | 0.144000 | 0.145000 | 0.004000 | 0.272664 | 0.313324 | 166.71 |
| 10m | rudb-parquet | q1 | 0.135112 | 0.105335 | 0.007222 | 0.106779 | 0.109181 | 118.89 |
| 10m | rudb-parquet | q2 | 0.102844 | 0.104939 | 0.002740 | 0.106567 | 0.127126 | 118.88 |
| 10m | rudb-parquet | q3 | 0.111496 | 0.121219 | 0.061614 | 0.122746 | 0.158329 | 118.88 |
| 10m | rudb-parquet | q4 | 0.124515 | 0.114783 | 0.012177 | 0.116246 | 0.145439 | 118.88 |
| 10m | rudb-parquet | q5 | 0.154144 | 0.155864 | 0.050108 | 0.159381 | 0.363728 | 230.62 |
| 10m | rudb-parquet | q6 | 0.216764 | 0.215229 | 0.027563 | 0.225330 | 0.649843 | 270.54 |
| 10m | rudb-parquet | q7 | 0.108790 | 0.107827 | 0.022975 | 0.109353 | 0.125420 | 118.91 |
| 10m | rudb-parquet | q8 | 0.106247 | 0.105614 | 0.005715 | 0.107072 | 0.127632 | 118.89 |
| 10m | rudb-parquet | q9 | 0.133719 | 0.145637 | 0.070339 | 0.152565 | 0.318014 | 215.62 |
| 10m | rudb-parquet | q10 | 0.150668 | 0.155426 | 0.071759 | 0.162726 | 0.387704 | 219.52 |
| 10m | rudb-parquet | q11 | 0.131794 | 0.138900 | 0.010743 | 0.143702 | 0.256646 | 129.30 |
| 10m | rudb-parquet | q12 | 0.144566 | 0.148461 | 0.029744 | 0.153032 | 0.283033 | 127.40 |
| 10m | rudb-parquet | q13 | 0.308515 | 0.241434 | 0.070463 | 0.253862 | 0.662177 | 239.57 |
| 10m | rudb-parquet | q14 | 0.335680 | 0.252526 | 0.113463 | 0.262305 | 0.787851 | 322.95 |
| 10m | rudb-parquet | q15 | 0.209903 | 0.204243 | 0.005912 | 0.213817 | 0.634844 | 245.46 |
| 10m | rudb-parquet | q16 | 0.188160 | 0.160765 | 0.058167 | 0.162857 | 0.319126 | 218.28 |
| 10m | rudb-parquet | q17 | 0.474486 | 0.358456 | 0.285141 | 0.372760 | 1.331450 | 608.66 |
| 10m | rudb-parquet | q18 | 0.168189 | 0.176101 | 0.075676 | 0.181725 | 0.496491 | 118.72 |
| 10m | rudb-parquet | q19 | 0.535057 | 0.537793 | 0.008182 | 0.558087 | 1.943156 | 1087.82 |
| 10m | rudb-parquet | q20 | 0.117355 | 0.110161 | 0.005120 | 0.111579 | 0.127759 | 118.89 |
| 10m | rudb-parquet | q21 | 0.406142 | 0.321211 | 0.056250 | 0.330602 | 1.283497 | 165.11 |
| 10m | rudb-parquet | q22 | 0.305048 | 0.312116 | 0.096147 | 0.317311 | 1.302508 | 168.17 |
| 10m | rudb-parquet | q23 | 0.627156 | 0.515466 | 0.139204 | 0.520282 | 2.559779 | 176.41 |
| 10m | rudb-parquet | q24 | 0.433184 | 0.343335 | 0.007270 | 0.349500 | 1.209986 | 224.04 |
| 10m | rudb-parquet | q25 | 0.161448 | 0.163040 | 0.002695 | 0.167617 | 0.432666 | 123.19 |
| 10m | rudb-parquet | q26 | 0.415361 | 0.157631 | 0.177831 | 0.162645 | 0.382885 | 132.20 |
| 10m | rudb-parquet | q27 | 0.169086 | 0.182116 | 0.034348 | 0.186729 | 0.451992 | 124.18 |
| 10m | rudb-parquet | q28 | 0.268390 | 0.246906 | 0.009165 | 0.251345 | 0.976230 | 167.88 |
| 10m | rudb-parquet | q29 | 0.511540 | 0.537629 | 0.319788 | 0.555151 | 2.441256 | 462.17 |
| 10m | rudb-parquet | q30 | 0.111577 | 0.112750 | 0.002600 | 0.114265 | 0.135271 | 118.37 |
| 10m | rudb-parquet | q31 | 0.171778 | 0.165732 | 0.003260 | 0.171303 | 0.455593 | 144.30 |
| 10m | rudb-parquet | q32 | 0.172772 | 0.168415 | 0.004111 | 0.174084 | 0.480406 | 146.07 |
| 10m | rudb-parquet | q33 | 0.180203 | 0.183174 | 0.053409 | 0.198832 | 0.526294 | 289.32 |
| 10m | rudb-parquet | q34 | 0.784526 | 0.515255 | 0.082784 | 0.537632 | 2.163775 | 979.76 |
| 10m | rudb-parquet | q35 | 0.533180 | 0.518601 | 0.016356 | 0.541639 | 2.170649 | 1002.96 |
| 10m | rudb-parquet | q36 | 0.134312 | 0.138772 | 0.002830 | 0.140811 | 0.267321 | 219.33 |
| 10m | rudb-parquet | q37 | 0.119386 | 0.119368 | 0.002848 | 0.120930 | 0.143325 | 135.67 |
| 10m | rudb-parquet | q38 | 0.112928 | 0.113652 | 0.006695 | 0.115402 | 0.125758 | 119.06 |
| 10m | rudb-parquet | q39 | 0.118865 | 0.112130 | 0.007075 | 0.113622 | 0.124241 | 119.38 |
| 10m | rudb-parquet | q40 | 0.135365 | 0.129894 | 0.003104 | 0.131533 | 0.184124 | 161.36 |
| 10m | rudb-parquet | q41 | 0.114276 | 0.112562 | 0.001004 | 0.114103 | 0.117955 | 118.09 |
| 10m | rudb-parquet | q42 | 0.112171 | 0.112637 | 0.003882 | 0.114181 | 0.117614 | 118.09 |
| 10m | rudb-parquet | q43 | 0.109256 | 0.108813 | 0.003330 | 0.110339 | 0.114768 | 118.08 |
