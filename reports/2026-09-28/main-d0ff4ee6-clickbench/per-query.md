# ClickBench measurement audit

All 43 SQL queries are attempted on DuckDB native, rudb native, DuckDB Parquet, and rudb Parquet. 7 hot repetitions are required for a complete row. A failed repetition invalidates that query; successful fragments are never averaged into a result.

First execution is not disk-cold: the page cache is not flushed. Each repetition uses a fresh process. Query seconds are the CLI timer, including result rendering; wall and CPU seconds and peak RSS cover the whole child process. CPU and RSS come from wait4 for that child. RSS is the maximum resident set, not allocated bytes or an incremental memory delta. Both native rows open a loaded single-file database. Both Parquet rows query the same source file with native mirroring disabled; metadata-only paths remain available. These are sample results, not official ClickBench scores.

| Size | Engine | Load wall (s) | Load CPU (s) | Load peak RSS (MiB) | Native bytes |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 0.052587 | 0.073628 | 61.39 | 1060864 |
| 1k | rudb-native | 0.078249 | 0.094288 | 38.73 | 1876635 |
| 10k | duckdb-native | 0.104947 | 0.171084 | 66.28 | 4206592 |
| 10k | rudb-native | 0.306879 | 0.408064 | 71.86 | 7732602 |
| 1m | duckdb-native | 3.401233 | 9.235901 | 1998.99 | 600846336 |
| 1m | rudb-native | 1.981064 | 11.764244 | 1097.77 | 246676843 |
| 10m | duckdb-native | 28.248399 | 72.536099 | 4771.66 | 2915053568 |
| 10m | rudb-native | 15.523454 | 115.611612 | 5190.93 | 2029473489 |

| Size | Engine | Complete / 43 | Query median sum (s) | Process wall median sum (s) | CPU median sum (s) | Peak RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 43 | 0.070000 | 0.515282 | 0.627526 | 33.74 |
| 1k | rudb-native | 43 | 0.028779 | 0.093811 | 0.092965 | 13.78 |
| 1k | duckdb-parquet | 43 | 0.088000 | 0.535779 | 0.655603 | 32.93 |
| 1k | rudb-parquet | 43 | 0.042574 | 0.103682 | 0.102281 | 13.21 |
| 10k | duckdb-native | 43 | 0.084000 | 0.533559 | 0.667720 | 37.02 |
| 10k | rudb-native | 43 | 0.038419 | 0.116506 | 0.117691 | 15.19 |
| 10k | duckdb-parquet | 43 | 0.123000 | 0.569654 | 0.711647 | 37.73 |
| 10k | rudb-parquet | 43 | 0.094353 | 0.154622 | 0.152858 | 16.12 |
| 1m | duckdb-native | 43 | 0.595000 | 1.054353 | 3.081289 | 219.20 |
| 1m | rudb-native | 43 | 0.205740 | 0.283097 | 0.663472 | 115.88 |
| 1m | duckdb-parquet | 43 | 0.864000 | 1.342395 | 4.078788 | 359.57 |
| 1m | rudb-parquet | 43 | 1.359255 | 1.454961 | 5.580280 | 311.36 |
| 10m | duckdb-native | 43 | 4.591000 | 5.522126 | 25.593652 | 1497.43 |
| 10m | rudb-native | 43 | 1.151354 | 1.341309 | 4.938099 | 276.31 |
| 10m | duckdb-parquet | 43 | 11.347000 | 17.338109 | 41.564153 | 1240.74 |
| 10m | rudb-parquet | 43 | 9.819751 | 10.165842 | 30.724755 | 1088.79 |

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
| 1k | duckdb-native | q1 | 0.001000 | 0.001000 | 0.001000 | 0.010280 | 0.011604 | 26.48 |
| 1k | duckdb-native | q2 | 0.000000 | 0.001000 | 0.000000 | 0.010317 | 0.011918 | 27.48 |
| 1k | duckdb-native | q3 | 0.001000 | 0.001000 | 0.001000 | 0.010218 | 0.011424 | 27.47 |
| 1k | duckdb-native | q4 | 0.000000 | 0.000000 | 0.001000 | 0.009998 | 0.011637 | 27.05 |
| 1k | duckdb-native | q5 | 0.001000 | 0.001000 | 0.000000 | 0.010655 | 0.013090 | 28.72 |
| 1k | duckdb-native | q6 | 0.001000 | 0.001000 | 0.000000 | 0.010436 | 0.012499 | 28.98 |
| 1k | duckdb-native | q7 | 0.001000 | 0.000000 | 0.001000 | 0.009854 | 0.011171 | 26.73 |
| 1k | duckdb-native | q8 | 0.001000 | 0.001000 | 0.000000 | 0.010732 | 0.012823 | 28.98 |
| 1k | duckdb-native | q9 | 0.001000 | 0.002000 | 0.001000 | 0.010932 | 0.014789 | 32.62 |
| 1k | duckdb-native | q10 | 0.002000 | 0.002000 | 0.000000 | 0.011417 | 0.015892 | 33.74 |
| 1k | duckdb-native | q11 | 0.001000 | 0.002000 | 0.001000 | 0.011426 | 0.015247 | 32.43 |
| 1k | duckdb-native | q12 | 0.001000 | 0.002000 | 0.001000 | 0.011286 | 0.015425 | 32.74 |
| 1k | duckdb-native | q13 | 0.002000 | 0.001000 | 0.000000 | 0.010619 | 0.013354 | 29.50 |
| 1k | duckdb-native | q14 | 0.001000 | 0.002000 | 0.000000 | 0.011603 | 0.015928 | 33.05 |
| 1k | duckdb-native | q15 | 0.001000 | 0.001000 | 0.000000 | 0.010866 | 0.013670 | 30.68 |
| 1k | duckdb-native | q16 | 0.001000 | 0.001000 | 0.001000 | 0.010775 | 0.013244 | 31.05 |
| 1k | duckdb-native | q17 | 0.002000 | 0.002000 | 0.001000 | 0.010736 | 0.013280 | 31.80 |
| 1k | duckdb-native | q18 | 0.002000 | 0.001000 | 0.000000 | 0.010446 | 0.012508 | 31.21 |
| 1k | duckdb-native | q19 | 0.002000 | 0.002000 | 0.001000 | 0.010993 | 0.014105 | 32.36 |
| 1k | duckdb-native | q20 | 0.000000 | 0.001000 | 0.001000 | 0.013105 | 0.014938 | 26.98 |
| 1k | duckdb-native | q21 | 0.001000 | 0.001000 | 0.001000 | 0.012773 | 0.013858 | 27.62 |
| 1k | duckdb-native | q22 | 0.001000 | 0.002000 | 0.001000 | 0.012340 | 0.014710 | 28.71 |
| 1k | duckdb-native | q23 | 0.001000 | 0.001000 | 0.000000 | 0.011286 | 0.013374 | 28.74 |
| 1k | duckdb-native | q24 | 0.005000 | 0.005000 | 0.001000 | 0.014781 | 0.019372 | 33.47 |
| 1k | duckdb-native | q25 | 0.001000 | 0.002000 | 0.001000 | 0.011681 | 0.014284 | 29.71 |
| 1k | duckdb-native | q26 | 0.001000 | 0.001000 | 0.000000 | 0.010773 | 0.012077 | 27.04 |
| 1k | duckdb-native | q27 | 0.001000 | 0.001000 | 0.000000 | 0.010589 | 0.012117 | 27.62 |
| 1k | duckdb-native | q28 | 0.002000 | 0.002000 | 0.001000 | 0.011026 | 0.013840 | 29.93 |
| 1k | duckdb-native | q29 | 0.002000 | 0.002000 | 0.000000 | 0.011529 | 0.015024 | 30.41 |
| 1k | duckdb-native | q30 | 0.004000 | 0.004000 | 0.001000 | 0.013210 | 0.015702 | 30.73 |
| 1k | duckdb-native | q31 | 0.002000 | 0.002000 | 0.000000 | 0.011295 | 0.014389 | 32.30 |
| 1k | duckdb-native | q32 | 0.002000 | 0.003000 | 0.005000 | 0.024601 | 0.025498 | 32.30 |
| 1k | duckdb-native | q33 | 0.002000 | 0.003000 | 0.001000 | 0.021063 | 0.022907 | 32.53 |
| 1k | duckdb-native | q34 | 0.002000 | 0.002000 | 0.001000 | 0.016611 | 0.020838 | 30.49 |
| 1k | duckdb-native | q35 | 0.002000 | 0.002000 | 0.001000 | 0.011750 | 0.015353 | 30.99 |
| 1k | duckdb-native | q36 | 0.002000 | 0.002000 | 0.001000 | 0.011682 | 0.014575 | 31.83 |
| 1k | duckdb-native | q37 | 0.001000 | 0.002000 | 0.000000 | 0.012457 | 0.015582 | 30.30 |
| 1k | duckdb-native | q38 | 0.002000 | 0.001000 | 0.001000 | 0.012131 | 0.014958 | 30.04 |
| 1k | duckdb-native | q39 | 0.001000 | 0.001000 | 0.000000 | 0.011337 | 0.013284 | 27.99 |
| 1k | duckdb-native | q40 | 0.002000 | 0.002000 | 0.001000 | 0.012157 | 0.015357 | 30.87 |
| 1k | duckdb-native | q41 | 0.002000 | 0.001000 | 0.000000 | 0.010887 | 0.013377 | 30.97 |
| 1k | duckdb-native | q42 | 0.001000 | 0.002000 | 0.001000 | 0.011121 | 0.013884 | 30.02 |
| 1k | duckdb-native | q43 | 0.001000 | 0.001000 | 0.001000 | 0.011508 | 0.014620 | 29.67 |
| 1k | rudb-native | q1 | 0.000296 | 0.000338 | 0.000031 | 0.001721 | 0.001696 | 11.01 |
| 1k | rudb-native | q2 | 0.000358 | 0.000376 | 0.000012 | 0.001818 | 0.001792 | 10.96 |
| 1k | rudb-native | q3 | 0.000388 | 0.000384 | 0.000020 | 0.001741 | 0.001705 | 11.25 |
| 1k | rudb-native | q4 | 0.000351 | 0.000342 | 0.000003 | 0.001719 | 0.001695 | 10.96 |
| 1k | rudb-native | q5 | 0.000298 | 0.000334 | 0.000023 | 0.001712 | 0.001687 | 10.99 |
| 1k | rudb-native | q6 | 0.000340 | 0.000340 | 0.000014 | 0.001738 | 0.001710 | 10.91 |
| 1k | rudb-native | q7 | 0.000367 | 0.000348 | 0.000012 | 0.001717 | 0.001681 | 10.71 |
| 1k | rudb-native | q8 | 0.000585 | 0.000580 | 0.000013 | 0.002042 | 0.002017 | 12.85 |
| 1k | rudb-native | q9 | 0.000515 | 0.000573 | 0.000023 | 0.001985 | 0.001958 | 12.62 |
| 1k | rudb-native | q10 | 0.000879 | 0.000834 | 0.000013 | 0.002277 | 0.002253 | 13.34 |
| 1k | rudb-native | q11 | 0.000568 | 0.000604 | 0.000017 | 0.002043 | 0.002001 | 12.81 |
| 1k | rudb-native | q12 | 0.000659 | 0.000612 | 0.000037 | 0.002026 | 0.002004 | 12.85 |
| 1k | rudb-native | q13 | 0.000652 | 0.000739 | 0.000028 | 0.002230 | 0.002305 | 13.30 |
| 1k | rudb-native | q14 | 0.000642 | 0.000681 | 0.000038 | 0.002122 | 0.002096 | 13.07 |
| 1k | rudb-native | q15 | 0.000829 | 0.000859 | 0.000104 | 0.002301 | 0.002275 | 13.32 |
| 1k | rudb-native | q16 | 0.000411 | 0.000406 | 0.000036 | 0.001761 | 0.001737 | 11.51 |
| 1k | rudb-native | q17 | 0.000399 | 0.000424 | 0.000026 | 0.001763 | 0.001723 | 11.82 |
| 1k | rudb-native | q18 | 0.000427 | 0.000410 | 0.000016 | 0.001756 | 0.001733 | 11.82 |
| 1k | rudb-native | q19 | 0.000471 | 0.000489 | 0.000022 | 0.001830 | 0.001806 | 12.28 |
| 1k | rudb-native | q20 | 0.000385 | 0.000434 | 0.000137 | 0.001940 | 0.001891 | 11.25 |
| 1k | rudb-native | q21 | 0.000569 | 0.000592 | 0.000219 | 0.002120 | 0.002070 | 12.75 |
| 1k | rudb-native | q22 | 0.000962 | 0.000702 | 0.000293 | 0.002311 | 0.002256 | 13.07 |
| 1k | rudb-native | q23 | 0.000788 | 0.000777 | 0.000040 | 0.002313 | 0.002285 | 13.45 |
| 1k | rudb-native | q24 | 0.000647 | 0.000611 | 0.000035 | 0.002081 | 0.002052 | 12.07 |
| 1k | rudb-native | q25 | 0.000567 | 0.000556 | 0.000045 | 0.002007 | 0.001980 | 12.09 |
| 1k | rudb-native | q26 | 0.000514 | 0.000516 | 0.000057 | 0.001960 | 0.001934 | 12.10 |
| 1k | rudb-native | q27 | 0.000532 | 0.000537 | 0.000083 | 0.002013 | 0.001988 | 12.10 |
| 1k | rudb-native | q28 | 0.000785 | 0.000771 | 0.000100 | 0.002321 | 0.002294 | 13.56 |
| 1k | rudb-native | q29 | 0.000890 | 0.000912 | 0.000038 | 0.002359 | 0.002336 | 13.54 |
| 1k | rudb-native | q30 | 0.001940 | 0.001884 | 0.000065 | 0.003330 | 0.003303 | 12.53 |
| 1k | rudb-native | q31 | 0.000844 | 0.000865 | 0.000037 | 0.002290 | 0.002262 | 13.29 |
| 1k | rudb-native | q32 | 0.000634 | 0.000991 | 0.000164 | 0.003404 | 0.003339 | 12.21 |
| 1k | rudb-native | q33 | 0.000509 | 0.000854 | 0.000131 | 0.003338 | 0.003284 | 11.86 |
| 1k | rudb-native | q34 | 0.000898 | 0.000752 | 0.000241 | 0.002629 | 0.002587 | 12.05 |
| 1k | rudb-native | q35 | 0.000984 | 0.000795 | 0.000058 | 0.002359 | 0.002481 | 13.10 |
| 1k | rudb-native | q36 | 0.000765 | 0.000748 | 0.000095 | 0.002274 | 0.002245 | 12.86 |
| 1k | rudb-native | q37 | 0.000934 | 0.001027 | 0.000310 | 0.002648 | 0.002733 | 13.78 |
| 1k | rudb-native | q38 | 0.001190 | 0.001146 | 0.000161 | 0.002800 | 0.002892 | 13.66 |
| 1k | rudb-native | q39 | 0.000703 | 0.000678 | 0.000073 | 0.002160 | 0.002134 | 13.06 |
| 1k | rudb-native | q40 | 0.000773 | 0.000776 | 0.000026 | 0.002324 | 0.002295 | 13.41 |
| 1k | rudb-native | q41 | 0.000656 | 0.000706 | 0.000062 | 0.002087 | 0.002063 | 13.35 |
| 1k | rudb-native | q42 | 0.000749 | 0.000751 | 0.000016 | 0.002199 | 0.002174 | 13.28 |
| 1k | rudb-native | q43 | 0.000695 | 0.000723 | 0.000258 | 0.002243 | 0.002213 | 13.62 |
| 1k | duckdb-parquet | q1 | 0.001000 | 0.001000 | 0.000000 | 0.010847 | 0.012289 | 26.46 |
| 1k | duckdb-parquet | q2 | 0.001000 | 0.001000 | 0.000000 | 0.011083 | 0.012542 | 27.46 |
| 1k | duckdb-parquet | q3 | 0.001000 | 0.001000 | 0.001000 | 0.010592 | 0.012241 | 27.45 |
| 1k | duckdb-parquet | q4 | 0.001000 | 0.001000 | 0.000000 | 0.010537 | 0.012283 | 27.02 |
| 1k | duckdb-parquet | q5 | 0.002000 | 0.001000 | 0.001000 | 0.011060 | 0.013487 | 28.73 |
| 1k | duckdb-parquet | q6 | 0.002000 | 0.002000 | 0.000000 | 0.011169 | 0.013437 | 28.48 |
| 1k | duckdb-parquet | q7 | 0.001000 | 0.001000 | 0.000000 | 0.010552 | 0.012305 | 27.17 |
| 1k | duckdb-parquet | q8 | 0.002000 | 0.002000 | 0.001000 | 0.010986 | 0.013105 | 28.77 |
| 1k | duckdb-parquet | q9 | 0.002000 | 0.002000 | 0.000000 | 0.011566 | 0.015653 | 31.49 |
| 1k | duckdb-parquet | q10 | 0.002000 | 0.002000 | 0.001000 | 0.011988 | 0.016289 | 32.93 |
| 1k | duckdb-parquet | q11 | 0.002000 | 0.002000 | 0.000000 | 0.011484 | 0.014913 | 31.50 |
| 1k | duckdb-parquet | q12 | 0.002000 | 0.002000 | 0.000000 | 0.011692 | 0.015653 | 32.75 |
| 1k | duckdb-parquet | q13 | 0.002000 | 0.002000 | 0.000000 | 0.011235 | 0.013955 | 29.26 |
| 1k | duckdb-parquet | q14 | 0.003000 | 0.002000 | 0.000000 | 0.011916 | 0.016260 | 32.50 |
| 1k | duckdb-parquet | q15 | 0.002000 | 0.002000 | 0.000000 | 0.011292 | 0.014077 | 30.50 |
| 1k | duckdb-parquet | q16 | 0.001000 | 0.002000 | 0.000000 | 0.011439 | 0.013773 | 30.56 |
| 1k | duckdb-parquet | q17 | 0.002000 | 0.002000 | 0.001000 | 0.011199 | 0.013916 | 31.34 |
| 1k | duckdb-parquet | q18 | 0.001000 | 0.002000 | 0.001000 | 0.010949 | 0.013179 | 30.59 |
| 1k | duckdb-parquet | q19 | 0.002000 | 0.002000 | 0.000000 | 0.011496 | 0.014198 | 31.49 |
| 1k | duckdb-parquet | q20 | 0.001000 | 0.001000 | 0.001000 | 0.015817 | 0.017327 | 27.03 |
| 1k | duckdb-parquet | q21 | 0.002000 | 0.002000 | 0.001000 | 0.011669 | 0.013218 | 27.47 |
| 1k | duckdb-parquet | q22 | 0.002000 | 0.002000 | 0.001000 | 0.012320 | 0.014328 | 28.25 |
| 1k | duckdb-parquet | q23 | 0.002000 | 0.002000 | 0.001000 | 0.012434 | 0.014505 | 28.04 |
| 1k | duckdb-parquet | q24 | 0.006000 | 0.006000 | 0.002000 | 0.016530 | 0.022062 | 31.23 |
| 1k | duckdb-parquet | q25 | 0.002000 | 0.002000 | 0.001000 | 0.012536 | 0.015468 | 29.52 |
| 1k | duckdb-parquet | q26 | 0.001000 | 0.001000 | 0.000000 | 0.011136 | 0.013004 | 27.26 |
| 1k | duckdb-parquet | q27 | 0.002000 | 0.001000 | 0.001000 | 0.011108 | 0.012676 | 27.72 |
| 1k | duckdb-parquet | q28 | 0.001000 | 0.002000 | 0.000000 | 0.011408 | 0.014199 | 29.45 |
| 1k | duckdb-parquet | q29 | 0.003000 | 0.003000 | 0.001000 | 0.012033 | 0.015584 | 30.42 |
| 1k | duckdb-parquet | q30 | 0.004000 | 0.004000 | 0.000000 | 0.013859 | 0.016409 | 30.08 |
| 1k | duckdb-parquet | q31 | 0.002000 | 0.002000 | 0.000000 | 0.011842 | 0.014932 | 31.59 |
| 1k | duckdb-parquet | q32 | 0.002000 | 0.004000 | 0.004000 | 0.020924 | 0.025595 | 31.81 |
| 1k | duckdb-parquet | q33 | 0.004000 | 0.003000 | 0.001000 | 0.022095 | 0.025869 | 31.36 |
| 1k | duckdb-parquet | q34 | 0.003000 | 0.003000 | 0.001000 | 0.018215 | 0.021574 | 30.03 |
| 1k | duckdb-parquet | q35 | 0.003000 | 0.002000 | 0.001000 | 0.012248 | 0.016155 | 30.27 |
| 1k | duckdb-parquet | q36 | 0.002000 | 0.002000 | 0.000000 | 0.012150 | 0.015201 | 31.47 |
| 1k | duckdb-parquet | q37 | 0.002000 | 0.002000 | 0.001000 | 0.012216 | 0.015944 | 29.69 |
| 1k | duckdb-parquet | q38 | 0.002000 | 0.002000 | 0.000000 | 0.012500 | 0.015210 | 29.25 |
| 1k | duckdb-parquet | q39 | 0.002000 | 0.001000 | 0.001000 | 0.012048 | 0.013698 | 28.18 |
| 1k | duckdb-parquet | q40 | 0.002000 | 0.003000 | 0.001000 | 0.012514 | 0.015760 | 30.75 |
| 1k | duckdb-parquet | q41 | 0.002000 | 0.002000 | 0.000000 | 0.011332 | 0.014144 | 31.05 |
| 1k | duckdb-parquet | q42 | 0.002000 | 0.002000 | 0.000000 | 0.011542 | 0.014106 | 29.29 |
| 1k | duckdb-parquet | q43 | 0.002000 | 0.002000 | 0.000000 | 0.012223 | 0.015080 | 29.15 |
| 1k | rudb-parquet | q1 | 0.000596 | 0.000555 | 0.000100 | 0.001876 | 0.001851 | 11.30 |
| 1k | rudb-parquet | q2 | 0.000674 | 0.000658 | 0.000019 | 0.001996 | 0.001970 | 12.11 |
| 1k | rudb-parquet | q3 | 0.000623 | 0.000650 | 0.000063 | 0.001965 | 0.001933 | 11.82 |
| 1k | rudb-parquet | q4 | 0.000629 | 0.000580 | 0.000048 | 0.001882 | 0.001857 | 11.86 |
| 1k | rudb-parquet | q5 | 0.000604 | 0.000612 | 0.000049 | 0.001930 | 0.001906 | 11.53 |
| 1k | rudb-parquet | q6 | 0.000799 | 0.000790 | 0.000039 | 0.002121 | 0.002089 | 11.80 |
| 1k | rudb-parquet | q7 | 0.000596 | 0.000672 | 0.000038 | 0.002005 | 0.001979 | 12.38 |
| 1k | rudb-parquet | q8 | 0.000770 | 0.000720 | 0.000039 | 0.002056 | 0.002031 | 12.38 |
| 1k | rudb-parquet | q9 | 0.000722 | 0.000723 | 0.000030 | 0.002030 | 0.001997 | 11.86 |
| 1k | rudb-parquet | q10 | 0.000997 | 0.001010 | 0.000029 | 0.002339 | 0.002314 | 12.37 |
| 1k | rudb-parquet | q11 | 0.000686 | 0.000738 | 0.000048 | 0.002035 | 0.002005 | 12.12 |
| 1k | rudb-parquet | q12 | 0.000767 | 0.000744 | 0.000080 | 0.002060 | 0.002035 | 12.09 |
| 1k | rudb-parquet | q13 | 0.000792 | 0.000798 | 0.000045 | 0.002117 | 0.002094 | 12.12 |
| 1k | rudb-parquet | q14 | 0.000901 | 0.000826 | 0.000049 | 0.002168 | 0.002141 | 12.08 |
| 1k | rudb-parquet | q15 | 0.000723 | 0.000841 | 0.000037 | 0.002174 | 0.002144 | 12.07 |
| 1k | rudb-parquet | q16 | 0.000751 | 0.000728 | 0.000050 | 0.002022 | 0.001996 | 11.65 |
| 1k | rudb-parquet | q17 | 0.000829 | 0.000827 | 0.000021 | 0.002090 | 0.002068 | 11.87 |
| 1k | rudb-parquet | q18 | 0.000777 | 0.000752 | 0.000042 | 0.002006 | 0.001981 | 11.61 |
| 1k | rudb-parquet | q19 | 0.000905 | 0.000949 | 0.000068 | 0.002239 | 0.002214 | 12.68 |
| 1k | rudb-parquet | q20 | 0.000541 | 0.000563 | 0.000383 | 0.002020 | 0.001970 | 10.78 |
| 1k | rudb-parquet | q21 | 0.001283 | 0.001060 | 0.000170 | 0.002553 | 0.002513 | 12.35 |
| 1k | rudb-parquet | q22 | 0.001356 | 0.001175 | 0.000148 | 0.002603 | 0.002574 | 12.34 |
| 1k | rudb-parquet | q23 | 0.001699 | 0.001651 | 0.000482 | 0.003034 | 0.002999 | 12.37 |
| 1k | rudb-parquet | q24 | 0.001239 | 0.001239 | 0.000072 | 0.002703 | 0.002659 | 11.54 |
| 1k | rudb-parquet | q25 | 0.000763 | 0.000750 | 0.000031 | 0.002094 | 0.002069 | 11.96 |
| 1k | rudb-parquet | q26 | 0.000680 | 0.000672 | 0.000019 | 0.002008 | 0.001984 | 11.33 |
| 1k | rudb-parquet | q27 | 0.000740 | 0.000743 | 0.000047 | 0.002047 | 0.002024 | 11.96 |
| 1k | rudb-parquet | q28 | 0.001158 | 0.001159 | 0.000052 | 0.002526 | 0.002497 | 12.60 |
| 1k | rudb-parquet | q29 | 0.001273 | 0.001245 | 0.000028 | 0.002559 | 0.002532 | 12.87 |
| 1k | rudb-parquet | q30 | 0.002227 | 0.002227 | 0.000075 | 0.003628 | 0.003602 | 12.87 |
| 1k | rudb-parquet | q31 | 0.000965 | 0.001003 | 0.000064 | 0.002353 | 0.002328 | 12.18 |
| 1k | rudb-parquet | q32 | 0.000997 | 0.001629 | 0.000878 | 0.003817 | 0.003758 | 12.14 |
| 1k | rudb-parquet | q33 | 0.001571 | 0.001586 | 0.001028 | 0.004248 | 0.004081 | 12.12 |
| 1k | rudb-parquet | q34 | 0.001577 | 0.001679 | 0.000521 | 0.003822 | 0.003782 | 12.12 |
| 1k | rudb-parquet | q35 | 0.001737 | 0.001145 | 0.000080 | 0.002512 | 0.002487 | 12.12 |
| 1k | rudb-parquet | q36 | 0.000982 | 0.000815 | 0.000034 | 0.002201 | 0.002174 | 12.16 |
| 1k | rudb-parquet | q37 | 0.001234 | 0.001246 | 0.000076 | 0.002633 | 0.002605 | 12.97 |
| 1k | rudb-parquet | q38 | 0.001434 | 0.001280 | 0.000072 | 0.002734 | 0.002703 | 13.05 |
| 1k | rudb-parquet | q39 | 0.001489 | 0.001187 | 0.000035 | 0.002660 | 0.002629 | 12.37 |
| 1k | rudb-parquet | q40 | 0.001609 | 0.001593 | 0.000041 | 0.002978 | 0.002952 | 13.21 |
| 1k | rudb-parquet | q41 | 0.000906 | 0.000891 | 0.000088 | 0.002199 | 0.002174 | 12.93 |
| 1k | rudb-parquet | q42 | 0.000921 | 0.000915 | 0.000035 | 0.002308 | 0.002283 | 12.74 |
| 1k | rudb-parquet | q43 | 0.000856 | 0.000948 | 0.000076 | 0.002329 | 0.002297 | 12.97 |
| 10k | duckdb-native | q1 | 0.001000 | 0.001000 | 0.001000 | 0.010496 | 0.012039 | 26.47 |
| 10k | duckdb-native | q2 | 0.000000 | 0.001000 | 0.000000 | 0.011455 | 0.013297 | 27.54 |
| 10k | duckdb-native | q3 | 0.001000 | 0.001000 | 0.000000 | 0.013570 | 0.015501 | 28.29 |
| 10k | duckdb-native | q4 | 0.001000 | 0.001000 | 0.000000 | 0.015231 | 0.017174 | 27.23 |
| 10k | duckdb-native | q5 | 0.001000 | 0.001000 | 0.000000 | 0.013473 | 0.015981 | 29.87 |
| 10k | duckdb-native | q6 | 0.002000 | 0.001000 | 0.001000 | 0.011728 | 0.013922 | 29.41 |
| 10k | duckdb-native | q7 | 0.001000 | 0.001000 | 0.001000 | 0.009699 | 0.011089 | 26.73 |
| 10k | duckdb-native | q8 | 0.001000 | 0.001000 | 0.000000 | 0.010041 | 0.011873 | 28.93 |
| 10k | duckdb-native | q9 | 0.001000 | 0.002000 | 0.000000 | 0.011319 | 0.016032 | 33.99 |
| 10k | duckdb-native | q10 | 0.002000 | 0.002000 | 0.000000 | 0.011856 | 0.016874 | 35.72 |
| 10k | duckdb-native | q11 | 0.001000 | 0.002000 | 0.001000 | 0.010855 | 0.014301 | 32.49 |
| 10k | duckdb-native | q12 | 0.002000 | 0.002000 | 0.001000 | 0.011123 | 0.015241 | 33.24 |
| 10k | duckdb-native | q13 | 0.001000 | 0.001000 | 0.001000 | 0.010512 | 0.013413 | 29.99 |
| 10k | duckdb-native | q14 | 0.002000 | 0.002000 | 0.000000 | 0.011239 | 0.015743 | 33.18 |
| 10k | duckdb-native | q15 | 0.001000 | 0.001000 | 0.001000 | 0.010705 | 0.013673 | 30.73 |
| 10k | duckdb-native | q16 | 0.001000 | 0.001000 | 0.001000 | 0.010645 | 0.013765 | 32.55 |
| 10k | duckdb-native | q17 | 0.002000 | 0.002000 | 0.000000 | 0.011195 | 0.014895 | 33.79 |
| 10k | duckdb-native | q18 | 0.002000 | 0.002000 | 0.001000 | 0.010962 | 0.013210 | 33.29 |
| 10k | duckdb-native | q19 | 0.003000 | 0.003000 | 0.001000 | 0.011703 | 0.015834 | 34.80 |
| 10k | duckdb-native | q20 | 0.000000 | 0.001000 | 0.001000 | 0.009417 | 0.010678 | 26.73 |
| 10k | duckdb-native | q21 | 0.002000 | 0.001000 | 0.001000 | 0.010455 | 0.011905 | 28.23 |
| 10k | duckdb-native | q22 | 0.002000 | 0.002000 | 0.001000 | 0.011192 | 0.013484 | 29.74 |
| 10k | duckdb-native | q23 | 0.003000 | 0.003000 | 0.000000 | 0.012587 | 0.016593 | 32.52 |
| 10k | duckdb-native | q24 | 0.006000 | 0.006000 | 0.004000 | 0.016339 | 0.021699 | 37.02 |
| 10k | duckdb-native | q25 | 0.018000 | 0.003000 | 0.009000 | 0.038490 | 0.043694 | 30.22 |
| 10k | duckdb-native | q26 | 0.001000 | 0.002000 | 0.001000 | 0.020618 | 0.020217 | 28.22 |
| 10k | duckdb-native | q27 | 0.001000 | 0.001000 | 0.000000 | 0.010146 | 0.011637 | 28.22 |
| 10k | duckdb-native | q28 | 0.002000 | 0.002000 | 0.001000 | 0.011580 | 0.015080 | 31.23 |
| 10k | duckdb-native | q29 | 0.006000 | 0.006000 | 0.001000 | 0.015757 | 0.024146 | 32.23 |
| 10k | duckdb-native | q30 | 0.004000 | 0.004000 | 0.001000 | 0.012926 | 0.015031 | 30.73 |
| 10k | duckdb-native | q31 | 0.002000 | 0.002000 | 0.000000 | 0.011243 | 0.014402 | 32.84 |
| 10k | duckdb-native | q32 | 0.002000 | 0.002000 | 0.001000 | 0.011202 | 0.014072 | 32.98 |
| 10k | duckdb-native | q33 | 0.002000 | 0.002000 | 0.000000 | 0.011281 | 0.015034 | 34.47 |
| 10k | duckdb-native | q34 | 0.003000 | 0.002000 | 0.001000 | 0.011815 | 0.016126 | 33.96 |
| 10k | duckdb-native | q35 | 0.003000 | 0.003000 | 0.000000 | 0.012256 | 0.016836 | 34.24 |
| 10k | duckdb-native | q36 | 0.002000 | 0.002000 | 0.001000 | 0.010994 | 0.014397 | 33.10 |
| 10k | duckdb-native | q37 | 0.002000 | 0.002000 | 0.001000 | 0.011128 | 0.014220 | 31.27 |
| 10k | duckdb-native | q38 | 0.002000 | 0.002000 | 0.000000 | 0.011212 | 0.014252 | 31.30 |
| 10k | duckdb-native | q39 | 0.002000 | 0.002000 | 0.001000 | 0.010951 | 0.013909 | 30.74 |
| 10k | duckdb-native | q40 | 0.002000 | 0.002000 | 0.000000 | 0.011854 | 0.015863 | 33.18 |
| 10k | duckdb-native | q41 | 0.002000 | 0.001000 | 0.001000 | 0.010809 | 0.013478 | 32.29 |
| 10k | duckdb-native | q42 | 0.002000 | 0.002000 | 0.000000 | 0.010872 | 0.013907 | 31.24 |
| 10k | duckdb-native | q43 | 0.002000 | 0.001000 | 0.001000 | 0.010627 | 0.013203 | 29.73 |
| 10k | rudb-native | q1 | 0.000318 | 0.000365 | 0.000035 | 0.002126 | 0.002085 | 11.00 |
| 10k | rudb-native | q2 | 0.000554 | 0.000406 | 0.000022 | 0.002223 | 0.002191 | 10.96 |
| 10k | rudb-native | q3 | 0.000440 | 0.000616 | 0.000259 | 0.002985 | 0.002937 | 11.25 |
| 10k | rudb-native | q4 | 0.000551 | 0.000419 | 0.000126 | 0.002363 | 0.002293 | 10.99 |
| 10k | rudb-native | q5 | 0.000587 | 0.000509 | 0.000204 | 0.002451 | 0.002422 | 11.00 |
| 10k | rudb-native | q6 | 0.000386 | 0.000356 | 0.000173 | 0.001997 | 0.001974 | 10.90 |
| 10k | rudb-native | q7 | 0.000314 | 0.000359 | 0.000089 | 0.002000 | 0.001947 | 10.74 |
| 10k | rudb-native | q8 | 0.000697 | 0.000660 | 0.000103 | 0.002333 | 0.002420 | 13.09 |
| 10k | rudb-native | q9 | 0.000788 | 0.000806 | 0.000046 | 0.002489 | 0.002466 | 13.24 |
| 10k | rudb-native | q10 | 0.001188 | 0.001182 | 0.000037 | 0.002834 | 0.002812 | 13.80 |
| 10k | rudb-native | q11 | 0.001215 | 0.000694 | 0.000038 | 0.002376 | 0.002478 | 13.16 |
| 10k | rudb-native | q12 | 0.000776 | 0.000734 | 0.000028 | 0.002431 | 0.002550 | 13.25 |
| 10k | rudb-native | q13 | 0.000864 | 0.000919 | 0.000061 | 0.002682 | 0.002874 | 13.94 |
| 10k | rudb-native | q14 | 0.000954 | 0.000933 | 0.000032 | 0.002655 | 0.002788 | 13.58 |
| 10k | rudb-native | q15 | 0.001247 | 0.001172 | 0.000103 | 0.002889 | 0.002999 | 13.89 |
| 10k | rudb-native | q16 | 0.000920 | 0.000851 | 0.000028 | 0.002554 | 0.002583 | 12.85 |
| 10k | rudb-native | q17 | 0.001052 | 0.001098 | 0.000070 | 0.002835 | 0.002979 | 14.01 |
| 10k | rudb-native | q18 | 0.000737 | 0.000690 | 0.000036 | 0.002355 | 0.002331 | 12.28 |
| 10k | rudb-native | q19 | 0.001186 | 0.001188 | 0.000027 | 0.002920 | 0.003101 | 14.62 |
| 10k | rudb-native | q20 | 0.000410 | 0.000364 | 0.000025 | 0.001961 | 0.001939 | 11.25 |
| 10k | rudb-native | q21 | 0.000644 | 0.000673 | 0.000025 | 0.002292 | 0.002271 | 12.87 |
| 10k | rudb-native | q22 | 0.000969 | 0.000978 | 0.000107 | 0.002795 | 0.002855 | 13.43 |
| 10k | rudb-native | q23 | 0.002927 | 0.002842 | 0.000064 | 0.004683 | 0.004876 | 14.61 |
| 10k | rudb-native | q24 | 0.000782 | 0.000799 | 0.000439 | 0.002534 | 0.002511 | 12.68 |
| 10k | rudb-native | q25 | 0.010786 | 0.001321 | 0.000153 | 0.005546 | 0.004001 | 12.94 |
| 10k | rudb-native | q26 | 0.001191 | 0.001021 | 0.000072 | 0.003602 | 0.003496 | 12.55 |
| 10k | rudb-native | q27 | 0.000757 | 0.000802 | 0.000040 | 0.002410 | 0.002381 | 12.96 |
| 10k | rudb-native | q28 | 0.001189 | 0.001122 | 0.000054 | 0.002879 | 0.003075 | 14.69 |
| 10k | rudb-native | q29 | 0.001862 | 0.001877 | 0.000127 | 0.003687 | 0.004601 | 15.19 |
| 10k | rudb-native | q30 | 0.001879 | 0.001854 | 0.000030 | 0.003563 | 0.003532 | 12.56 |
| 10k | rudb-native | q31 | 0.001106 | 0.001099 | 0.000041 | 0.002837 | 0.002999 | 14.12 |
| 10k | rudb-native | q32 | 0.000715 | 0.000676 | 0.000040 | 0.002306 | 0.002285 | 12.93 |
| 10k | rudb-native | q33 | 0.000486 | 0.000501 | 0.000015 | 0.002138 | 0.002115 | 12.12 |
| 10k | rudb-native | q34 | 0.000611 | 0.000547 | 0.000027 | 0.002205 | 0.002183 | 12.03 |
| 10k | rudb-native | q35 | 0.001089 | 0.001145 | 0.000067 | 0.002895 | 0.003064 | 13.98 |
| 10k | rudb-native | q36 | 0.000568 | 0.000524 | 0.000021 | 0.002128 | 0.002106 | 12.57 |
| 10k | rudb-native | q37 | 0.000953 | 0.000996 | 0.000040 | 0.002752 | 0.002878 | 14.49 |
| 10k | rudb-native | q38 | 0.001230 | 0.001246 | 0.000024 | 0.002993 | 0.003123 | 14.76 |
| 10k | rudb-native | q39 | 0.000785 | 0.000783 | 0.000039 | 0.002534 | 0.002657 | 14.05 |
| 10k | rudb-native | q40 | 0.000924 | 0.000859 | 0.000057 | 0.002619 | 0.002679 | 14.10 |
| 10k | rudb-native | q41 | 0.000790 | 0.000828 | 0.000026 | 0.002572 | 0.002638 | 14.26 |
| 10k | rudb-native | q42 | 0.000817 | 0.000804 | 0.000039 | 0.002509 | 0.002564 | 14.53 |
| 10k | rudb-native | q43 | 0.000788 | 0.000801 | 0.000061 | 0.002566 | 0.002632 | 14.84 |
| 10k | duckdb-parquet | q1 | 0.001000 | 0.001000 | 0.000000 | 0.010854 | 0.012521 | 26.71 |
| 10k | duckdb-parquet | q2 | 0.001000 | 0.001000 | 0.001000 | 0.012572 | 0.014114 | 27.50 |
| 10k | duckdb-parquet | q3 | 0.001000 | 0.002000 | 0.001000 | 0.016440 | 0.018025 | 27.52 |
| 10k | duckdb-parquet | q4 | 0.002000 | 0.002000 | 0.001000 | 0.015128 | 0.017034 | 27.27 |
| 10k | duckdb-parquet | q5 | 0.003000 | 0.002000 | 0.001000 | 0.013520 | 0.016553 | 29.48 |
| 10k | duckdb-parquet | q6 | 0.002000 | 0.002000 | 0.000000 | 0.012136 | 0.014926 | 28.94 |
| 10k | duckdb-parquet | q7 | 0.001000 | 0.001000 | 0.000000 | 0.010668 | 0.012202 | 27.46 |
| 10k | duckdb-parquet | q8 | 0.002000 | 0.001000 | 0.001000 | 0.010720 | 0.012665 | 29.04 |
| 10k | duckdb-parquet | q9 | 0.002000 | 0.003000 | 0.001000 | 0.011682 | 0.016052 | 33.50 |
| 10k | duckdb-parquet | q10 | 0.003000 | 0.003000 | 0.001000 | 0.012084 | 0.016948 | 34.10 |
| 10k | duckdb-parquet | q11 | 0.002000 | 0.002000 | 0.000000 | 0.011307 | 0.014861 | 31.93 |
| 10k | duckdb-parquet | q12 | 0.002000 | 0.003000 | 0.001000 | 0.011487 | 0.015704 | 32.93 |
| 10k | duckdb-parquet | q13 | 0.002000 | 0.002000 | 0.000000 | 0.011052 | 0.013968 | 30.03 |
| 10k | duckdb-parquet | q14 | 0.003000 | 0.003000 | 0.001000 | 0.011850 | 0.016486 | 33.24 |
| 10k | duckdb-parquet | q15 | 0.002000 | 0.002000 | 0.000000 | 0.011159 | 0.014194 | 30.48 |
| 10k | duckdb-parquet | q16 | 0.002000 | 0.002000 | 0.000000 | 0.011284 | 0.014609 | 32.09 |
| 10k | duckdb-parquet | q17 | 0.002000 | 0.003000 | 0.001000 | 0.011758 | 0.015612 | 33.55 |
| 10k | duckdb-parquet | q18 | 0.003000 | 0.003000 | 0.001000 | 0.011461 | 0.014394 | 32.55 |
| 10k | duckdb-parquet | q19 | 0.002000 | 0.003000 | 0.000000 | 0.012143 | 0.016025 | 34.06 |
| 10k | duckdb-parquet | q20 | 0.001000 | 0.001000 | 0.000000 | 0.009932 | 0.011250 | 27.25 |
| 10k | duckdb-parquet | q21 | 0.003000 | 0.002000 | 0.001000 | 0.011267 | 0.012628 | 28.17 |
| 10k | duckdb-parquet | q22 | 0.003000 | 0.003000 | 0.001000 | 0.012065 | 0.014961 | 28.99 |
| 10k | duckdb-parquet | q23 | 0.004000 | 0.004000 | 0.000000 | 0.013959 | 0.019156 | 32.98 |
| 10k | duckdb-parquet | q24 | 0.010000 | 0.010000 | 0.007000 | 0.020106 | 0.030361 | 37.73 |
| 10k | duckdb-parquet | q25 | 0.006000 | 0.008000 | 0.007000 | 0.041324 | 0.040498 | 29.99 |
| 10k | duckdb-parquet | q26 | 0.003000 | 0.003000 | 0.006000 | 0.024493 | 0.024301 | 27.70 |
| 10k | duckdb-parquet | q27 | 0.002000 | 0.002000 | 0.000000 | 0.010821 | 0.012844 | 27.93 |
| 10k | duckdb-parquet | q28 | 0.003000 | 0.003000 | 0.000000 | 0.012236 | 0.016012 | 30.68 |
| 10k | duckdb-parquet | q29 | 0.007000 | 0.007000 | 0.001000 | 0.016619 | 0.026020 | 31.98 |
| 10k | duckdb-parquet | q30 | 0.004000 | 0.004000 | 0.000000 | 0.013589 | 0.015949 | 30.23 |
| 10k | duckdb-parquet | q31 | 0.002000 | 0.002000 | 0.001000 | 0.011580 | 0.014988 | 32.34 |
| 10k | duckdb-parquet | q32 | 0.002000 | 0.002000 | 0.000000 | 0.011477 | 0.014445 | 32.30 |
| 10k | duckdb-parquet | q33 | 0.002000 | 0.003000 | 0.001000 | 0.011646 | 0.015342 | 33.29 |
| 10k | duckdb-parquet | q34 | 0.004000 | 0.004000 | 0.001000 | 0.012829 | 0.017897 | 33.50 |
| 10k | duckdb-parquet | q35 | 0.004000 | 0.004000 | 0.001000 | 0.012931 | 0.018139 | 33.50 |
| 10k | duckdb-parquet | q36 | 0.002000 | 0.002000 | 0.001000 | 0.011337 | 0.014728 | 33.09 |
| 10k | duckdb-parquet | q37 | 0.003000 | 0.003000 | 0.001000 | 0.011815 | 0.015265 | 30.75 |
| 10k | duckdb-parquet | q38 | 0.003000 | 0.003000 | 0.001000 | 0.011969 | 0.015372 | 31.00 |
| 10k | duckdb-parquet | q39 | 0.003000 | 0.003000 | 0.001000 | 0.011871 | 0.015378 | 30.47 |
| 10k | duckdb-parquet | q40 | 0.004000 | 0.003000 | 0.001000 | 0.012823 | 0.017356 | 32.50 |
| 10k | duckdb-parquet | q41 | 0.002000 | 0.002000 | 0.000000 | 0.011281 | 0.013860 | 31.59 |
| 10k | duckdb-parquet | q42 | 0.002000 | 0.002000 | 0.000000 | 0.011250 | 0.013985 | 30.25 |
| 10k | duckdb-parquet | q43 | 0.002000 | 0.002000 | 0.000000 | 0.011131 | 0.014019 | 29.42 |
| 10k | rudb-parquet | q1 | 0.000556 | 0.000566 | 0.000034 | 0.001922 | 0.001897 | 11.37 |
| 10k | rudb-parquet | q2 | 0.000788 | 0.000816 | 0.000264 | 0.002519 | 0.002453 | 12.14 |
| 10k | rudb-parquet | q3 | 0.001249 | 0.000824 | 0.000509 | 0.002698 | 0.002668 | 11.80 |
| 10k | rudb-parquet | q4 | 0.001056 | 0.001076 | 0.000101 | 0.003307 | 0.003273 | 11.86 |
| 10k | rudb-parquet | q5 | 0.001158 | 0.000839 | 0.000085 | 0.002343 | 0.002307 | 11.80 |
| 10k | rudb-parquet | q6 | 0.002119 | 0.001437 | 0.000389 | 0.002767 | 0.002741 | 12.48 |
| 10k | rudb-parquet | q7 | 0.000657 | 0.000724 | 0.000080 | 0.001985 | 0.001960 | 12.42 |
| 10k | rudb-parquet | q8 | 0.000714 | 0.000743 | 0.000066 | 0.002032 | 0.001973 | 12.36 |
| 10k | rudb-parquet | q9 | 0.001035 | 0.001068 | 0.000047 | 0.002314 | 0.002289 | 12.34 |
| 10k | rudb-parquet | q10 | 0.001619 | 0.001554 | 0.000045 | 0.002835 | 0.002808 | 12.89 |
| 10k | rudb-parquet | q11 | 0.000801 | 0.000841 | 0.000054 | 0.002084 | 0.002063 | 12.33 |
| 10k | rudb-parquet | q12 | 0.000932 | 0.000906 | 0.000041 | 0.002168 | 0.002147 | 12.34 |
| 10k | rudb-parquet | q13 | 0.001317 | 0.001234 | 0.000035 | 0.002472 | 0.002450 | 12.37 |
| 10k | rudb-parquet | q14 | 0.001429 | 0.001401 | 0.000059 | 0.002674 | 0.002650 | 12.62 |
| 10k | rudb-parquet | q15 | 0.001296 | 0.001341 | 0.000054 | 0.002596 | 0.002574 | 12.29 |
| 10k | rudb-parquet | q16 | 0.000969 | 0.000943 | 0.000082 | 0.002243 | 0.002288 | 11.83 |
| 10k | rudb-parquet | q17 | 0.002051 | 0.001985 | 0.000065 | 0.003302 | 0.003278 | 13.32 |
| 10k | rudb-parquet | q18 | 0.001367 | 0.001376 | 0.000069 | 0.002634 | 0.002602 | 12.12 |
| 10k | rudb-parquet | q19 | 0.002381 | 0.002480 | 0.000082 | 0.003769 | 0.003735 | 14.43 |
| 10k | rudb-parquet | q20 | 0.000534 | 0.000529 | 0.000037 | 0.001758 | 0.001738 | 10.81 |
| 10k | rudb-parquet | q21 | 0.003707 | 0.003735 | 0.000060 | 0.005009 | 0.004986 | 14.12 |
| 10k | rudb-parquet | q22 | 0.004216 | 0.004200 | 0.000445 | 0.005601 | 0.005571 | 14.36 |
| 10k | rudb-parquet | q23 | 0.007503 | 0.007523 | 0.000203 | 0.009004 | 0.008978 | 16.12 |
| 10k | rudb-parquet | q24 | 0.004268 | 0.004260 | 0.003463 | 0.005661 | 0.005633 | 13.78 |
| 10k | rudb-parquet | q25 | 0.001925 | 0.002060 | 0.000489 | 0.004981 | 0.004174 | 12.24 |
| 10k | rudb-parquet | q26 | 0.001889 | 0.001683 | 0.000473 | 0.003774 | 0.003625 | 11.36 |
| 10k | rudb-parquet | q27 | 0.001312 | 0.001324 | 0.000016 | 0.002544 | 0.002523 | 12.21 |
| 10k | rudb-parquet | q28 | 0.003745 | 0.003772 | 0.000089 | 0.005055 | 0.005033 | 14.38 |
| 10k | rudb-parquet | q29 | 0.004299 | 0.004345 | 0.000076 | 0.005668 | 0.005644 | 14.62 |
| 10k | rudb-parquet | q30 | 0.002287 | 0.002254 | 0.000088 | 0.003558 | 0.003536 | 12.84 |
| 10k | rudb-parquet | q31 | 0.001677 | 0.001617 | 0.000065 | 0.002868 | 0.002846 | 12.68 |
| 10k | rudb-parquet | q32 | 0.001680 | 0.001662 | 0.000089 | 0.002921 | 0.002898 | 12.63 |
| 10k | rudb-parquet | q33 | 0.001420 | 0.001433 | 0.000059 | 0.002738 | 0.002887 | 12.84 |
| 10k | rudb-parquet | q34 | 0.004541 | 0.004494 | 0.000094 | 0.005837 | 0.005806 | 14.93 |
| 10k | rudb-parquet | q35 | 0.004522 | 0.004524 | 0.000049 | 0.005852 | 0.005828 | 14.91 |
| 10k | rudb-parquet | q36 | 0.001002 | 0.001011 | 0.000052 | 0.002286 | 0.002327 | 12.37 |
| 10k | rudb-parquet | q37 | 0.003821 | 0.003762 | 0.000050 | 0.005061 | 0.005039 | 14.54 |
| 10k | rudb-parquet | q38 | 0.004175 | 0.004109 | 0.000142 | 0.005432 | 0.005409 | 14.55 |
| 10k | rudb-parquet | q39 | 0.003784 | 0.003780 | 0.000070 | 0.005104 | 0.005079 | 14.47 |
| 10k | rudb-parquet | q40 | 0.006198 | 0.006275 | 0.000168 | 0.007595 | 0.007557 | 15.23 |
| 10k | rudb-parquet | q41 | 0.001239 | 0.001250 | 0.000013 | 0.002525 | 0.002503 | 13.49 |
| 10k | rudb-parquet | q42 | 0.001394 | 0.001339 | 0.000082 | 0.002597 | 0.002574 | 13.49 |
| 10k | rudb-parquet | q43 | 0.001201 | 0.001257 | 0.000041 | 0.002529 | 0.002508 | 13.45 |
| 1m | duckdb-native | q1 | 0.000000 | 0.000000 | 0.001000 | 0.009671 | 0.011119 | 26.66 |
| 1m | duckdb-native | q2 | 0.001000 | 0.001000 | 0.000000 | 0.010287 | 0.013133 | 29.96 |
| 1m | duckdb-native | q3 | 0.001000 | 0.001000 | 0.001000 | 0.010835 | 0.015143 | 32.66 |
| 1m | duckdb-native | q4 | 0.002000 | 0.002000 | 0.000000 | 0.011466 | 0.017352 | 36.22 |
| 1m | duckdb-native | q5 | 0.011000 | 0.011000 | 0.000000 | 0.021680 | 0.063752 | 66.42 |
| 1m | duckdb-native | q6 | 0.009000 | 0.009000 | 0.000000 | 0.019333 | 0.050110 | 58.35 |
| 1m | duckdb-native | q7 | 0.000000 | 0.001000 | 0.001000 | 0.009884 | 0.010989 | 26.94 |
| 1m | duckdb-native | q8 | 0.002000 | 0.001000 | 0.001000 | 0.010599 | 0.013634 | 31.51 |
| 1m | duckdb-native | q9 | 0.015000 | 0.015000 | 0.000000 | 0.025556 | 0.081735 | 75.91 |
| 1m | duckdb-native | q10 | 0.019000 | 0.020000 | 0.000000 | 0.030464 | 0.103151 | 84.20 |
| 1m | duckdb-native | q11 | 0.005000 | 0.005000 | 0.000000 | 0.015293 | 0.032144 | 52.54 |
| 1m | duckdb-native | q12 | 0.006000 | 0.006000 | 0.001000 | 0.015728 | 0.034238 | 54.98 |
| 1m | duckdb-native | q13 | 0.007000 | 0.007000 | 0.000000 | 0.017310 | 0.042093 | 61.29 |
| 1m | duckdb-native | q14 | 0.011000 | 0.012000 | 0.000000 | 0.022536 | 0.063372 | 83.99 |
| 1m | duckdb-native | q15 | 0.008000 | 0.008000 | 0.001000 | 0.018378 | 0.044941 | 65.13 |
| 1m | duckdb-native | q16 | 0.013000 | 0.013000 | 0.001000 | 0.023273 | 0.069869 | 77.79 |
| 1m | duckdb-native | q17 | 0.023000 | 0.023000 | 0.001000 | 0.034961 | 0.125732 | 129.82 |
| 1m | duckdb-native | q18 | 0.019000 | 0.019000 | 0.001000 | 0.030807 | 0.098470 | 124.51 |
| 1m | duckdb-native | q19 | 0.028000 | 0.027000 | 0.000000 | 0.038673 | 0.142371 | 143.04 |
| 1m | duckdb-native | q20 | 0.002000 | 0.001000 | 0.001000 | 0.011090 | 0.015910 | 35.72 |
| 1m | duckdb-native | q21 | 0.018000 | 0.019000 | 0.001000 | 0.029417 | 0.089757 | 92.09 |
| 1m | duckdb-native | q22 | 0.019000 | 0.019000 | 0.000000 | 0.030178 | 0.088919 | 103.74 |
| 1m | duckdb-native | q23 | 0.017000 | 0.017000 | 0.000000 | 0.028365 | 0.090164 | 123.26 |
| 1m | duckdb-native | q24 | 0.051000 | 0.048000 | 0.001000 | 0.060814 | 0.183556 | 212.23 |
| 1m | duckdb-native | q25 | 0.004000 | 0.004000 | 0.000000 | 0.013649 | 0.026357 | 42.10 |
| 1m | duckdb-native | q26 | 0.004000 | 0.004000 | 0.000000 | 0.013703 | 0.027476 | 37.11 |
| 1m | duckdb-native | q27 | 0.004000 | 0.004000 | 0.001000 | 0.013266 | 0.026009 | 40.12 |
| 1m | duckdb-native | q28 | 0.018000 | 0.018000 | 0.000000 | 0.028916 | 0.086048 | 99.34 |
| 1m | duckdb-native | q29 | 0.118000 | 0.120000 | 0.006000 | 0.135083 | 0.575776 | 137.43 |
| 1m | duckdb-native | q30 | 0.004000 | 0.004000 | 0.000000 | 0.013729 | 0.018138 | 33.78 |
| 1m | duckdb-native | q31 | 0.008000 | 0.008000 | 0.000000 | 0.018966 | 0.047186 | 70.71 |
| 1m | duckdb-native | q32 | 0.010000 | 0.009000 | 0.001000 | 0.019662 | 0.051754 | 77.83 |
| 1m | duckdb-native | q33 | 0.025000 | 0.025000 | 0.002000 | 0.036974 | 0.125496 | 133.79 |
| 1m | duckdb-native | q34 | 0.040000 | 0.040000 | 0.001000 | 0.051958 | 0.189839 | 217.18 |
| 1m | duckdb-native | q35 | 0.041000 | 0.040000 | 0.002000 | 0.052332 | 0.194843 | 219.20 |
| 1m | duckdb-native | q36 | 0.015000 | 0.014000 | 0.000000 | 0.025055 | 0.077270 | 79.00 |
| 1m | duckdb-native | q37 | 0.003000 | 0.003000 | 0.000000 | 0.012791 | 0.018802 | 35.98 |
| 1m | duckdb-native | q38 | 0.002000 | 0.003000 | 0.001000 | 0.012737 | 0.017967 | 34.19 |
| 1m | duckdb-native | q39 | 0.003000 | 0.002000 | 0.001000 | 0.012754 | 0.017868 | 34.57 |
| 1m | duckdb-native | q40 | 0.005000 | 0.005000 | 0.001000 | 0.016445 | 0.024231 | 42.42 |
| 1m | duckdb-native | q41 | 0.003000 | 0.003000 | 0.001000 | 0.014062 | 0.018392 | 35.04 |
| 1m | duckdb-native | q42 | 0.002000 | 0.002000 | 0.002000 | 0.013108 | 0.019308 | 34.22 |
| 1m | duckdb-native | q43 | 0.002000 | 0.002000 | 0.001000 | 0.012565 | 0.016875 | 32.79 |
| 1m | rudb-native | q1 | 0.000335 | 0.000306 | 0.000040 | 0.001918 | 0.001895 | 11.00 |
| 1m | rudb-native | q2 | 0.000347 | 0.000342 | 0.000023 | 0.001938 | 0.001915 | 10.96 |
| 1m | rudb-native | q3 | 0.000359 | 0.000366 | 0.000037 | 0.002003 | 0.001981 | 11.22 |
| 1m | rudb-native | q4 | 0.000363 | 0.000358 | 0.000024 | 0.002058 | 0.002031 | 10.99 |
| 1m | rudb-native | q5 | 0.000309 | 0.000359 | 0.000010 | 0.002107 | 0.002082 | 11.01 |
| 1m | rudb-native | q6 | 0.000376 | 0.000368 | 0.000012 | 0.002095 | 0.002073 | 10.91 |
| 1m | rudb-native | q7 | 0.000324 | 0.000324 | 0.000039 | 0.001945 | 0.001907 | 10.96 |
| 1m | rudb-native | q8 | 0.000943 | 0.000883 | 0.000036 | 0.002577 | 0.003855 | 25.89 |
| 1m | rudb-native | q9 | 0.005459 | 0.005562 | 0.000299 | 0.007457 | 0.025562 | 50.98 |
| 1m | rudb-native | q10 | 0.007095 | 0.007163 | 0.000207 | 0.008929 | 0.031936 | 68.70 |
| 1m | rudb-native | q11 | 0.001854 | 0.001917 | 0.000045 | 0.003717 | 0.007861 | 44.84 |
| 1m | rudb-native | q12 | 0.002198 | 0.002135 | 0.000097 | 0.003887 | 0.008730 | 45.70 |
| 1m | rudb-native | q13 | 0.001820 | 0.001906 | 0.000033 | 0.003672 | 0.006129 | 26.60 |
| 1m | rudb-native | q14 | 0.003495 | 0.003599 | 0.000101 | 0.005456 | 0.015506 | 48.21 |
| 1m | rudb-native | q15 | 0.003407 | 0.003395 | 0.000208 | 0.005190 | 0.014829 | 28.52 |
| 1m | rudb-native | q16 | 0.000525 | 0.000504 | 0.000005 | 0.002269 | 0.002244 | 12.05 |
| 1m | rudb-native | q17 | 0.006875 | 0.006814 | 0.000091 | 0.008714 | 0.031278 | 60.13 |
| 1m | rudb-native | q18 | 0.001300 | 0.001254 | 0.000057 | 0.003069 | 0.003034 | 42.70 |
| 1m | rudb-native | q19 | 0.008279 | 0.008306 | 0.000274 | 0.010163 | 0.037513 | 64.23 |
| 1m | rudb-native | q20 | 0.000843 | 0.000673 | 0.000030 | 0.002411 | 0.003302 | 19.25 |
| 1m | rudb-native | q21 | 0.018249 | 0.019311 | 0.000273 | 0.021174 | 0.050302 | 53.97 |
| 1m | rudb-native | q22 | 0.020714 | 0.022124 | 0.000750 | 0.024032 | 0.056123 | 73.84 |
| 1m | rudb-native | q23 | 0.025935 | 0.026201 | 0.001570 | 0.028158 | 0.056291 | 79.94 |
| 1m | rudb-native | q24 | 0.023660 | 0.021611 | 0.001353 | 0.023712 | 0.056841 | 115.88 |
| 1m | rudb-native | q25 | 0.001832 | 0.001718 | 0.000060 | 0.003507 | 0.005417 | 37.71 |
| 1m | rudb-native | q26 | 0.002928 | 0.002767 | 0.000218 | 0.004494 | 0.009087 | 24.63 |
| 1m | rudb-native | q27 | 0.002080 | 0.002139 | 0.000076 | 0.003963 | 0.006014 | 38.35 |
| 1m | rudb-native | q28 | 0.004500 | 0.004501 | 0.000260 | 0.006359 | 0.012756 | 50.71 |
| 1m | rudb-native | q29 | 0.035074 | 0.034748 | 0.000922 | 0.036720 | 0.137300 | 73.33 |
| 1m | rudb-native | q30 | 0.001859 | 0.001860 | 0.000048 | 0.003609 | 0.003586 | 12.80 |
| 1m | rudb-native | q31 | 0.003107 | 0.003061 | 0.000163 | 0.004887 | 0.012639 | 53.11 |
| 1m | rudb-native | q32 | 0.000819 | 0.000824 | 0.000038 | 0.002627 | 0.002602 | 15.81 |
| 1m | rudb-native | q33 | 0.000514 | 0.000527 | 0.000033 | 0.002292 | 0.002267 | 15.22 |
| 1m | rudb-native | q34 | 0.000582 | 0.000578 | 0.000035 | 0.002290 | 0.002268 | 12.04 |
| 1m | rudb-native | q35 | 0.003244 | 0.003139 | 0.000044 | 0.004942 | 0.009417 | 29.16 |
| 1m | rudb-native | q36 | 0.000608 | 0.000573 | 0.000017 | 0.002378 | 0.002353 | 12.55 |
| 1m | rudb-native | q37 | 0.002100 | 0.002072 | 0.000080 | 0.003891 | 0.004725 | 24.91 |
| 1m | rudb-native | q38 | 0.002128 | 0.002110 | 0.000131 | 0.003995 | 0.004646 | 23.01 |
| 1m | rudb-native | q39 | 0.001507 | 0.001560 | 0.000100 | 0.003389 | 0.004173 | 27.60 |
| 1m | rudb-native | q40 | 0.003688 | 0.003733 | 0.000220 | 0.005649 | 0.006995 | 29.75 |
| 1m | rudb-native | q41 | 0.001313 | 0.001291 | 0.000267 | 0.003125 | 0.003975 | 28.21 |
| 1m | rudb-native | q42 | 0.001294 | 0.001288 | 0.000103 | 0.003082 | 0.003836 | 28.07 |
| 1m | rudb-native | q43 | 0.001447 | 0.001467 | 0.000066 | 0.003243 | 0.004196 | 22.10 |
| 1m | duckdb-parquet | q1 | 0.002000 | 0.002000 | 0.001000 | 0.011347 | 0.012914 | 27.23 |
| 1m | duckdb-parquet | q2 | 0.002000 | 0.002000 | 0.000000 | 0.011891 | 0.015277 | 28.67 |
| 1m | duckdb-parquet | q3 | 0.003000 | 0.003000 | 0.000000 | 0.013139 | 0.019516 | 30.44 |
| 1m | duckdb-parquet | q4 | 0.003000 | 0.003000 | 0.000000 | 0.013191 | 0.019994 | 38.98 |
| 1m | duckdb-parquet | q5 | 0.012000 | 0.012000 | 0.001000 | 0.023273 | 0.062164 | 62.19 |
| 1m | duckdb-parquet | q6 | 0.009000 | 0.010000 | 0.001000 | 0.020011 | 0.048249 | 55.58 |
| 1m | duckdb-parquet | q7 | 0.002000 | 0.002000 | 0.000000 | 0.011973 | 0.015722 | 28.25 |
| 1m | duckdb-parquet | q8 | 0.002000 | 0.002000 | 0.001000 | 0.012121 | 0.015635 | 30.31 |
| 1m | duckdb-parquet | q9 | 0.017000 | 0.016000 | 0.000000 | 0.027361 | 0.084217 | 68.97 |
| 1m | duckdb-parquet | q10 | 0.019000 | 0.018000 | 0.001000 | 0.029383 | 0.093846 | 74.03 |
| 1m | duckdb-parquet | q11 | 0.005000 | 0.006000 | 0.000000 | 0.016265 | 0.031716 | 49.98 |
| 1m | duckdb-parquet | q12 | 0.006000 | 0.006000 | 0.001000 | 0.016952 | 0.034543 | 50.25 |
| 1m | duckdb-parquet | q13 | 0.010000 | 0.010000 | 0.000000 | 0.020455 | 0.053016 | 60.49 |
| 1m | duckdb-parquet | q14 | 0.014000 | 0.014000 | 0.001000 | 0.024773 | 0.070647 | 72.88 |
| 1m | duckdb-parquet | q15 | 0.011000 | 0.011000 | 0.000000 | 0.021138 | 0.056137 | 63.00 |
| 1m | duckdb-parquet | q16 | 0.013000 | 0.014000 | 0.001000 | 0.024509 | 0.070759 | 73.60 |
| 1m | duckdb-parquet | q17 | 0.025000 | 0.026000 | 0.001000 | 0.037110 | 0.133716 | 122.35 |
| 1m | duckdb-parquet | q18 | 0.023000 | 0.022000 | 0.002000 | 0.033714 | 0.108291 | 121.31 |
| 1m | duckdb-parquet | q19 | 0.030000 | 0.031000 | 0.001000 | 0.042715 | 0.151071 | 133.80 |
| 1m | duckdb-parquet | q20 | 0.003000 | 0.003000 | 0.000000 | 0.013073 | 0.019319 | 39.82 |
| 1m | duckdb-parquet | q21 | 0.027000 | 0.028000 | 0.002000 | 0.039147 | 0.133009 | 123.44 |
| 1m | duckdb-parquet | q22 | 0.029000 | 0.030000 | 0.004000 | 0.042388 | 0.136192 | 149.25 |
| 1m | duckdb-parquet | q23 | 0.056000 | 0.059000 | 0.005000 | 0.072273 | 0.262591 | 253.28 |
| 1m | duckdb-parquet | q24 | 0.104000 | 0.092000 | 0.016000 | 0.107434 | 0.411382 | 359.57 |
| 1m | duckdb-parquet | q25 | 0.012000 | 0.013000 | 0.000000 | 0.023090 | 0.063134 | 56.73 |
| 1m | duckdb-parquet | q26 | 0.006000 | 0.007000 | 0.000000 | 0.017008 | 0.037296 | 40.21 |
| 1m | duckdb-parquet | q27 | 0.009000 | 0.009000 | 0.000000 | 0.019632 | 0.046869 | 49.96 |
| 1m | duckdb-parquet | q28 | 0.027000 | 0.028000 | 0.001000 | 0.039640 | 0.123915 | 141.75 |
| 1m | duckdb-parquet | q29 | 0.130000 | 0.130000 | 0.010000 | 0.143883 | 0.634257 | 164.73 |
| 1m | duckdb-parquet | q30 | 0.006000 | 0.006000 | 0.001000 | 0.015715 | 0.022138 | 32.82 |
| 1m | duckdb-parquet | q31 | 0.011000 | 0.011000 | 0.001000 | 0.021477 | 0.056238 | 60.41 |
| 1m | duckdb-parquet | q32 | 0.011000 | 0.012000 | 0.001000 | 0.022360 | 0.059286 | 68.70 |
| 1m | duckdb-parquet | q33 | 0.024000 | 0.026000 | 0.001000 | 0.037495 | 0.128840 | 123.10 |
| 1m | duckdb-parquet | q34 | 0.050000 | 0.049000 | 0.002000 | 0.061501 | 0.224930 | 234.12 |
| 1m | duckdb-parquet | q35 | 0.049000 | 0.050000 | 0.002000 | 0.063175 | 0.235251 | 237.99 |
| 1m | duckdb-parquet | q36 | 0.016000 | 0.015000 | 0.001000 | 0.026356 | 0.078378 | 77.34 |
| 1m | duckdb-parquet | q37 | 0.017000 | 0.016000 | 0.001000 | 0.027577 | 0.053699 | 64.60 |
| 1m | duckdb-parquet | q38 | 0.014000 | 0.013000 | 0.001000 | 0.024152 | 0.046521 | 60.37 |
| 1m | duckdb-parquet | q39 | 0.016000 | 0.017000 | 0.001000 | 0.028015 | 0.053911 | 63.30 |
| 1m | duckdb-parquet | q40 | 0.026000 | 0.027000 | 0.001000 | 0.039316 | 0.087633 | 87.23 |
| 1m | duckdb-parquet | q41 | 0.009000 | 0.005000 | 0.001000 | 0.016211 | 0.024133 | 38.54 |
| 1m | duckdb-parquet | q42 | 0.005000 | 0.004000 | 0.000000 | 0.015045 | 0.020972 | 35.50 |
| 1m | duckdb-parquet | q43 | 0.004000 | 0.004000 | 0.001000 | 0.015113 | 0.021464 | 33.99 |
| 1m | rudb-parquet | q1 | 0.001602 | 0.001394 | 0.000042 | 0.002573 | 0.003052 | 12.11 |
| 1m | rudb-parquet | q2 | 0.002214 | 0.002121 | 0.000169 | 0.003327 | 0.006130 | 16.12 |
| 1m | rudb-parquet | q3 | 0.003752 | 0.003703 | 0.000062 | 0.004963 | 0.013091 | 19.91 |
| 1m | rudb-parquet | q4 | 0.003091 | 0.002970 | 0.000168 | 0.004299 | 0.010906 | 31.04 |
| 1m | rudb-parquet | q5 | 0.006379 | 0.006366 | 0.000120 | 0.007691 | 0.028838 | 32.57 |
| 1m | rudb-parquet | q6 | 0.017965 | 0.018535 | 0.000548 | 0.019915 | 0.071896 | 45.51 |
| 1m | rudb-parquet | q7 | 0.001861 | 0.001894 | 0.000047 | 0.003126 | 0.005659 | 16.13 |
| 1m | rudb-parquet | q8 | 0.002231 | 0.002183 | 0.000141 | 0.003449 | 0.006292 | 16.36 |
| 1m | rudb-parquet | q9 | 0.009169 | 0.009039 | 0.000340 | 0.010446 | 0.037374 | 45.10 |
| 1m | rudb-parquet | q10 | 0.012726 | 0.012734 | 0.000069 | 0.014124 | 0.052873 | 53.86 |
| 1m | rudb-parquet | q11 | 0.006032 | 0.005982 | 0.000125 | 0.007328 | 0.023024 | 32.57 |
| 1m | rudb-parquet | q12 | 0.006665 | 0.006818 | 0.000106 | 0.008232 | 0.026327 | 34.61 |
| 1m | rudb-parquet | q13 | 0.015432 | 0.015497 | 0.000356 | 0.016896 | 0.065724 | 46.04 |
| 1m | rudb-parquet | q14 | 0.021985 | 0.022080 | 0.000868 | 0.023501 | 0.088173 | 70.07 |
| 1m | rudb-parquet | q15 | 0.016671 | 0.016834 | 0.000274 | 0.018240 | 0.071962 | 49.07 |
| 1m | rudb-parquet | q16 | 0.005461 | 0.005344 | 0.000102 | 0.006762 | 0.021931 | 38.08 |
| 1m | rudb-parquet | q17 | 0.043345 | 0.044823 | 0.003281 | 0.046236 | 0.157991 | 105.34 |
| 1m | rudb-parquet | q18 | 0.012744 | 0.012680 | 0.000234 | 0.014142 | 0.055703 | 38.50 |
| 1m | rudb-parquet | q19 | 0.060508 | 0.060146 | 0.003328 | 0.062175 | 0.209524 | 131.12 |
| 1m | rudb-parquet | q20 | 0.002794 | 0.002826 | 0.000084 | 0.004158 | 0.010459 | 29.04 |
| 1m | rudb-parquet | q21 | 0.067674 | 0.063773 | 0.004266 | 0.068540 | 0.263164 | 120.16 |
| 1m | rudb-parquet | q22 | 0.072872 | 0.074228 | 0.003738 | 0.077472 | 0.303607 | 108.55 |
| 1m | rudb-parquet | q23 | 0.136492 | 0.139179 | 0.005519 | 0.143890 | 0.580672 | 200.12 |
| 1m | rudb-parquet | q24 | 0.135304 | 0.136716 | 0.003841 | 0.139920 | 1.220990 | 311.36 |
| 1m | rudb-parquet | q25 | 0.016451 | 0.016198 | 0.000387 | 0.017638 | 0.068583 | 39.43 |
| 1m | rudb-parquet | q26 | 0.010634 | 0.010656 | 0.000101 | 0.011996 | 0.045152 | 26.62 |
| 1m | rudb-parquet | q27 | 0.016490 | 0.016266 | 0.000201 | 0.017667 | 0.069194 | 39.40 |
| 1m | rudb-parquet | q28 | 0.062107 | 0.062137 | 0.003792 | 0.066728 | 0.258771 | 119.66 |
| 1m | rudb-parquet | q29 | 0.093846 | 0.094422 | 0.000388 | 0.100476 | 0.378589 | 148.05 |
| 1m | rudb-parquet | q30 | 0.004955 | 0.004989 | 0.000088 | 0.006277 | 0.013200 | 17.85 |
| 1m | rudb-parquet | q31 | 0.015147 | 0.015234 | 0.000116 | 0.016612 | 0.064853 | 46.86 |
| 1m | rudb-parquet | q32 | 0.015609 | 0.015462 | 0.000346 | 0.016901 | 0.066932 | 54.84 |
| 1m | rudb-parquet | q33 | 0.010513 | 0.010454 | 0.000104 | 0.011949 | 0.045836 | 56.09 |
| 1m | rudb-parquet | q34 | 0.095414 | 0.096984 | 0.003880 | 0.105250 | 0.384515 | 229.54 |
| 1m | rudb-parquet | q35 | 0.096831 | 0.097927 | 0.004886 | 0.105912 | 0.389510 | 230.16 |
| 1m | rudb-parquet | q36 | 0.005115 | 0.005062 | 0.000133 | 0.006453 | 0.020356 | 32.63 |
| 1m | rudb-parquet | q37 | 0.045262 | 0.047267 | 0.002185 | 0.049518 | 0.080040 | 55.71 |
| 1m | rudb-parquet | q38 | 0.043363 | 0.043934 | 0.001190 | 0.045974 | 0.078517 | 49.66 |
| 1m | rudb-parquet | q39 | 0.047444 | 0.045960 | 0.002464 | 0.048371 | 0.078698 | 55.66 |
| 1m | rudb-parquet | q40 | 0.079561 | 0.088420 | 0.012713 | 0.091348 | 0.161949 | 78.69 |
| 1m | rudb-parquet | q41 | 0.007806 | 0.007009 | 0.001031 | 0.008633 | 0.014217 | 28.23 |
| 1m | rudb-parquet | q42 | 0.007091 | 0.006713 | 0.000092 | 0.008090 | 0.013334 | 26.20 |
| 1m | rudb-parquet | q43 | 0.006622 | 0.006296 | 0.000183 | 0.007765 | 0.012672 | 22.52 |
| 10m | duckdb-native | q1 | 0.002000 | 0.001000 | 0.001000 | 0.011204 | 0.012905 | 27.20 |
| 10m | duckdb-native | q2 | 0.011000 | 0.005000 | 0.001000 | 0.014818 | 0.027483 | 50.45 |
| 10m | duckdb-native | q3 | 0.013000 | 0.008000 | 0.001000 | 0.018557 | 0.049287 | 72.53 |
| 10m | duckdb-native | q4 | 0.035000 | 0.012000 | 0.001000 | 0.023756 | 0.074073 | 98.41 |
| 10m | duckdb-native | q5 | 0.060000 | 0.063000 | 0.002000 | 0.076278 | 0.373054 | 219.14 |
| 10m | duckdb-native | q6 | 0.069000 | 0.060000 | 0.001000 | 0.073263 | 0.354909 | 226.77 |
| 10m | duckdb-native | q7 | 0.002000 | 0.002000 | 0.001000 | 0.012180 | 0.013407 | 29.00 |
| 10m | duckdb-native | q8 | 0.005000 | 0.005000 | 0.000000 | 0.015423 | 0.029410 | 52.43 |
| 10m | duckdb-native | q9 | 0.089000 | 0.083000 | 0.004000 | 0.097336 | 0.485744 | 268.50 |
| 10m | duckdb-native | q10 | 0.108000 | 0.106000 | 0.002000 | 0.121332 | 0.631672 | 314.52 |
| 10m | duckdb-native | q11 | 0.031000 | 0.026000 | 0.000000 | 0.038434 | 0.153959 | 160.04 |
| 10m | duckdb-native | q12 | 0.030000 | 0.029000 | 0.026000 | 0.042206 | 0.171265 | 168.78 |
| 10m | duckdb-native | q13 | 0.049000 | 0.064000 | 0.022000 | 0.080858 | 0.384518 | 230.94 |
| 10m | duckdb-native | q14 | 0.373000 | 0.124000 | 0.212000 | 0.146764 | 0.741587 | 387.61 |
| 10m | duckdb-native | q15 | 0.054000 | 0.053000 | 0.001000 | 0.066444 | 0.308203 | 250.24 |
| 10m | duckdb-native | q16 | 0.075000 | 0.071000 | 0.004000 | 0.086845 | 0.420314 | 271.77 |
| 10m | duckdb-native | q17 | 0.153000 | 0.148000 | 0.010000 | 0.167348 | 0.892432 | 665.74 |
| 10m | duckdb-native | q18 | 0.123000 | 0.121000 | 0.007000 | 0.140547 | 0.722022 | 665.22 |
| 10m | duckdb-native | q19 | 0.446000 | 0.240000 | 0.037000 | 0.271277 | 1.456849 | 944.14 |
| 10m | duckdb-native | q20 | 0.009000 | 0.009000 | 0.001000 | 0.021062 | 0.055448 | 96.79 |
| 10m | duckdb-native | q21 | 0.277000 | 0.135000 | 0.016000 | 0.157822 | 0.813984 | 615.92 |
| 10m | duckdb-native | q22 | 0.130000 | 0.123000 | 0.011000 | 0.143780 | 0.716104 | 676.64 |
| 10m | duckdb-native | q23 | 0.256000 | 0.385000 | 0.966000 | 0.482494 | 1.779227 | 925.87 |
| 10m | duckdb-native | q24 | 0.262000 | 0.166000 | 0.101000 | 0.197561 | 0.869554 | 552.14 |
| 10m | duckdb-native | q25 | 0.012000 | 0.012000 | 0.004000 | 0.025810 | 0.067819 | 64.93 |
| 10m | duckdb-native | q26 | 0.035000 | 0.028000 | 0.014000 | 0.041385 | 0.168329 | 88.47 |
| 10m | duckdb-native | q27 | 0.012000 | 0.012000 | 0.002000 | 0.023525 | 0.065315 | 63.49 |
| 10m | duckdb-native | q28 | 0.229000 | 0.158000 | 0.088000 | 0.188432 | 0.907730 | 636.66 |
| 10m | duckdb-native | q29 | 1.211000 | 1.142000 | 0.619000 | 1.182421 | 6.569368 | 780.52 |
| 10m | duckdb-native | q30 | 0.046000 | 0.009000 | 0.001000 | 0.020054 | 0.043122 | 54.80 |
| 10m | duckdb-native | q31 | 0.067000 | 0.053000 | 0.009000 | 0.067913 | 0.309665 | 270.47 |
| 10m | duckdb-native | q32 | 0.088000 | 0.269000 | 0.230000 | 0.399960 | 0.755909 | 371.64 |
| 10m | duckdb-native | q33 | 0.217000 | 0.191000 | 0.006000 | 0.216602 | 1.146927 | 866.88 |
| 10m | duckdb-native | q34 | 0.274000 | 0.269000 | 0.032000 | 0.310986 | 1.636658 | 1466.48 |
| 10m | duckdb-native | q35 | 0.283000 | 0.272000 | 0.008000 | 0.309577 | 1.660920 | 1497.43 |
| 10m | duckdb-native | q36 | 0.083000 | 0.086000 | 0.005000 | 0.100713 | 0.510847 | 300.74 |
| 10m | duckdb-native | q37 | 0.014000 | 0.010000 | 0.000000 | 0.021185 | 0.038752 | 54.17 |
| 10m | duckdb-native | q38 | 0.005000 | 0.005000 | 0.001000 | 0.015923 | 0.025474 | 41.71 |
| 10m | duckdb-native | q39 | 0.005000 | 0.006000 | 0.011000 | 0.017158 | 0.027547 | 42.18 |
| 10m | duckdb-native | q40 | 0.055000 | 0.018000 | 0.002000 | 0.029328 | 0.054586 | 76.42 |
| 10m | duckdb-native | q41 | 0.007000 | 0.004000 | 0.001000 | 0.014612 | 0.022653 | 40.89 |
| 10m | duckdb-native | q42 | 0.005000 | 0.004000 | 0.000000 | 0.014463 | 0.022276 | 39.46 |
| 10m | duckdb-native | q43 | 0.006000 | 0.004000 | 0.001000 | 0.014487 | 0.022345 | 36.29 |
| 10m | rudb-native | q1 | 0.006104 | 0.000348 | 0.000040 | 0.003199 | 0.003176 | 12.30 |
| 10m | rudb-native | q2 | 0.000412 | 0.000390 | 0.000022 | 0.003317 | 0.003292 | 12.61 |
| 10m | rudb-native | q3 | 0.000398 | 0.000402 | 0.000008 | 0.003338 | 0.003314 | 12.80 |
| 10m | rudb-native | q4 | 0.000369 | 0.000364 | 0.000034 | 0.003237 | 0.003214 | 12.55 |
| 10m | rudb-native | q5 | 0.000345 | 0.000346 | 0.000034 | 0.003205 | 0.003180 | 12.31 |
| 10m | rudb-native | q6 | 0.000387 | 0.000379 | 0.000013 | 0.003317 | 0.003293 | 12.57 |
| 10m | rudb-native | q7 | 0.000406 | 0.000372 | 0.000007 | 0.003292 | 0.003269 | 12.56 |
| 10m | rudb-native | q8 | 0.002010 | 0.001987 | 0.000098 | 0.005027 | 0.011421 | 31.54 |
| 10m | rudb-native | q9 | 0.034791 | 0.034149 | 0.000424 | 0.037253 | 0.195395 | 135.16 |
| 10m | rudb-native | q10 | 0.042973 | 0.043235 | 0.001427 | 0.046345 | 0.248188 | 161.00 |
| 10m | rudb-native | q11 | 0.008473 | 0.008365 | 0.000232 | 0.011437 | 0.047119 | 55.37 |
| 10m | rudb-native | q12 | 0.009740 | 0.009829 | 0.000174 | 0.012943 | 0.055122 | 56.10 |
| 10m | rudb-native | q13 | 0.010021 | 0.007334 | 0.001905 | 0.010405 | 0.032333 | 40.20 |
| 10m | rudb-native | q14 | 0.059007 | 0.025387 | 0.005957 | 0.029827 | 0.133673 | 80.41 |
| 10m | rudb-native | q15 | 0.014597 | 0.014945 | 0.000614 | 0.018218 | 0.079037 | 66.04 |
| 10m | rudb-native | q16 | 0.000534 | 0.000549 | 0.000085 | 0.003622 | 0.003591 | 13.34 |
| 10m | rudb-native | q17 | 0.058361 | 0.054928 | 0.004080 | 0.060787 | 0.295489 | 218.98 |
| 10m | rudb-native | q18 | 0.003160 | 0.003209 | 0.003493 | 0.006291 | 0.006266 | 50.93 |
| 10m | rudb-native | q19 | 0.225753 | 0.086322 | 0.010669 | 0.101542 | 0.491250 | 272.80 |
| 10m | rudb-native | q20 | 0.005675 | 0.001600 | 0.000160 | 0.004888 | 0.009424 | 20.86 |
| 10m | rudb-native | q21 | 0.141391 | 0.078317 | 0.005575 | 0.084838 | 0.355751 | 111.44 |
| 10m | rudb-native | q22 | 0.090666 | 0.086237 | 0.003297 | 0.092585 | 0.365462 | 111.01 |
| 10m | rudb-native | q23 | 0.239052 | 0.160720 | 0.063930 | 0.171976 | 0.431012 | 125.75 |
| 10m | rudb-native | q24 | 0.103470 | 0.074881 | 0.007802 | 0.082191 | 0.224818 | 128.96 |
| 10m | rudb-native | q25 | 0.004223 | 0.003740 | 0.000295 | 0.006970 | 0.011786 | 24.57 |
| 10m | rudb-native | q26 | 0.032519 | 0.013344 | 0.002922 | 0.017317 | 0.045460 | 30.30 |
| 10m | rudb-native | q27 | 0.009491 | 0.007461 | 0.000361 | 0.010846 | 0.015550 | 28.33 |
| 10m | rudb-native | q28 | 0.035382 | 0.027146 | 0.007859 | 0.031428 | 0.071102 | 56.23 |
| 10m | rudb-native | q29 | 0.669441 | 0.318263 | 0.130253 | 0.336233 | 1.476813 | 276.31 |
| 10m | rudb-native | q30 | 0.003802 | 0.001904 | 0.001115 | 0.005008 | 0.004978 | 13.84 |
| 10m | rudb-native | q31 | 0.025519 | 0.022908 | 0.008648 | 0.026512 | 0.127510 | 67.05 |
| 10m | rudb-native | q32 | 0.001803 | 0.002639 | 0.001560 | 0.009363 | 0.008802 | 16.14 |
| 10m | rudb-native | q33 | 0.000643 | 0.000587 | 0.000020 | 0.003616 | 0.003581 | 13.87 |
| 10m | rudb-native | q34 | 0.000575 | 0.000581 | 0.000019 | 0.003591 | 0.003557 | 13.55 |
| 10m | rudb-native | q35 | 0.017118 | 0.015131 | 0.000636 | 0.018498 | 0.057492 | 79.93 |
| 10m | rudb-native | q36 | 0.000593 | 0.000605 | 0.000030 | 0.003623 | 0.003597 | 14.06 |
| 10m | rudb-native | q37 | 0.010026 | 0.006387 | 0.000420 | 0.009749 | 0.012691 | 29.61 |
| 10m | rudb-native | q38 | 0.004440 | 0.004583 | 0.003167 | 0.007727 | 0.010419 | 24.81 |
| 10m | rudb-native | q39 | 0.006802 | 0.006708 | 0.016827 | 0.010164 | 0.011652 | 27.66 |
| 10m | rudb-native | q40 | 0.039970 | 0.018809 | 0.006134 | 0.022497 | 0.042113 | 45.65 |
| 10m | rudb-native | q41 | 0.005484 | 0.001851 | 0.000211 | 0.004922 | 0.007232 | 20.30 |
| 10m | rudb-native | q42 | 0.002393 | 0.002305 | 0.000429 | 0.005374 | 0.007638 | 20.84 |
| 10m | rudb-native | q43 | 0.002904 | 0.001807 | 0.000166 | 0.004791 | 0.008037 | 21.28 |
| 10m | duckdb-parquet | q1 | 0.132000 | 0.131000 | 0.004000 | 0.249082 | 0.247485 | 150.25 |
| 10m | duckdb-parquet | q2 | 0.147000 | 0.138000 | 0.007000 | 0.258121 | 0.281029 | 158.57 |
| 10m | duckdb-parquet | q3 | 0.150000 | 0.141000 | 0.008000 | 0.261567 | 0.330945 | 174.23 |
| 10m | duckdb-parquet | q4 | 0.208000 | 0.140000 | 0.005000 | 0.264081 | 0.305940 | 159.02 |
| 10m | duckdb-parquet | q5 | 0.174000 | 0.175000 | 0.003000 | 0.298115 | 0.581513 | 276.77 |
| 10m | duckdb-parquet | q6 | 0.220000 | 0.189000 | 0.007000 | 0.310960 | 0.636618 | 315.17 |
| 10m | duckdb-parquet | q7 | 0.147000 | 0.142000 | 0.009000 | 0.269802 | 0.309184 | 162.71 |
| 10m | duckdb-parquet | q8 | 0.137000 | 0.139000 | 0.006000 | 0.264062 | 0.293978 | 160.27 |
| 10m | duckdb-parquet | q9 | 0.231000 | 0.201000 | 0.005000 | 0.326580 | 0.719736 | 316.73 |
| 10m | duckdb-parquet | q10 | 0.223000 | 0.223000 | 0.001000 | 0.352050 | 0.834373 | 344.10 |
| 10m | duckdb-parquet | q11 | 0.152000 | 0.150000 | 0.002000 | 0.274353 | 0.388238 | 206.92 |
| 10m | duckdb-parquet | q12 | 0.164000 | 0.159000 | 0.001000 | 0.285917 | 0.435000 | 219.46 |
| 10m | duckdb-parquet | q13 | 0.191000 | 0.209000 | 0.218000 | 0.337985 | 0.734814 | 320.24 |
| 10m | duckdb-parquet | q14 | 0.679000 | 0.347000 | 0.116000 | 0.536497 | 1.315845 | 416.30 |
| 10m | duckdb-parquet | q15 | 0.192000 | 0.195000 | 0.002000 | 0.318618 | 0.677743 | 336.55 |
| 10m | duckdb-parquet | q16 | 0.243000 | 0.190000 | 0.036000 | 0.317778 | 0.650647 | 330.42 |
| 10m | duckdb-parquet | q17 | 0.295000 | 0.297000 | 0.104000 | 0.433162 | 1.266737 | 686.33 |
| 10m | duckdb-parquet | q18 | 0.264000 | 0.262000 | 0.118000 | 0.393933 | 1.057847 | 684.81 |
| 10m | duckdb-parquet | q19 | 0.552000 | 0.405000 | 0.034000 | 0.548811 | 1.906251 | 932.34 |
| 10m | duckdb-parquet | q20 | 0.138000 | 0.140000 | 0.004000 | 0.283812 | 0.307113 | 156.48 |
| 10m | duckdb-parquet | q21 | 0.314000 | 0.297000 | 0.018000 | 0.424217 | 1.240487 | 217.23 |
| 10m | duckdb-parquet | q22 | 0.268000 | 0.276000 | 0.008000 | 0.403148 | 1.099868 | 257.84 |
| 10m | duckdb-parquet | q23 | 0.363000 | 0.469000 | 0.178000 | 0.631724 | 1.948059 | 240.49 |
| 10m | duckdb-parquet | q24 | 0.569000 | 0.556000 | 0.177000 | 0.715207 | 2.037420 | 360.29 |
| 10m | duckdb-parquet | q25 | 0.296000 | 0.306000 | 0.127000 | 0.451557 | 0.752263 | 291.73 |
| 10m | duckdb-parquet | q26 | 0.254000 | 0.251000 | 0.121000 | 0.447753 | 0.731110 | 171.29 |
| 10m | duckdb-parquet | q27 | 0.231000 | 0.197000 | 0.024000 | 0.328673 | 0.615912 | 178.53 |
| 10m | duckdb-parquet | q28 | 0.436000 | 0.292000 | 0.073000 | 0.432412 | 1.186246 | 259.55 |
| 10m | duckdb-parquet | q29 | 1.823000 | 1.304000 | 0.503000 | 1.461792 | 7.117077 | 515.21 |
| 10m | duckdb-parquet | q30 | 0.243000 | 0.145000 | 0.006000 | 0.277991 | 0.327909 | 166.02 |
| 10m | duckdb-parquet | q31 | 0.207000 | 0.228000 | 0.088000 | 0.362103 | 0.762935 | 295.04 |
| 10m | duckdb-parquet | q32 | 1.353000 | 0.598000 | 0.433000 | 0.860254 | 1.700030 | 317.71 |
| 10m | duckdb-parquet | q33 | 0.345000 | 0.325000 | 0.021000 | 0.467838 | 1.422582 | 850.12 |
| 10m | duckdb-parquet | q34 | 0.437000 | 0.436000 | 0.060000 | 0.574986 | 2.125660 | 1147.24 |
| 10m | duckdb-parquet | q35 | 0.442000 | 0.437000 | 0.008000 | 0.584041 | 2.168327 | 1240.74 |
| 10m | duckdb-parquet | q36 | 0.209000 | 0.207000 | 0.015000 | 0.334449 | 0.744280 | 387.16 |
| 10m | duckdb-parquet | q37 | 0.181000 | 0.148000 | 0.006000 | 0.273618 | 0.322063 | 186.18 |
| 10m | duckdb-parquet | q38 | 0.146000 | 0.155000 | 0.045000 | 0.307587 | 0.363486 | 172.56 |
| 10m | duckdb-parquet | q39 | 0.153000 | 0.147000 | 0.134000 | 0.299136 | 0.339424 | 175.03 |
| 10m | duckdb-parquet | q40 | 0.151000 | 0.159000 | 0.087000 | 0.296165 | 0.359598 | 202.11 |
| 10m | duckdb-parquet | q41 | 0.147000 | 0.147000 | 0.009000 | 0.268597 | 0.298052 | 165.80 |
| 10m | duckdb-parquet | q42 | 0.146000 | 0.149000 | 0.003000 | 0.275186 | 0.316440 | 166.96 |
| 10m | duckdb-parquet | q43 | 0.233000 | 0.145000 | 0.004000 | 0.274391 | 0.303889 | 166.54 |
| 10m | rudb-parquet | q1 | 0.111901 | 0.108437 | 0.002381 | 0.109897 | 0.112777 | 118.35 |
| 10m | rudb-parquet | q2 | 0.125921 | 0.111905 | 0.005264 | 0.113512 | 0.132955 | 118.35 |
| 10m | rudb-parquet | q3 | 0.144156 | 0.109573 | 0.006939 | 0.111027 | 0.137656 | 118.34 |
| 10m | rudb-parquet | q4 | 0.113017 | 0.110511 | 0.005031 | 0.111945 | 0.141804 | 118.34 |
| 10m | rudb-parquet | q5 | 0.140602 | 0.144520 | 0.004686 | 0.152709 | 0.344497 | 227.61 |
| 10m | rudb-parquet | q6 | 0.208952 | 0.210352 | 0.003943 | 0.220723 | 0.640207 | 265.84 |
| 10m | rudb-parquet | q7 | 0.129654 | 0.112364 | 0.005443 | 0.113804 | 0.129785 | 118.34 |
| 10m | rudb-parquet | q8 | 0.113531 | 0.114875 | 0.009939 | 0.116464 | 0.136610 | 118.35 |
| 10m | rudb-parquet | q9 | 0.149068 | 0.139222 | 0.003658 | 0.143919 | 0.309269 | 215.57 |
| 10m | rudb-parquet | q10 | 0.156076 | 0.150694 | 0.001792 | 0.159011 | 0.382239 | 218.32 |
| 10m | rudb-parquet | q11 | 0.155184 | 0.134746 | 0.006047 | 0.137225 | 0.250926 | 128.87 |
| 10m | rudb-parquet | q12 | 0.142498 | 0.141669 | 0.014831 | 0.147019 | 0.275853 | 125.61 |
| 10m | rudb-parquet | q13 | 0.193576 | 0.272520 | 0.413092 | 0.286953 | 0.806150 | 239.78 |
| 10m | rudb-parquet | q14 | 1.036785 | 0.383173 | 0.296534 | 0.399851 | 1.243228 | 322.03 |
| 10m | rudb-parquet | q15 | 0.199968 | 0.198880 | 0.007083 | 0.205997 | 0.615303 | 248.93 |
| 10m | rudb-parquet | q16 | 0.175895 | 0.142286 | 0.049910 | 0.143852 | 0.262828 | 220.16 |
| 10m | rudb-parquet | q17 | 0.352752 | 0.353887 | 0.016707 | 0.369477 | 1.329512 | 608.11 |
| 10m | rudb-parquet | q18 | 0.161369 | 0.160039 | 0.010371 | 0.165709 | 0.441332 | 118.32 |
| 10m | rudb-parquet | q19 | 0.874034 | 0.570214 | 0.014463 | 0.590826 | 2.162652 | 1088.79 |
| 10m | rudb-parquet | q20 | 0.113519 | 0.120087 | 0.008059 | 0.121778 | 0.141260 | 118.29 |
| 10m | rudb-parquet | q21 | 0.294072 | 0.290202 | 0.046991 | 0.296510 | 1.185745 | 184.55 |
| 10m | rudb-parquet | q22 | 0.293077 | 0.292801 | 0.097504 | 0.299127 | 1.207381 | 165.62 |
| 10m | rudb-parquet | q23 | 0.974286 | 0.621948 | 0.347698 | 0.647022 | 3.147673 | 175.11 |
| 10m | rudb-parquet | q24 | 0.442819 | 0.438165 | 0.199533 | 0.439719 | 1.602648 | 240.46 |
| 10m | rudb-parquet | q25 | 0.176411 | 0.176551 | 0.018424 | 0.182552 | 0.479083 | 122.69 |
| 10m | rudb-parquet | q26 | 0.249380 | 0.186828 | 0.095002 | 0.188694 | 0.453307 | 131.79 |
| 10m | rudb-parquet | q27 | 0.229242 | 0.167014 | 0.012475 | 0.172666 | 0.483061 | 123.08 |
| 10m | rudb-parquet | q28 | 0.387882 | 0.345914 | 0.090601 | 0.355598 | 1.340759 | 171.06 |
| 10m | rudb-parquet | q29 | 0.862876 | 0.730330 | 0.654623 | 0.779124 | 3.258763 | 461.62 |
| 10m | rudb-parquet | q30 | 0.166776 | 0.121056 | 0.021785 | 0.122579 | 0.143100 | 118.51 |
| 10m | rudb-parquet | q31 | 0.181516 | 0.176867 | 0.086109 | 0.178468 | 0.483850 | 146.62 |
| 10m | rudb-parquet | q32 | 0.790783 | 0.345964 | 0.104460 | 0.376176 | 0.982959 | 152.82 |
| 10m | rudb-parquet | q33 | 0.181863 | 0.161295 | 0.010325 | 0.173391 | 0.434721 | 291.96 |
| 10m | rudb-parquet | q34 | 0.541677 | 0.531121 | 0.032496 | 0.554755 | 2.223782 | 980.21 |
| 10m | rudb-parquet | q35 | 0.520800 | 0.508024 | 0.006671 | 0.529516 | 2.116364 | 991.29 |
| 10m | rudb-parquet | q36 | 0.133680 | 0.134841 | 0.018589 | 0.136324 | 0.261230 | 219.54 |
| 10m | rudb-parquet | q37 | 0.117707 | 0.115350 | 0.002594 | 0.116975 | 0.140798 | 133.79 |
| 10m | rudb-parquet | q38 | 0.107651 | 0.121021 | 0.077053 | 0.122499 | 0.135317 | 118.54 |
| 10m | rudb-parquet | q39 | 0.113957 | 0.109368 | 0.095591 | 0.111019 | 0.121808 | 119.32 |
| 10m | rudb-parquet | q40 | 0.125429 | 0.128060 | 0.009584 | 0.129610 | 0.182436 | 161.29 |
| 10m | rudb-parquet | q41 | 0.113461 | 0.109129 | 0.003948 | 0.110720 | 0.114668 | 118.53 |
| 10m | rudb-parquet | q42 | 0.109487 | 0.110591 | 0.008995 | 0.112206 | 0.115724 | 118.50 |
| 10m | rudb-parquet | q43 | 0.169448 | 0.107357 | 0.005997 | 0.108892 | 0.112735 | 118.21 |
