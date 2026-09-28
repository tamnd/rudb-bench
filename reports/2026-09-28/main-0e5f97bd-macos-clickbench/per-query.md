# ClickBench measurement audit

All 43 SQL queries are attempted on DuckDB native, rudb native, DuckDB Parquet, and rudb Parquet. 7 hot repetitions are required for a complete row. A failed repetition invalidates that query; successful fragments are never averaged into a result.

First execution is not disk-cold: the page cache is not flushed. Each repetition uses a fresh process. Query seconds are the CLI timer, including result rendering; wall and CPU seconds and peak RSS cover the whole child process. CPU and RSS come from wait4 for that child. RSS is the maximum resident set, not allocated bytes or an incremental memory delta. Both native rows open a loaded single-file database. Both Parquet rows query the same source file with native mirroring disabled; metadata-only paths remain available. These are sample results, not official ClickBench scores.

| Size | Engine | Load wall (s) | Load CPU (s) | Load peak RSS (MiB) | Native bytes |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 0.042349 | 0.042954 | 53.59 | 1060864 |
| 1k | rudb-native | 0.105619 | 0.088822 | 34.11 | 1876635 |
| 10k | duckdb-native | 0.108724 | 0.169753 | 68.98 | 4730880 |
| 10k | rudb-native | 0.390307 | 0.394696 | 68.55 | 7732602 |
| 1m | duckdb-native | 5.457988 | 11.787111 | 1262.22 | 454832128 |
| 1m | rudb-native | 3.100947 | 11.248987 | 1224.31 | 246899372 |
| 10m | duckdb-native | 58.690621 | 117.735968 | 1525.41 | 2342531072 |
| 10m | rudb-native | 25.330825 | 89.533552 | 2314.44 | 1468833892 |

| Size | Engine | Complete / 43 | Query median sum (s) | Process wall median sum (s) | CPU median sum (s) | Peak RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 43 | 0.057000 | 0.765230 | 0.772180 | 33.78 |
| 1k | rudb-native | 43 | 0.041530 | 0.248534 | 0.209965 | 11.95 |
| 1k | duckdb-parquet | 43 | 0.071000 | 0.766781 | 0.776507 | 32.45 |
| 1k | rudb-parquet | 43 | 0.047199 | 0.250550 | 0.212606 | 11.55 |
| 10k | duckdb-native | 43 | 0.091000 | 0.926786 | 0.956336 | 38.28 |
| 10k | rudb-native | 43 | 0.057632 | 0.310059 | 0.270490 | 14.67 |
| 10k | duckdb-parquet | 43 | 0.112000 | 0.946574 | 0.972666 | 37.09 |
| 10k | rudb-parquet | 43 | 0.086244 | 0.323249 | 0.279051 | 14.75 |
| 1m | duckdb-native | 43 | 0.802000 | 1.837202 | 4.602681 | 271.95 |
| 1m | rudb-native | 43 | 0.280693 | 0.589857 | 1.122177 | 82.62 |
| 1m | duckdb-parquet | 43 | 1.161000 | 2.191874 | 5.708960 | 405.28 |
| 1m | rudb-parquet | 43 | 1.198498 | 1.547823 | 4.815067 | 339.66 |
| 10m | duckdb-native | 43 | 5.378000 | 7.325699 | 30.012010 | 1164.67 |
| 10m | rudb-native | 43 | 1.145699 | 1.602111 | 5.054315 | 279.86 |
| 10m | duckdb-parquet | 43 | 7.576000 | 9.395098 | 39.080299 | 995.81 |
| 10m | rudb-parquet | 43 | 6.936277 | 7.451229 | 35.642443 | 804.72 |

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
| 1k | duckdb-native | q1 | 0.001000 | 0.001000 | 0.001000 | 0.015686 | 0.014727 | 18.89 |
| 1k | duckdb-native | q2 | 0.001000 | 0.001000 | 0.001000 | 0.016303 | 0.015383 | 18.89 |
| 1k | duckdb-native | q3 | 0.000000 | 0.001000 | 0.001000 | 0.016561 | 0.015738 | 19.48 |
| 1k | duckdb-native | q4 | 0.000000 | 0.000000 | 0.001000 | 0.017184 | 0.016256 | 20.59 |
| 1k | duckdb-native | q5 | 0.001000 | 0.001000 | 0.000000 | 0.017162 | 0.016889 | 23.11 |
| 1k | duckdb-native | q6 | 0.001000 | 0.001000 | 0.000000 | 0.016769 | 0.016578 | 23.33 |
| 1k | duckdb-native | q7 | 0.001000 | 0.000000 | 0.001000 | 0.016291 | 0.015291 | 19.19 |
| 1k | duckdb-native | q8 | 0.001000 | 0.001000 | 0.001000 | 0.017104 | 0.016026 | 20.08 |
| 1k | duckdb-native | q9 | 0.003000 | 0.002000 | 0.001000 | 0.019523 | 0.020820 | 31.52 |
| 1k | duckdb-native | q10 | 0.002000 | 0.002000 | 0.001000 | 0.018356 | 0.020365 | 32.27 |
| 1k | duckdb-native | q11 | 0.001000 | 0.001000 | 0.001000 | 0.017397 | 0.019095 | 29.91 |
| 1k | duckdb-native | q12 | 0.001000 | 0.002000 | 0.001000 | 0.018119 | 0.020124 | 31.33 |
| 1k | duckdb-native | q13 | 0.001000 | 0.001000 | 0.001000 | 0.017787 | 0.018105 | 25.50 |
| 1k | duckdb-native | q14 | 0.002000 | 0.001000 | 0.001000 | 0.017980 | 0.020142 | 32.94 |
| 1k | duckdb-native | q15 | 0.001000 | 0.001000 | 0.000000 | 0.017637 | 0.018143 | 25.72 |
| 1k | duckdb-native | q16 | 0.001000 | 0.001000 | 0.000000 | 0.017485 | 0.017648 | 25.44 |
| 1k | duckdb-native | q17 | 0.001000 | 0.001000 | 0.001000 | 0.017682 | 0.018319 | 27.22 |
| 1k | duckdb-native | q18 | 0.001000 | 0.001000 | 0.000000 | 0.017151 | 0.017210 | 25.73 |
| 1k | duckdb-native | q19 | 0.002000 | 0.001000 | 0.001000 | 0.017805 | 0.018560 | 27.61 |
| 1k | duckdb-native | q20 | 0.001000 | 0.001000 | 0.001000 | 0.016351 | 0.015355 | 18.50 |
| 1k | duckdb-native | q21 | 0.001000 | 0.001000 | 0.001000 | 0.017210 | 0.016081 | 19.00 |
| 1k | duckdb-native | q22 | 0.001000 | 0.001000 | 0.001000 | 0.016362 | 0.015706 | 20.03 |
| 1k | duckdb-native | q23 | 0.001000 | 0.001000 | 0.001000 | 0.017257 | 0.016774 | 21.47 |
| 1k | duckdb-native | q24 | 0.004000 | 0.003000 | 0.001000 | 0.020076 | 0.021236 | 33.78 |
| 1k | duckdb-native | q25 | 0.001000 | 0.001000 | 0.000000 | 0.016850 | 0.016721 | 21.39 |
| 1k | duckdb-native | q26 | 0.001000 | 0.001000 | 0.001000 | 0.016224 | 0.015555 | 18.98 |
| 1k | duckdb-native | q27 | 0.001000 | 0.001000 | 0.001000 | 0.016697 | 0.015722 | 20.42 |
| 1k | duckdb-native | q28 | 0.002000 | 0.001000 | 0.001000 | 0.018036 | 0.018073 | 26.72 |
| 1k | duckdb-native | q29 | 0.002000 | 0.002000 | 0.000000 | 0.017943 | 0.019135 | 27.06 |
| 1k | duckdb-native | q30 | 0.004000 | 0.005000 | 0.001000 | 0.021035 | 0.020676 | 29.80 |
| 1k | duckdb-native | q31 | 0.003000 | 0.002000 | 0.001000 | 0.018051 | 0.018507 | 27.27 |
| 1k | duckdb-native | q32 | 0.003000 | 0.002000 | 0.000000 | 0.018033 | 0.018508 | 27.00 |
| 1k | duckdb-native | q33 | 0.001000 | 0.002000 | 0.000000 | 0.018804 | 0.020108 | 27.27 |
| 1k | duckdb-native | q34 | 0.001000 | 0.001000 | 0.001000 | 0.019488 | 0.020534 | 25.78 |
| 1k | duckdb-native | q35 | 0.001000 | 0.002000 | 0.001000 | 0.018572 | 0.019403 | 26.22 |
| 1k | duckdb-native | q36 | 0.002000 | 0.001000 | 0.001000 | 0.019166 | 0.019676 | 26.91 |
| 1k | duckdb-native | q37 | 0.002000 | 0.002000 | 0.001000 | 0.019170 | 0.020126 | 25.33 |
| 1k | duckdb-native | q38 | 0.002000 | 0.001000 | 0.001000 | 0.019484 | 0.019664 | 24.66 |
| 1k | duckdb-native | q39 | 0.001000 | 0.001000 | 0.000000 | 0.018999 | 0.018313 | 21.11 |
| 1k | duckdb-native | q40 | 0.003000 | 0.001000 | 0.001000 | 0.018000 | 0.018516 | 27.47 |
| 1k | duckdb-native | q41 | 0.001000 | 0.002000 | 0.001000 | 0.017657 | 0.017245 | 24.66 |
| 1k | duckdb-native | q42 | 0.001000 | 0.001000 | 0.001000 | 0.017459 | 0.017737 | 23.08 |
| 1k | duckdb-native | q43 | 0.001000 | 0.001000 | 0.000000 | 0.018324 | 0.017390 | 24.69 |
| 1k | rudb-native | q1 | 0.000578 | 0.000584 | 0.000050 | 0.005212 | 0.004405 | 8.98 |
| 1k | rudb-native | q2 | 0.000762 | 0.000649 | 0.000053 | 0.005450 | 0.004629 | 9.55 |
| 1k | rudb-native | q3 | 0.000683 | 0.000682 | 0.000037 | 0.005518 | 0.004623 | 9.20 |
| 1k | rudb-native | q4 | 0.000708 | 0.000626 | 0.000036 | 0.005309 | 0.004403 | 9.14 |
| 1k | rudb-native | q5 | 0.000622 | 0.000579 | 0.000026 | 0.005167 | 0.004334 | 9.05 |
| 1k | rudb-native | q6 | 0.000808 | 0.000611 | 0.000024 | 0.005213 | 0.004395 | 9.33 |
| 1k | rudb-native | q7 | 0.000610 | 0.000641 | 0.000086 | 0.005225 | 0.004388 | 9.38 |
| 1k | rudb-native | q8 | 0.001074 | 0.000952 | 0.000093 | 0.005602 | 0.004747 | 10.94 |
| 1k | rudb-native | q9 | 0.000939 | 0.000888 | 0.000076 | 0.005543 | 0.004665 | 10.38 |
| 1k | rudb-native | q10 | 0.001203 | 0.001164 | 0.000097 | 0.005838 | 0.004981 | 10.62 |
| 1k | rudb-native | q11 | 0.000868 | 0.000926 | 0.000037 | 0.005613 | 0.004757 | 10.83 |
| 1k | rudb-native | q12 | 0.001056 | 0.001013 | 0.000100 | 0.005775 | 0.004976 | 10.78 |
| 1k | rudb-native | q13 | 0.001107 | 0.001104 | 0.000239 | 0.005820 | 0.005071 | 11.86 |
| 1k | rudb-native | q14 | 0.000959 | 0.001022 | 0.000025 | 0.005874 | 0.004993 | 10.59 |
| 1k | rudb-native | q15 | 0.001104 | 0.001212 | 0.000092 | 0.005834 | 0.004914 | 10.77 |
| 1k | rudb-native | q16 | 0.000763 | 0.000719 | 0.000124 | 0.005374 | 0.004518 | 9.61 |
| 1k | rudb-native | q17 | 0.000704 | 0.000723 | 0.000051 | 0.005417 | 0.004604 | 9.48 |
| 1k | rudb-native | q18 | 0.000728 | 0.000696 | 0.000057 | 0.005106 | 0.004324 | 9.50 |
| 1k | rudb-native | q19 | 0.000792 | 0.000891 | 0.000240 | 0.005793 | 0.004793 | 9.81 |
| 1k | rudb-native | q20 | 0.000666 | 0.000636 | 0.000079 | 0.005214 | 0.004400 | 9.53 |
| 1k | rudb-native | q21 | 0.000888 | 0.000900 | 0.000049 | 0.005558 | 0.004705 | 10.50 |
| 1k | rudb-native | q22 | 0.001303 | 0.000987 | 0.000053 | 0.005937 | 0.004951 | 10.73 |
| 1k | rudb-native | q23 | 0.001274 | 0.001065 | 0.000135 | 0.005945 | 0.005042 | 11.06 |
| 1k | rudb-native | q24 | 0.000897 | 0.000890 | 0.000029 | 0.005655 | 0.004788 | 9.97 |
| 1k | rudb-native | q25 | 0.000809 | 0.000862 | 0.000056 | 0.005657 | 0.004685 | 10.00 |
| 1k | rudb-native | q26 | 0.000834 | 0.000829 | 0.000048 | 0.005395 | 0.004555 | 10.06 |
| 1k | rudb-native | q27 | 0.000869 | 0.000883 | 0.000040 | 0.005453 | 0.004629 | 9.92 |
| 1k | rudb-native | q28 | 0.001049 | 0.001033 | 0.000054 | 0.005700 | 0.004844 | 11.55 |
| 1k | rudb-native | q29 | 0.001297 | 0.001269 | 0.000081 | 0.005953 | 0.005130 | 11.39 |
| 1k | rudb-native | q30 | 0.002563 | 0.002523 | 0.000197 | 0.007358 | 0.006473 | 10.58 |
| 1k | rudb-native | q31 | 0.001172 | 0.001150 | 0.000072 | 0.005975 | 0.004953 | 10.86 |
| 1k | rudb-native | q32 | 0.000958 | 0.000912 | 0.000079 | 0.005955 | 0.004866 | 9.94 |
| 1k | rudb-native | q33 | 0.000755 | 0.000782 | 0.000096 | 0.005483 | 0.004650 | 9.80 |
| 1k | rudb-native | q34 | 0.000846 | 0.000908 | 0.000109 | 0.005818 | 0.004968 | 10.25 |
| 1k | rudb-native | q35 | 0.001057 | 0.001088 | 0.000198 | 0.006068 | 0.005318 | 11.09 |
| 1k | rudb-native | q36 | 0.001093 | 0.001061 | 0.000020 | 0.005898 | 0.005054 | 10.59 |
| 1k | rudb-native | q37 | 0.001368 | 0.001409 | 0.000345 | 0.007867 | 0.006132 | 11.95 |
| 1k | rudb-native | q38 | 0.001481 | 0.001526 | 0.000125 | 0.006910 | 0.006025 | 11.33 |
| 1k | rudb-native | q39 | 0.001060 | 0.001065 | 0.000141 | 0.006803 | 0.005715 | 11.22 |
| 1k | rudb-native | q40 | 0.001135 | 0.001038 | 0.000092 | 0.005675 | 0.004829 | 11.08 |
| 1k | rudb-native | q41 | 0.000960 | 0.001000 | 0.000078 | 0.005873 | 0.004888 | 11.09 |
| 1k | rudb-native | q42 | 0.000983 | 0.001015 | 0.000043 | 0.005633 | 0.004757 | 11.50 |
| 1k | rudb-native | q43 | 0.001026 | 0.001021 | 0.000161 | 0.006068 | 0.005088 | 11.20 |
| 1k | duckdb-parquet | q1 | 0.000000 | 0.001000 | 0.000000 | 0.015994 | 0.014954 | 18.62 |
| 1k | duckdb-parquet | q2 | 0.001000 | 0.001000 | 0.000000 | 0.016539 | 0.015907 | 18.70 |
| 1k | duckdb-parquet | q3 | 0.001000 | 0.001000 | 0.000000 | 0.016816 | 0.015999 | 18.66 |
| 1k | duckdb-parquet | q4 | 0.001000 | 0.001000 | 0.001000 | 0.016819 | 0.015791 | 18.80 |
| 1k | duckdb-parquet | q5 | 0.001000 | 0.001000 | 0.000000 | 0.017121 | 0.016870 | 22.47 |
| 1k | duckdb-parquet | q6 | 0.001000 | 0.001000 | 0.001000 | 0.017087 | 0.016841 | 22.89 |
| 1k | duckdb-parquet | q7 | 0.001000 | 0.001000 | 0.000000 | 0.016857 | 0.015381 | 19.50 |
| 1k | duckdb-parquet | q8 | 0.001000 | 0.001000 | 0.001000 | 0.017081 | 0.016461 | 19.77 |
| 1k | duckdb-parquet | q9 | 0.001000 | 0.002000 | 0.000000 | 0.018397 | 0.020030 | 30.41 |
| 1k | duckdb-parquet | q10 | 0.002000 | 0.002000 | 0.000000 | 0.018087 | 0.020243 | 31.55 |
| 1k | duckdb-parquet | q11 | 0.002000 | 0.002000 | 0.000000 | 0.017908 | 0.018894 | 28.88 |
| 1k | duckdb-parquet | q12 | 0.002000 | 0.002000 | 0.001000 | 0.017520 | 0.018892 | 30.50 |
| 1k | duckdb-parquet | q13 | 0.002000 | 0.002000 | 0.001000 | 0.017431 | 0.018051 | 25.67 |
| 1k | duckdb-parquet | q14 | 0.003000 | 0.002000 | 0.002000 | 0.018436 | 0.019766 | 30.92 |
| 1k | duckdb-parquet | q15 | 0.001000 | 0.001000 | 0.001000 | 0.017521 | 0.017980 | 25.98 |
| 1k | duckdb-parquet | q16 | 0.002000 | 0.002000 | 0.001000 | 0.017647 | 0.017839 | 24.27 |
| 1k | duckdb-parquet | q17 | 0.002000 | 0.002000 | 0.001000 | 0.017529 | 0.018225 | 25.70 |
| 1k | duckdb-parquet | q18 | 0.002000 | 0.001000 | 0.001000 | 0.017102 | 0.016950 | 24.97 |
| 1k | duckdb-parquet | q19 | 0.002000 | 0.002000 | 0.000000 | 0.018003 | 0.018721 | 26.70 |
| 1k | duckdb-parquet | q20 | 0.001000 | 0.001000 | 0.000000 | 0.016263 | 0.015571 | 18.81 |
| 1k | duckdb-parquet | q21 | 0.001000 | 0.001000 | 0.000000 | 0.016915 | 0.016194 | 18.56 |
| 1k | duckdb-parquet | q22 | 0.001000 | 0.001000 | 0.000000 | 0.017288 | 0.016767 | 19.73 |
| 1k | duckdb-parquet | q23 | 0.001000 | 0.001000 | 0.001000 | 0.017262 | 0.016941 | 20.33 |
| 1k | duckdb-parquet | q24 | 0.004000 | 0.004000 | 0.000000 | 0.020323 | 0.021374 | 32.45 |
| 1k | duckdb-parquet | q25 | 0.001000 | 0.002000 | 0.001000 | 0.017549 | 0.017332 | 21.16 |
| 1k | duckdb-parquet | q26 | 0.001000 | 0.001000 | 0.000000 | 0.016791 | 0.015939 | 19.19 |
| 1k | duckdb-parquet | q27 | 0.001000 | 0.001000 | 0.000000 | 0.017047 | 0.016182 | 19.19 |
| 1k | duckdb-parquet | q28 | 0.001000 | 0.001000 | 0.001000 | 0.017340 | 0.017983 | 23.72 |
| 1k | duckdb-parquet | q29 | 0.002000 | 0.002000 | 0.001000 | 0.018312 | 0.020032 | 27.12 |
| 1k | duckdb-parquet | q30 | 0.004000 | 0.005000 | 0.001000 | 0.020407 | 0.020341 | 28.73 |
| 1k | duckdb-parquet | q31 | 0.002000 | 0.002000 | 0.000000 | 0.019271 | 0.020134 | 26.69 |
| 1k | duckdb-parquet | q32 | 0.001000 | 0.002000 | 0.001000 | 0.018402 | 0.019428 | 25.69 |
| 1k | duckdb-parquet | q33 | 0.001000 | 0.002000 | 0.000000 | 0.018438 | 0.019348 | 26.14 |
| 1k | duckdb-parquet | q34 | 0.002000 | 0.001000 | 0.001000 | 0.018893 | 0.020052 | 25.91 |
| 1k | duckdb-parquet | q35 | 0.001000 | 0.002000 | 0.001000 | 0.018748 | 0.019623 | 26.02 |
| 1k | duckdb-parquet | q36 | 0.002000 | 0.002000 | 0.000000 | 0.019007 | 0.019702 | 25.62 |
| 1k | duckdb-parquet | q37 | 0.002000 | 0.002000 | 0.000000 | 0.019426 | 0.020086 | 24.47 |
| 1k | duckdb-parquet | q38 | 0.002000 | 0.002000 | 0.000000 | 0.019541 | 0.020355 | 24.48 |
| 1k | duckdb-parquet | q39 | 0.001000 | 0.001000 | 0.000000 | 0.018538 | 0.017890 | 19.69 |
| 1k | duckdb-parquet | q40 | 0.002000 | 0.002000 | 0.000000 | 0.017974 | 0.018263 | 26.55 |
| 1k | duckdb-parquet | q41 | 0.002000 | 0.002000 | 0.001000 | 0.017898 | 0.018206 | 24.44 |
| 1k | duckdb-parquet | q42 | 0.002000 | 0.001000 | 0.001000 | 0.017457 | 0.017269 | 23.88 |
| 1k | duckdb-parquet | q43 | 0.001000 | 0.002000 | 0.001000 | 0.017796 | 0.017700 | 23.61 |
| 1k | rudb-parquet | q1 | 0.000916 | 0.000833 | 0.000074 | 0.005563 | 0.004666 | 9.50 |
| 1k | rudb-parquet | q2 | 0.000964 | 0.000901 | 0.000082 | 0.005471 | 0.004623 | 9.84 |
| 1k | rudb-parquet | q3 | 0.000967 | 0.000885 | 0.000096 | 0.005999 | 0.005048 | 10.17 |
| 1k | rudb-parquet | q4 | 0.000934 | 0.000865 | 0.000036 | 0.005638 | 0.004708 | 9.75 |
| 1k | rudb-parquet | q5 | 0.000795 | 0.000836 | 0.000059 | 0.005464 | 0.004616 | 9.88 |
| 1k | rudb-parquet | q6 | 0.001000 | 0.000976 | 0.000095 | 0.005739 | 0.004843 | 10.47 |
| 1k | rudb-parquet | q7 | 0.000628 | 0.000591 | 0.000042 | 0.005613 | 0.004571 | 9.33 |
| 1k | rudb-parquet | q8 | 0.000991 | 0.000951 | 0.000026 | 0.005630 | 0.004823 | 10.11 |
| 1k | rudb-parquet | q9 | 0.001028 | 0.000928 | 0.000086 | 0.005569 | 0.004650 | 10.05 |
| 1k | rudb-parquet | q10 | 0.001315 | 0.001226 | 0.000108 | 0.005938 | 0.005078 | 10.23 |
| 1k | rudb-parquet | q11 | 0.000963 | 0.000966 | 0.000166 | 0.005784 | 0.004929 | 10.52 |
| 1k | rudb-parquet | q12 | 0.000988 | 0.000993 | 0.000070 | 0.005512 | 0.004643 | 10.08 |
| 1k | rudb-parquet | q13 | 0.000959 | 0.001008 | 0.000108 | 0.005626 | 0.004710 | 10.00 |
| 1k | rudb-parquet | q14 | 0.001021 | 0.001018 | 0.000041 | 0.005681 | 0.004817 | 10.50 |
| 1k | rudb-parquet | q15 | 0.001031 | 0.001008 | 0.000094 | 0.005678 | 0.004784 | 9.98 |
| 1k | rudb-parquet | q16 | 0.001257 | 0.000982 | 0.000080 | 0.005546 | 0.004717 | 9.83 |
| 1k | rudb-parquet | q17 | 0.001107 | 0.001027 | 0.000070 | 0.006040 | 0.005104 | 9.98 |
| 1k | rudb-parquet | q18 | 0.001011 | 0.000945 | 0.000074 | 0.005492 | 0.004656 | 10.08 |
| 1k | rudb-parquet | q19 | 0.001158 | 0.001139 | 0.000144 | 0.005715 | 0.004786 | 10.20 |
| 1k | rudb-parquet | q20 | 0.000720 | 0.000702 | 0.000049 | 0.005100 | 0.004302 | 9.41 |
| 1k | rudb-parquet | q21 | 0.000987 | 0.001074 | 0.000102 | 0.005742 | 0.004867 | 10.67 |
| 1k | rudb-parquet | q22 | 0.001172 | 0.001100 | 0.000070 | 0.005706 | 0.004850 | 10.47 |
| 1k | rudb-parquet | q23 | 0.001339 | 0.001400 | 0.000093 | 0.006043 | 0.005206 | 10.50 |
| 1k | rudb-parquet | q24 | 0.001047 | 0.001099 | 0.000098 | 0.005897 | 0.004982 | 9.77 |
| 1k | rudb-parquet | q25 | 0.000871 | 0.000883 | 0.000090 | 0.005531 | 0.004669 | 9.48 |
| 1k | rudb-parquet | q26 | 0.001039 | 0.000820 | 0.000122 | 0.005535 | 0.004532 | 9.38 |
| 1k | rudb-parquet | q27 | 0.000896 | 0.000896 | 0.000094 | 0.005464 | 0.004644 | 9.42 |
| 1k | rudb-parquet | q28 | 0.001192 | 0.001167 | 0.000102 | 0.005962 | 0.005011 | 11.00 |
| 1k | rudb-parquet | q29 | 0.001305 | 0.001403 | 0.000162 | 0.005983 | 0.005041 | 10.72 |
| 1k | rudb-parquet | q30 | 0.003088 | 0.002859 | 0.000103 | 0.007625 | 0.006722 | 10.88 |
| 1k | rudb-parquet | q31 | 0.001212 | 0.001223 | 0.000080 | 0.005981 | 0.005040 | 10.19 |
| 1k | rudb-parquet | q32 | 0.001212 | 0.001191 | 0.000037 | 0.005773 | 0.004909 | 10.14 |
| 1k | rudb-parquet | q33 | 0.001220 | 0.001263 | 0.000125 | 0.005887 | 0.005072 | 10.14 |
| 1k | rudb-parquet | q34 | 0.001083 | 0.001197 | 0.000105 | 0.005983 | 0.005131 | 10.22 |
| 1k | rudb-parquet | q35 | 0.001175 | 0.001173 | 0.000052 | 0.006194 | 0.005259 | 10.31 |
| 1k | rudb-parquet | q36 | 0.001152 | 0.001119 | 0.000051 | 0.006077 | 0.005197 | 10.12 |
| 1k | rudb-parquet | q37 | 0.001290 | 0.001284 | 0.000332 | 0.006571 | 0.005495 | 10.52 |
| 1k | rudb-parquet | q38 | 0.001391 | 0.001434 | 0.000086 | 0.006373 | 0.005496 | 10.72 |
| 1k | rudb-parquet | q39 | 0.001200 | 0.001240 | 0.000078 | 0.006405 | 0.005546 | 11.55 |
| 1k | rudb-parquet | q40 | 0.001424 | 0.001353 | 0.000097 | 0.005821 | 0.005057 | 10.92 |
| 1k | rudb-parquet | q41 | 0.001049 | 0.001090 | 0.000093 | 0.005734 | 0.004871 | 10.50 |
| 1k | rudb-parquet | q42 | 0.001065 | 0.001069 | 0.000073 | 0.005888 | 0.005137 | 10.45 |
| 1k | rudb-parquet | q43 | 0.001021 | 0.001078 | 0.000322 | 0.005577 | 0.004798 | 10.62 |
| 10k | duckdb-native | q1 | 0.000000 | 0.001000 | 0.001000 | 0.017486 | 0.016134 | 18.91 |
| 10k | duckdb-native | q2 | 0.000000 | 0.001000 | 0.000000 | 0.017460 | 0.016069 | 19.22 |
| 10k | duckdb-native | q3 | 0.001000 | 0.001000 | 0.000000 | 0.018406 | 0.017520 | 19.64 |
| 10k | duckdb-native | q4 | 0.001000 | 0.001000 | 0.000000 | 0.018875 | 0.017574 | 19.73 |
| 10k | duckdb-native | q5 | 0.002000 | 0.001000 | 0.000000 | 0.020076 | 0.020321 | 24.53 |
| 10k | duckdb-native | q6 | 0.001000 | 0.001000 | 0.001000 | 0.020402 | 0.020000 | 24.56 |
| 10k | duckdb-native | q7 | 0.000000 | 0.001000 | 0.001000 | 0.019247 | 0.018145 | 18.77 |
| 10k | duckdb-native | q8 | 0.001000 | 0.001000 | 0.000000 | 0.019164 | 0.018440 | 20.02 |
| 10k | duckdb-native | q9 | 0.002000 | 0.002000 | 0.001000 | 0.021973 | 0.025044 | 31.88 |
| 10k | duckdb-native | q10 | 0.003000 | 0.003000 | 0.001000 | 0.022000 | 0.024015 | 34.58 |
| 10k | duckdb-native | q11 | 0.002000 | 0.002000 | 0.000000 | 0.020772 | 0.022625 | 31.86 |
| 10k | duckdb-native | q12 | 0.002000 | 0.002000 | 0.000000 | 0.021310 | 0.023646 | 32.08 |
| 10k | duckdb-native | q13 | 0.001000 | 0.001000 | 0.001000 | 0.019923 | 0.021027 | 25.73 |
| 10k | duckdb-native | q14 | 0.002000 | 0.002000 | 0.000000 | 0.021110 | 0.023233 | 33.83 |
| 10k | duckdb-native | q15 | 0.002000 | 0.002000 | 0.001000 | 0.021283 | 0.022390 | 27.02 |
| 10k | duckdb-native | q16 | 0.002000 | 0.002000 | 0.001000 | 0.020612 | 0.021475 | 26.97 |
| 10k | duckdb-native | q17 | 0.003000 | 0.002000 | 0.001000 | 0.021239 | 0.022788 | 28.23 |
| 10k | duckdb-native | q18 | 0.002000 | 0.002000 | 0.001000 | 0.021673 | 0.021885 | 26.64 |
| 10k | duckdb-native | q19 | 0.002000 | 0.003000 | 0.001000 | 0.022086 | 0.023960 | 29.02 |
| 10k | duckdb-native | q20 | 0.000000 | 0.001000 | 0.001000 | 0.019493 | 0.018725 | 18.88 |
| 10k | duckdb-native | q21 | 0.002000 | 0.002000 | 0.000000 | 0.020223 | 0.019313 | 19.72 |
| 10k | duckdb-native | q22 | 0.002000 | 0.001000 | 0.001000 | 0.020515 | 0.020605 | 21.28 |
| 10k | duckdb-native | q23 | 0.002000 | 0.002000 | 0.001000 | 0.021192 | 0.022675 | 28.08 |
| 10k | duckdb-native | q24 | 0.008000 | 0.006000 | 0.001000 | 0.025314 | 0.027382 | 38.28 |
| 10k | duckdb-native | q25 | 0.002000 | 0.002000 | 0.001000 | 0.020105 | 0.019901 | 22.23 |
| 10k | duckdb-native | q26 | 0.001000 | 0.001000 | 0.000000 | 0.020304 | 0.019082 | 20.06 |
| 10k | duckdb-native | q27 | 0.001000 | 0.001000 | 0.000000 | 0.020124 | 0.019081 | 20.02 |
| 10k | duckdb-native | q28 | 0.002000 | 0.002000 | 0.000000 | 0.021330 | 0.022770 | 26.20 |
| 10k | duckdb-native | q29 | 0.009000 | 0.008000 | 0.001000 | 0.027557 | 0.033850 | 28.56 |
| 10k | duckdb-native | q30 | 0.005000 | 0.005000 | 0.002000 | 0.025067 | 0.024987 | 30.62 |
| 10k | duckdb-native | q31 | 0.002000 | 0.002000 | 0.000000 | 0.022550 | 0.023130 | 29.47 |
| 10k | duckdb-native | q32 | 0.002000 | 0.002000 | 0.000000 | 0.020847 | 0.022344 | 28.33 |
| 10k | duckdb-native | q33 | 0.002000 | 0.002000 | 0.000000 | 0.021283 | 0.022476 | 28.75 |
| 10k | duckdb-native | q34 | 0.003000 | 0.003000 | 0.001000 | 0.021899 | 0.024137 | 29.25 |
| 10k | duckdb-native | q35 | 0.003000 | 0.003000 | 0.000000 | 0.023692 | 0.025117 | 29.61 |
| 10k | duckdb-native | q36 | 0.002000 | 0.002000 | 0.000000 | 0.021876 | 0.023004 | 27.56 |
| 10k | duckdb-native | q37 | 0.002000 | 0.002000 | 0.001000 | 0.021864 | 0.022637 | 26.86 |
| 10k | duckdb-native | q38 | 0.001000 | 0.002000 | 0.001000 | 0.023893 | 0.024440 | 28.16 |
| 10k | duckdb-native | q39 | 0.003000 | 0.002000 | 0.000000 | 0.026326 | 0.026844 | 26.80 |
| 10k | duckdb-native | q40 | 0.003000 | 0.003000 | 0.001000 | 0.024591 | 0.024760 | 31.00 |
| 10k | duckdb-native | q41 | 0.002000 | 0.002000 | 0.001000 | 0.024181 | 0.024766 | 28.27 |
| 10k | duckdb-native | q42 | 0.002000 | 0.002000 | 0.000000 | 0.024186 | 0.024419 | 26.95 |
| 10k | duckdb-native | q43 | 0.002000 | 0.002000 | 0.001000 | 0.023277 | 0.023600 | 27.23 |
| 10k | rudb-native | q1 | 0.000578 | 0.000555 | 0.000060 | 0.005343 | 0.004508 | 9.38 |
| 10k | rudb-native | q2 | 0.000646 | 0.000653 | 0.000133 | 0.005565 | 0.004704 | 9.77 |
| 10k | rudb-native | q3 | 0.000693 | 0.000702 | 0.000103 | 0.006163 | 0.005114 | 9.97 |
| 10k | rudb-native | q4 | 0.000661 | 0.000658 | 0.000021 | 0.006055 | 0.005195 | 9.39 |
| 10k | rudb-native | q5 | 0.000624 | 0.000699 | 0.000062 | 0.006441 | 0.005449 | 9.03 |
| 10k | rudb-native | q6 | 0.000630 | 0.000683 | 0.000084 | 0.006363 | 0.005512 | 9.09 |
| 10k | rudb-native | q7 | 0.000706 | 0.000698 | 0.000075 | 0.006124 | 0.005145 | 9.25 |
| 10k | rudb-native | q8 | 0.001136 | 0.001063 | 0.000089 | 0.006780 | 0.005960 | 11.00 |
| 10k | rudb-native | q9 | 0.001224 | 0.001257 | 0.000181 | 0.007233 | 0.006082 | 11.06 |
| 10k | rudb-native | q10 | 0.001575 | 0.001631 | 0.000102 | 0.007223 | 0.006254 | 11.38 |
| 10k | rudb-native | q11 | 0.001097 | 0.001131 | 0.000060 | 0.007048 | 0.006207 | 11.48 |
| 10k | rudb-native | q12 | 0.001223 | 0.001143 | 0.000309 | 0.006669 | 0.005818 | 11.47 |
| 10k | rudb-native | q13 | 0.001348 | 0.001325 | 0.000349 | 0.007058 | 0.006374 | 12.09 |
| 10k | rudb-native | q14 | 0.001575 | 0.001342 | 0.000177 | 0.007177 | 0.006322 | 12.28 |
| 10k | rudb-native | q15 | 0.001773 | 0.001796 | 0.000305 | 0.007566 | 0.006759 | 12.23 |
| 10k | rudb-native | q16 | 0.001266 | 0.001185 | 0.000149 | 0.006703 | 0.005735 | 10.86 |
| 10k | rudb-native | q17 | 0.001469 | 0.001658 | 0.000192 | 0.007560 | 0.006745 | 12.06 |
| 10k | rudb-native | q18 | 0.001099 | 0.001138 | 0.000075 | 0.006874 | 0.005842 | 10.44 |
| 10k | rudb-native | q19 | 0.001768 | 0.001784 | 0.000219 | 0.007704 | 0.006897 | 12.19 |
| 10k | rudb-native | q20 | 0.000689 | 0.000727 | 0.000459 | 0.006546 | 0.005538 | 10.50 |
| 10k | rudb-native | q21 | 0.001121 | 0.001099 | 0.000066 | 0.006562 | 0.005616 | 10.62 |
| 10k | rudb-native | q22 | 0.001287 | 0.001398 | 0.000128 | 0.007236 | 0.006332 | 11.42 |
| 10k | rudb-native | q23 | 0.003187 | 0.003254 | 0.000170 | 0.009059 | 0.008135 | 13.02 |
| 10k | rudb-native | q24 | 0.001287 | 0.001239 | 0.000106 | 0.006897 | 0.005876 | 10.62 |
| 10k | rudb-native | q25 | 0.001267 | 0.001191 | 0.000041 | 0.006768 | 0.005836 | 10.36 |
| 10k | rudb-native | q26 | 0.001155 | 0.001144 | 0.000188 | 0.006770 | 0.005810 | 10.31 |
| 10k | rudb-native | q27 | 0.001164 | 0.001246 | 0.000086 | 0.006980 | 0.005922 | 10.69 |
| 10k | rudb-native | q28 | 0.001625 | 0.001499 | 0.000204 | 0.007163 | 0.006416 | 12.55 |
| 10k | rudb-native | q29 | 0.002573 | 0.002604 | 0.000489 | 0.008706 | 0.008830 | 14.67 |
| 10k | rudb-native | q30 | 0.003077 | 0.002994 | 0.000152 | 0.008983 | 0.007872 | 10.86 |
| 10k | rudb-native | q31 | 0.001806 | 0.001724 | 0.000182 | 0.007617 | 0.007085 | 12.50 |
| 10k | rudb-native | q32 | 0.001464 | 0.001177 | 0.000114 | 0.006880 | 0.005871 | 10.34 |
| 10k | rudb-native | q33 | 0.000880 | 0.000944 | 0.000076 | 0.006724 | 0.005709 | 10.03 |
| 10k | rudb-native | q34 | 0.000924 | 0.000961 | 0.000147 | 0.007038 | 0.005864 | 10.09 |
| 10k | rudb-native | q35 | 0.001650 | 0.001701 | 0.000168 | 0.007990 | 0.007030 | 11.95 |
| 10k | rudb-native | q36 | 0.000953 | 0.001020 | 0.000083 | 0.006877 | 0.005832 | 10.27 |
| 10k | rudb-native | q37 | 0.001491 | 0.001583 | 0.000119 | 0.007440 | 0.006712 | 12.03 |
| 10k | rudb-native | q38 | 0.001807 | 0.001886 | 0.000222 | 0.008868 | 0.007757 | 13.11 |
| 10k | rudb-native | q39 | 0.001363 | 0.001335 | 0.000231 | 0.007995 | 0.007078 | 12.62 |
| 10k | rudb-native | q40 | 0.001394 | 0.001510 | 0.000254 | 0.008404 | 0.007346 | 12.89 |
| 10k | rudb-native | q41 | 0.001533 | 0.001435 | 0.000330 | 0.008315 | 0.007126 | 12.59 |
| 10k | rudb-native | q42 | 0.001490 | 0.001409 | 0.000165 | 0.008294 | 0.007086 | 12.58 |
| 10k | rudb-native | q43 | 0.001443 | 0.001450 | 0.000253 | 0.008298 | 0.007189 | 12.55 |
| 10k | duckdb-parquet | q1 | 0.001000 | 0.000000 | 0.001000 | 0.018597 | 0.015769 | 18.88 |
| 10k | duckdb-parquet | q2 | 0.001000 | 0.001000 | 0.001000 | 0.016713 | 0.015973 | 19.34 |
| 10k | duckdb-parquet | q3 | 0.002000 | 0.001000 | 0.000000 | 0.018772 | 0.017899 | 20.23 |
| 10k | duckdb-parquet | q4 | 0.001000 | 0.001000 | 0.000000 | 0.019005 | 0.018050 | 19.03 |
| 10k | duckdb-parquet | q5 | 0.001000 | 0.002000 | 0.000000 | 0.020909 | 0.020826 | 24.34 |
| 10k | duckdb-parquet | q6 | 0.002000 | 0.002000 | 0.001000 | 0.019998 | 0.020153 | 23.92 |
| 10k | duckdb-parquet | q7 | 0.001000 | 0.001000 | 0.000000 | 0.018964 | 0.018236 | 18.42 |
| 10k | duckdb-parquet | q8 | 0.001000 | 0.001000 | 0.001000 | 0.019858 | 0.019020 | 20.38 |
| 10k | duckdb-parquet | q9 | 0.003000 | 0.002000 | 0.001000 | 0.023401 | 0.025648 | 30.91 |
| 10k | duckdb-parquet | q10 | 0.002000 | 0.003000 | 0.001000 | 0.021784 | 0.024158 | 31.89 |
| 10k | duckdb-parquet | q11 | 0.002000 | 0.002000 | 0.000000 | 0.020558 | 0.022244 | 30.20 |
| 10k | duckdb-parquet | q12 | 0.002000 | 0.002000 | 0.001000 | 0.021259 | 0.022907 | 32.47 |
| 10k | duckdb-parquet | q13 | 0.002000 | 0.002000 | 0.000000 | 0.021506 | 0.021847 | 25.55 |
| 10k | duckdb-parquet | q14 | 0.002000 | 0.002000 | 0.001000 | 0.021096 | 0.023576 | 31.95 |
| 10k | duckdb-parquet | q15 | 0.002000 | 0.002000 | 0.001000 | 0.021494 | 0.022888 | 25.80 |
| 10k | duckdb-parquet | q16 | 0.002000 | 0.002000 | 0.001000 | 0.020562 | 0.021750 | 25.50 |
| 10k | duckdb-parquet | q17 | 0.003000 | 0.003000 | 0.001000 | 0.021068 | 0.022576 | 27.53 |
| 10k | duckdb-parquet | q18 | 0.003000 | 0.002000 | 0.001000 | 0.021681 | 0.021804 | 27.44 |
| 10k | duckdb-parquet | q19 | 0.003000 | 0.003000 | 0.000000 | 0.021510 | 0.023425 | 28.52 |
| 10k | duckdb-parquet | q20 | 0.001000 | 0.001000 | 0.000000 | 0.019048 | 0.018277 | 18.62 |
| 10k | duckdb-parquet | q21 | 0.002000 | 0.002000 | 0.001000 | 0.021321 | 0.020232 | 20.30 |
| 10k | duckdb-parquet | q22 | 0.002000 | 0.002000 | 0.000000 | 0.020836 | 0.020781 | 21.22 |
| 10k | duckdb-parquet | q23 | 0.004000 | 0.004000 | 0.001000 | 0.022304 | 0.024667 | 28.72 |
| 10k | duckdb-parquet | q24 | 0.008000 | 0.009000 | 0.002000 | 0.028710 | 0.029909 | 37.09 |
| 10k | duckdb-parquet | q25 | 0.003000 | 0.003000 | 0.001000 | 0.021704 | 0.021825 | 22.16 |
| 10k | duckdb-parquet | q26 | 0.001000 | 0.001000 | 0.001000 | 0.019431 | 0.018358 | 20.69 |
| 10k | duckdb-parquet | q27 | 0.001000 | 0.002000 | 0.001000 | 0.019862 | 0.018981 | 20.30 |
| 10k | duckdb-parquet | q28 | 0.003000 | 0.003000 | 0.001000 | 0.020905 | 0.021540 | 26.02 |
| 10k | duckdb-parquet | q29 | 0.009000 | 0.009000 | 0.003000 | 0.030261 | 0.037044 | 29.39 |
| 10k | duckdb-parquet | q30 | 0.005000 | 0.005000 | 0.000000 | 0.024710 | 0.023040 | 29.47 |
| 10k | duckdb-parquet | q31 | 0.003000 | 0.002000 | 0.001000 | 0.021550 | 0.022588 | 27.05 |
| 10k | duckdb-parquet | q32 | 0.003000 | 0.003000 | 0.001000 | 0.021590 | 0.022465 | 27.59 |
| 10k | duckdb-parquet | q33 | 0.003000 | 0.003000 | 0.000000 | 0.021687 | 0.023640 | 28.41 |
| 10k | duckdb-parquet | q34 | 0.004000 | 0.003000 | 0.001000 | 0.022588 | 0.025094 | 28.39 |
| 10k | duckdb-parquet | q35 | 0.004000 | 0.004000 | 0.001000 | 0.023966 | 0.026565 | 29.84 |
| 10k | duckdb-parquet | q36 | 0.003000 | 0.002000 | 0.001000 | 0.021677 | 0.023170 | 27.05 |
| 10k | duckdb-parquet | q37 | 0.003000 | 0.003000 | 0.000000 | 0.022018 | 0.023493 | 26.80 |
| 10k | duckdb-parquet | q38 | 0.002000 | 0.003000 | 0.001000 | 0.024954 | 0.024931 | 27.28 |
| 10k | duckdb-parquet | q39 | 0.003000 | 0.003000 | 0.001000 | 0.025538 | 0.025738 | 27.56 |
| 10k | duckdb-parquet | q40 | 0.004000 | 0.004000 | 0.001000 | 0.026650 | 0.026902 | 31.06 |
| 10k | duckdb-parquet | q41 | 0.003000 | 0.002000 | 0.001000 | 0.025836 | 0.024685 | 27.23 |
| 10k | duckdb-parquet | q42 | 0.003000 | 0.002000 | 0.001000 | 0.024769 | 0.024764 | 25.38 |
| 10k | duckdb-parquet | q43 | 0.002000 | 0.003000 | 0.001000 | 0.025924 | 0.025228 | 26.19 |
| 10k | rudb-parquet | q1 | 0.000729 | 0.000838 | 0.000171 | 0.005745 | 0.004746 | 10.03 |
| 10k | rudb-parquet | q2 | 0.001539 | 0.000927 | 0.000104 | 0.005897 | 0.004832 | 10.25 |
| 10k | rudb-parquet | q3 | 0.000954 | 0.000941 | 0.000274 | 0.005850 | 0.004907 | 10.23 |
| 10k | rudb-parquet | q4 | 0.001345 | 0.000933 | 0.000079 | 0.006028 | 0.005117 | 9.91 |
| 10k | rudb-parquet | q5 | 0.001023 | 0.001030 | 0.000166 | 0.006416 | 0.005578 | 10.50 |
| 10k | rudb-parquet | q6 | 0.001423 | 0.001426 | 0.000150 | 0.006644 | 0.005619 | 10.56 |
| 10k | rudb-parquet | q7 | 0.000664 | 0.000690 | 0.000033 | 0.006152 | 0.005153 | 8.58 |
| 10k | rudb-parquet | q8 | 0.001169 | 0.001120 | 0.000050 | 0.006352 | 0.005408 | 10.48 |
| 10k | rudb-parquet | q9 | 0.001282 | 0.001357 | 0.000968 | 0.007026 | 0.006000 | 11.89 |
| 10k | rudb-parquet | q10 | 0.001916 | 0.001839 | 0.000169 | 0.007285 | 0.006218 | 11.34 |
| 10k | rudb-parquet | q11 | 0.001428 | 0.001252 | 0.000142 | 0.006405 | 0.005482 | 10.56 |
| 10k | rudb-parquet | q12 | 0.001344 | 0.001311 | 0.000092 | 0.006665 | 0.005672 | 11.16 |
| 10k | rudb-parquet | q13 | 0.001366 | 0.001545 | 0.000134 | 0.007201 | 0.006078 | 10.70 |
| 10k | rudb-parquet | q14 | 0.001562 | 0.001576 | 0.000083 | 0.006776 | 0.005856 | 10.92 |
| 10k | rudb-parquet | q15 | 0.001513 | 0.001527 | 0.000258 | 0.006906 | 0.005958 | 10.73 |
| 10k | rudb-parquet | q16 | 0.001262 | 0.001227 | 0.000057 | 0.006391 | 0.005533 | 10.41 |
| 10k | rudb-parquet | q17 | 0.002131 | 0.002178 | 0.000173 | 0.007595 | 0.006625 | 11.88 |
| 10k | rudb-parquet | q18 | 0.001959 | 0.001511 | 0.000124 | 0.007010 | 0.006048 | 11.34 |
| 10k | rudb-parquet | q19 | 0.002620 | 0.002451 | 0.000157 | 0.008091 | 0.007072 | 12.17 |
| 10k | rudb-parquet | q20 | 0.000847 | 0.000850 | 0.000060 | 0.006267 | 0.005441 | 9.53 |
| 10k | rudb-parquet | q21 | 0.002682 | 0.002525 | 0.000093 | 0.007731 | 0.006721 | 12.36 |
| 10k | rudb-parquet | q22 | 0.002745 | 0.002877 | 0.000164 | 0.008348 | 0.007335 | 12.70 |
| 10k | rudb-parquet | q23 | 0.004881 | 0.005200 | 0.000340 | 0.010510 | 0.009410 | 14.75 |
| 10k | rudb-parquet | q24 | 0.002826 | 0.002790 | 0.000195 | 0.008380 | 0.007326 | 12.48 |
| 10k | rudb-parquet | q25 | 0.001261 | 0.001388 | 0.000219 | 0.006892 | 0.005791 | 10.31 |
| 10k | rudb-parquet | q26 | 0.001512 | 0.001338 | 0.000170 | 0.006728 | 0.005933 | 9.98 |
| 10k | rudb-parquet | q27 | 0.001402 | 0.001421 | 0.000104 | 0.007165 | 0.006030 | 10.02 |
| 10k | rudb-parquet | q28 | 0.002624 | 0.002541 | 0.000072 | 0.008033 | 0.006930 | 12.41 |
| 10k | rudb-parquet | q29 | 0.003834 | 0.003861 | 0.000200 | 0.009229 | 0.008104 | 12.92 |
| 10k | rudb-parquet | q30 | 0.003446 | 0.003404 | 0.000144 | 0.008857 | 0.007819 | 11.14 |
| 10k | rudb-parquet | q31 | 0.001851 | 0.001890 | 0.000108 | 0.007397 | 0.006378 | 10.80 |
| 10k | rudb-parquet | q32 | 0.001933 | 0.001852 | 0.000207 | 0.007416 | 0.006331 | 11.11 |
| 10k | rudb-parquet | q33 | 0.001610 | 0.001566 | 0.000109 | 0.006929 | 0.006013 | 11.14 |
| 10k | rudb-parquet | q34 | 0.003576 | 0.003390 | 0.000254 | 0.009102 | 0.008007 | 14.03 |
| 10k | rudb-parquet | q35 | 0.003318 | 0.003447 | 0.000149 | 0.009102 | 0.008012 | 13.03 |
| 10k | rudb-parquet | q36 | 0.001511 | 0.001461 | 0.000111 | 0.006986 | 0.006081 | 11.27 |
| 10k | rudb-parquet | q37 | 0.002741 | 0.002677 | 0.000086 | 0.008320 | 0.007318 | 12.86 |
| 10k | rudb-parquet | q38 | 0.003741 | 0.004017 | 0.000497 | 0.010535 | 0.009049 | 13.33 |
| 10k | rudb-parquet | q39 | 0.002784 | 0.003169 | 0.000642 | 0.009523 | 0.008184 | 13.75 |
| 10k | rudb-parquet | q40 | 0.004129 | 0.004247 | 0.000502 | 0.010694 | 0.009401 | 14.72 |
| 10k | rudb-parquet | q41 | 0.001608 | 0.001583 | 0.000049 | 0.007445 | 0.006563 | 11.80 |
| 10k | rudb-parquet | q42 | 0.001491 | 0.001510 | 0.000094 | 0.007434 | 0.006317 | 11.86 |
| 10k | rudb-parquet | q43 | 0.001795 | 0.001558 | 0.000252 | 0.007791 | 0.006655 | 11.83 |
| 1m | duckdb-native | q1 | 0.000000 | 0.001000 | 0.001000 | 0.020606 | 0.019539 | 19.56 |
| 1m | duckdb-native | q2 | 0.001000 | 0.001000 | 0.000000 | 0.021399 | 0.021749 | 23.38 |
| 1m | duckdb-native | q3 | 0.003000 | 0.002000 | 0.001000 | 0.021799 | 0.025030 | 25.81 |
| 1m | duckdb-native | q4 | 0.003000 | 0.002000 | 0.000000 | 0.021730 | 0.025093 | 29.70 |
| 1m | duckdb-native | q5 | 0.012000 | 0.013000 | 0.002000 | 0.034830 | 0.082690 | 65.02 |
| 1m | duckdb-native | q6 | 0.011000 | 0.009000 | 0.001000 | 0.031745 | 0.064600 | 53.39 |
| 1m | duckdb-native | q7 | 0.001000 | 0.001000 | 0.001000 | 0.020420 | 0.019450 | 19.00 |
| 1m | duckdb-native | q8 | 0.001000 | 0.001000 | 0.000000 | 0.022197 | 0.023398 | 24.66 |
| 1m | duckdb-native | q9 | 0.020000 | 0.016000 | 0.001000 | 0.039780 | 0.103719 | 76.69 |
| 1m | duckdb-native | q10 | 0.022000 | 0.025000 | 0.005000 | 0.048772 | 0.141167 | 86.22 |
| 1m | duckdb-native | q11 | 0.006000 | 0.005000 | 0.001000 | 0.027678 | 0.042113 | 49.30 |
| 1m | duckdb-native | q12 | 0.006000 | 0.006000 | 0.006000 | 0.028447 | 0.045395 | 53.38 |
| 1m | duckdb-native | q13 | 0.011000 | 0.010000 | 0.001000 | 0.032935 | 0.061040 | 56.16 |
| 1m | duckdb-native | q14 | 0.013000 | 0.014000 | 0.003000 | 0.037489 | 0.083762 | 84.08 |
| 1m | duckdb-native | q15 | 0.009000 | 0.011000 | 0.001000 | 0.034182 | 0.066239 | 60.20 |
| 1m | duckdb-native | q16 | 0.015000 | 0.016000 | 0.001000 | 0.039975 | 0.099006 | 76.58 |
| 1m | duckdb-native | q17 | 0.032000 | 0.034000 | 0.003000 | 0.061957 | 0.190899 | 157.02 |
| 1m | duckdb-native | q18 | 0.026000 | 0.028000 | 0.007000 | 0.057068 | 0.132733 | 136.41 |
| 1m | duckdb-native | q19 | 0.042000 | 0.039000 | 0.006000 | 0.068215 | 0.213934 | 174.08 |
| 1m | duckdb-native | q20 | 0.001000 | 0.002000 | 0.001000 | 0.023257 | 0.025575 | 28.44 |
| 1m | duckdb-native | q21 | 0.035000 | 0.023000 | 0.002000 | 0.050879 | 0.134390 | 92.05 |
| 1m | duckdb-native | q22 | 0.018000 | 0.017000 | 0.002000 | 0.042708 | 0.106262 | 106.44 |
| 1m | duckdb-native | q23 | 0.032000 | 0.020000 | 0.001000 | 0.048532 | 0.122041 | 120.59 |
| 1m | duckdb-native | q24 | 0.051000 | 0.043000 | 0.005000 | 0.076187 | 0.207349 | 227.45 |
| 1m | duckdb-native | q25 | 0.006000 | 0.005000 | 0.001000 | 0.028807 | 0.042193 | 36.92 |
| 1m | duckdb-native | q26 | 0.005000 | 0.005000 | 0.000000 | 0.026933 | 0.040814 | 30.34 |
| 1m | duckdb-native | q27 | 0.004000 | 0.004000 | 0.001000 | 0.025463 | 0.038760 | 33.77 |
| 1m | duckdb-native | q28 | 0.021000 | 0.017000 | 0.001000 | 0.041952 | 0.106338 | 101.38 |
| 1m | duckdb-native | q29 | 0.249000 | 0.232000 | 0.005000 | 0.261227 | 1.137507 | 138.77 |
| 1m | duckdb-native | q30 | 0.006000 | 0.007000 | 0.001000 | 0.030772 | 0.033461 | 33.88 |
| 1m | duckdb-native | q31 | 0.012000 | 0.010000 | 0.001000 | 0.032134 | 0.064132 | 64.33 |
| 1m | duckdb-native | q32 | 0.011000 | 0.011000 | 0.001000 | 0.034155 | 0.067778 | 72.25 |
| 1m | duckdb-native | q33 | 0.036000 | 0.032000 | 0.005000 | 0.060892 | 0.176617 | 133.36 |
| 1m | duckdb-native | q34 | 0.045000 | 0.045000 | 0.004000 | 0.076085 | 0.244122 | 262.48 |
| 1m | duckdb-native | q35 | 0.058000 | 0.053000 | 0.002000 | 0.088255 | 0.277183 | 271.95 |
| 1m | duckdb-native | q36 | 0.019000 | 0.021000 | 0.002000 | 0.045000 | 0.123006 | 77.28 |
| 1m | duckdb-native | q37 | 0.003000 | 0.004000 | 0.001000 | 0.025923 | 0.029330 | 32.48 |
| 1m | duckdb-native | q38 | 0.003000 | 0.003000 | 0.000000 | 0.024608 | 0.027848 | 30.89 |
| 1m | duckdb-native | q39 | 0.003000 | 0.002000 | 0.000000 | 0.023481 | 0.026270 | 30.28 |
| 1m | duckdb-native | q40 | 0.005000 | 0.005000 | 0.000000 | 0.026461 | 0.030982 | 39.23 |
| 1m | duckdb-native | q41 | 0.003000 | 0.003000 | 0.001000 | 0.024234 | 0.026364 | 30.16 |
| 1m | duckdb-native | q42 | 0.003000 | 0.002000 | 0.001000 | 0.024541 | 0.026808 | 30.25 |
| 1m | duckdb-native | q43 | 0.002000 | 0.002000 | 0.001000 | 0.023492 | 0.025995 | 28.70 |
| 1m | rudb-native | q1 | 0.000868 | 0.000677 | 0.000049 | 0.006752 | 0.005681 | 9.47 |
| 1m | rudb-native | q2 | 0.000930 | 0.000787 | 0.000200 | 0.006758 | 0.005934 | 9.67 |
| 1m | rudb-native | q3 | 0.000912 | 0.000799 | 0.000195 | 0.006772 | 0.005753 | 9.48 |
| 1m | rudb-native | q4 | 0.000699 | 0.000732 | 0.000099 | 0.006788 | 0.005725 | 9.36 |
| 1m | rudb-native | q5 | 0.000667 | 0.000743 | 0.000149 | 0.007159 | 0.006087 | 9.36 |
| 1m | rudb-native | q6 | 0.000807 | 0.000763 | 0.000081 | 0.006986 | 0.005920 | 9.48 |
| 1m | rudb-native | q7 | 0.000781 | 0.000718 | 0.000033 | 0.006446 | 0.005538 | 9.25 |
| 1m | rudb-native | q8 | 0.001538 | 0.001548 | 0.000130 | 0.008395 | 0.009304 | 15.48 |
| 1m | rudb-native | q9 | 0.006583 | 0.006767 | 0.000234 | 0.014135 | 0.036463 | 47.69 |
| 1m | rudb-native | q10 | 0.009565 | 0.009312 | 0.003703 | 0.017585 | 0.049685 | 52.67 |
| 1m | rudb-native | q11 | 0.002922 | 0.002864 | 0.000312 | 0.009906 | 0.014924 | 25.27 |
| 1m | rudb-native | q12 | 0.002935 | 0.003098 | 0.000185 | 0.010110 | 0.015827 | 26.03 |
| 1m | rudb-native | q13 | 0.003079 | 0.002937 | 0.000120 | 0.010030 | 0.012890 | 17.92 |
| 1m | rudb-native | q14 | 0.004893 | 0.005163 | 0.000537 | 0.012617 | 0.026331 | 32.42 |
| 1m | rudb-native | q15 | 0.005031 | 0.005089 | 0.000525 | 0.012718 | 0.025438 | 27.11 |
| 1m | rudb-native | q16 | 0.000973 | 0.000968 | 0.000078 | 0.007594 | 0.006558 | 9.98 |
| 1m | rudb-native | q17 | 0.008141 | 0.008927 | 0.000230 | 0.017220 | 0.048092 | 52.28 |
| 1m | rudb-native | q18 | 0.001964 | 0.001976 | 0.000136 | 0.008904 | 0.007753 | 12.95 |
| 1m | rudb-native | q19 | 0.010627 | 0.011438 | 0.001453 | 0.019479 | 0.058161 | 56.75 |
| 1m | rudb-native | q20 | 0.001165 | 0.001183 | 0.000059 | 0.008099 | 0.008445 | 13.12 |
| 1m | rudb-native | q21 | 0.020697 | 0.023894 | 0.003576 | 0.031776 | 0.072279 | 47.16 |
| 1m | rudb-native | q22 | 0.026883 | 0.025418 | 0.001877 | 0.034516 | 0.077074 | 56.92 |
| 1m | rudb-native | q23 | 0.025821 | 0.027199 | 0.000517 | 0.035683 | 0.061596 | 47.44 |
| 1m | rudb-native | q24 | 0.027627 | 0.028198 | 0.000752 | 0.037076 | 0.093384 | 69.75 |
| 1m | rudb-native | q25 | 0.002800 | 0.002566 | 0.000387 | 0.009784 | 0.012345 | 16.78 |
| 1m | rudb-native | q26 | 0.003640 | 0.003692 | 0.000340 | 0.010694 | 0.016075 | 19.12 |
| 1m | rudb-native | q27 | 0.002502 | 0.002584 | 0.000190 | 0.009377 | 0.011405 | 17.05 |
| 1m | rudb-native | q28 | 0.005168 | 0.005034 | 0.000247 | 0.012103 | 0.020306 | 31.20 |
| 1m | rudb-native | q29 | 0.050692 | 0.061050 | 0.005526 | 0.071281 | 0.251572 | 82.62 |
| 1m | rudb-native | q30 | 0.003616 | 0.003591 | 0.000359 | 0.010680 | 0.009343 | 11.22 |
| 1m | rudb-native | q31 | 0.004384 | 0.004417 | 0.000412 | 0.011538 | 0.021930 | 30.80 |
| 1m | rudb-native | q32 | 0.001405 | 0.001301 | 0.000134 | 0.007615 | 0.006515 | 11.12 |
| 1m | rudb-native | q33 | 0.001120 | 0.001134 | 0.000135 | 0.008331 | 0.007117 | 11.09 |
| 1m | rudb-native | q34 | 0.001347 | 0.001158 | 0.000110 | 0.008272 | 0.007019 | 10.80 |
| 1m | rudb-native | q35 | 0.004317 | 0.005028 | 0.000577 | 0.014075 | 0.019769 | 24.94 |
| 1m | rudb-native | q36 | 0.001190 | 0.001149 | 0.000081 | 0.008335 | 0.007096 | 11.25 |
| 1m | rudb-native | q37 | 0.002867 | 0.002586 | 0.000101 | 0.009325 | 0.009635 | 16.03 |
| 1m | rudb-native | q38 | 0.002326 | 0.002545 | 0.000327 | 0.009344 | 0.009504 | 15.89 |
| 1m | rudb-native | q39 | 0.001667 | 0.001872 | 0.000166 | 0.008609 | 0.008952 | 15.64 |
| 1m | rudb-native | q40 | 0.004903 | 0.004110 | 0.000269 | 0.010793 | 0.011805 | 19.86 |
| 1m | rudb-native | q41 | 0.001686 | 0.001843 | 0.000461 | 0.009047 | 0.009172 | 15.12 |
| 1m | rudb-native | q42 | 0.001664 | 0.001824 | 0.000189 | 0.008703 | 0.008844 | 14.52 |
| 1m | rudb-native | q43 | 0.001916 | 0.002008 | 0.000106 | 0.008447 | 0.008931 | 15.12 |
| 1m | duckdb-parquet | q1 | 0.002000 | 0.002000 | 0.000000 | 0.021993 | 0.020997 | 19.44 |
| 1m | duckdb-parquet | q2 | 0.003000 | 0.003000 | 0.000000 | 0.023136 | 0.025246 | 21.72 |
| 1m | duckdb-parquet | q3 | 0.003000 | 0.004000 | 0.001000 | 0.024155 | 0.028778 | 23.86 |
| 1m | duckdb-parquet | q4 | 0.003000 | 0.004000 | 0.000000 | 0.023672 | 0.027476 | 34.88 |
| 1m | duckdb-parquet | q5 | 0.013000 | 0.014000 | 0.001000 | 0.035862 | 0.082763 | 68.83 |
| 1m | duckdb-parquet | q6 | 0.011000 | 0.012000 | 0.001000 | 0.034450 | 0.068456 | 58.05 |
| 1m | duckdb-parquet | q7 | 0.002000 | 0.002000 | 0.000000 | 0.022316 | 0.021288 | 19.39 |
| 1m | duckdb-parquet | q8 | 0.004000 | 0.003000 | 0.000000 | 0.023612 | 0.026234 | 23.50 |
| 1m | duckdb-parquet | q9 | 0.019000 | 0.019000 | 0.002000 | 0.042610 | 0.108340 | 79.02 |
| 1m | duckdb-parquet | q10 | 0.023000 | 0.024000 | 0.003000 | 0.046602 | 0.126384 | 84.81 |
| 1m | duckdb-parquet | q11 | 0.007000 | 0.006000 | 0.002000 | 0.030088 | 0.045021 | 53.22 |
| 1m | duckdb-parquet | q12 | 0.008000 | 0.008000 | 0.005000 | 0.029414 | 0.047745 | 54.36 |
| 1m | duckdb-parquet | q13 | 0.014000 | 0.013000 | 0.002000 | 0.036344 | 0.074577 | 61.88 |
| 1m | duckdb-parquet | q14 | 0.018000 | 0.018000 | 0.002000 | 0.041734 | 0.098385 | 87.56 |
| 1m | duckdb-parquet | q15 | 0.013000 | 0.014000 | 0.001000 | 0.038365 | 0.079512 | 66.41 |
| 1m | duckdb-parquet | q16 | 0.017000 | 0.018000 | 0.003000 | 0.040478 | 0.098862 | 78.36 |
| 1m | duckdb-parquet | q17 | 0.037000 | 0.038000 | 0.005000 | 0.065401 | 0.199711 | 158.53 |
| 1m | duckdb-parquet | q18 | 0.033000 | 0.032000 | 0.002000 | 0.057439 | 0.150479 | 138.97 |
| 1m | duckdb-parquet | q19 | 0.048000 | 0.049000 | 0.010000 | 0.076267 | 0.238907 | 175.98 |
| 1m | duckdb-parquet | q20 | 0.004000 | 0.004000 | 0.001000 | 0.026566 | 0.030113 | 35.41 |
| 1m | duckdb-parquet | q21 | 0.035000 | 0.037000 | 0.002000 | 0.061715 | 0.177159 | 141.58 |
| 1m | duckdb-parquet | q22 | 0.037000 | 0.032000 | 0.005000 | 0.056992 | 0.157294 | 159.66 |
| 1m | duckdb-parquet | q23 | 0.066000 | 0.062000 | 0.006000 | 0.090173 | 0.299223 | 278.80 |
| 1m | duckdb-parquet | q24 | 0.117000 | 0.116000 | 0.011000 | 0.148433 | 0.483720 | 405.28 |
| 1m | duckdb-parquet | q25 | 0.019000 | 0.017000 | 0.001000 | 0.040603 | 0.090868 | 58.55 |
| 1m | duckdb-parquet | q26 | 0.008000 | 0.009000 | 0.001000 | 0.031630 | 0.057017 | 38.02 |
| 1m | duckdb-parquet | q27 | 0.012000 | 0.012000 | 0.004000 | 0.033826 | 0.068519 | 48.95 |
| 1m | duckdb-parquet | q28 | 0.032000 | 0.030000 | 0.001000 | 0.053678 | 0.140687 | 159.17 |
| 1m | duckdb-parquet | q29 | 0.244000 | 0.242000 | 0.008000 | 0.270489 | 1.166541 | 185.92 |
| 1m | duckdb-parquet | q30 | 0.011000 | 0.009000 | 0.002000 | 0.035362 | 0.037632 | 33.80 |
| 1m | duckdb-parquet | q31 | 0.015000 | 0.015000 | 0.002000 | 0.037703 | 0.078091 | 65.66 |
| 1m | duckdb-parquet | q32 | 0.015000 | 0.014000 | 0.001000 | 0.037381 | 0.077919 | 71.75 |
| 1m | duckdb-parquet | q33 | 0.041000 | 0.035000 | 0.005000 | 0.060938 | 0.178254 | 130.30 |
| 1m | duckdb-parquet | q34 | 0.070000 | 0.063000 | 0.008000 | 0.091961 | 0.300617 | 275.44 |
| 1m | duckdb-parquet | q35 | 0.084000 | 0.073000 | 0.009000 | 0.103859 | 0.327124 | 276.02 |
| 1m | duckdb-parquet | q36 | 0.023000 | 0.022000 | 0.002000 | 0.048028 | 0.122924 | 79.42 |
| 1m | duckdb-parquet | q37 | 0.017000 | 0.017000 | 0.004000 | 0.040747 | 0.052340 | 67.61 |
| 1m | duckdb-parquet | q38 | 0.014000 | 0.015000 | 0.003000 | 0.040734 | 0.056661 | 63.16 |
| 1m | duckdb-parquet | q39 | 0.016000 | 0.015000 | 0.002000 | 0.039542 | 0.058814 | 66.64 |
| 1m | duckdb-parquet | q40 | 0.023000 | 0.024000 | 0.001000 | 0.046444 | 0.083335 | 97.23 |
| 1m | duckdb-parquet | q41 | 0.006000 | 0.006000 | 0.002000 | 0.029538 | 0.034794 | 36.48 |
| 1m | duckdb-parquet | q42 | 0.005000 | 0.004000 | 0.001000 | 0.026124 | 0.029859 | 31.52 |
| 1m | duckdb-parquet | q43 | 0.005000 | 0.005000 | 0.001000 | 0.025470 | 0.030294 | 30.62 |
| 1m | rudb-parquet | q1 | 0.002455 | 0.001948 | 0.000266 | 0.007791 | 0.007994 | 12.16 |
| 1m | rudb-parquet | q2 | 0.002927 | 0.002777 | 0.000169 | 0.008636 | 0.012268 | 17.17 |
| 1m | rudb-parquet | q3 | 0.003348 | 0.003160 | 0.000450 | 0.008984 | 0.013577 | 21.42 |
| 1m | rudb-parquet | q4 | 0.003127 | 0.003124 | 0.000187 | 0.009206 | 0.014229 | 33.36 |
| 1m | rudb-parquet | q5 | 0.008089 | 0.008624 | 0.000620 | 0.015259 | 0.042193 | 45.02 |
| 1m | rudb-parquet | q6 | 0.020915 | 0.018217 | 0.001060 | 0.025762 | 0.081224 | 52.69 |
| 1m | rudb-parquet | q7 | 0.001158 | 0.001069 | 0.000081 | 0.006395 | 0.005612 | 9.38 |
| 1m | rudb-parquet | q8 | 0.002758 | 0.003068 | 0.000411 | 0.008984 | 0.012313 | 17.44 |
| 1m | rudb-parquet | q9 | 0.008694 | 0.008508 | 0.000369 | 0.015551 | 0.040898 | 61.34 |
| 1m | rudb-parquet | q10 | 0.013154 | 0.013239 | 0.001131 | 0.021117 | 0.062554 | 67.62 |
| 1m | rudb-parquet | q11 | 0.008219 | 0.007754 | 0.000770 | 0.015266 | 0.032790 | 42.25 |
| 1m | rudb-parquet | q12 | 0.007846 | 0.008275 | 0.002518 | 0.014867 | 0.034642 | 45.30 |
| 1m | rudb-parquet | q13 | 0.015888 | 0.015232 | 0.000837 | 0.022797 | 0.070733 | 50.50 |
| 1m | rudb-parquet | q14 | 0.018872 | 0.019335 | 0.003103 | 0.026969 | 0.089260 | 78.94 |
| 1m | rudb-parquet | q15 | 0.017200 | 0.017061 | 0.001945 | 0.024394 | 0.076764 | 55.03 |
| 1m | rudb-parquet | q16 | 0.006842 | 0.006727 | 0.000352 | 0.013880 | 0.031288 | 48.83 |
| 1m | rudb-parquet | q17 | 0.044091 | 0.044578 | 0.002629 | 0.053879 | 0.190113 | 123.77 |
| 1m | rudb-parquet | q18 | 0.014754 | 0.013851 | 0.000595 | 0.021252 | 0.059080 | 50.00 |
| 1m | rudb-parquet | q19 | 0.051713 | 0.056380 | 0.001386 | 0.065574 | 0.236034 | 153.48 |
| 1m | rudb-parquet | q20 | 0.003255 | 0.003217 | 0.000324 | 0.009635 | 0.014040 | 31.16 |
| 1m | rudb-parquet | q21 | 0.051239 | 0.060116 | 0.011247 | 0.071074 | 0.230198 | 186.80 |
| 1m | rudb-parquet | q22 | 0.055166 | 0.057112 | 0.003588 | 0.067903 | 0.224650 | 179.66 |
| 1m | rudb-parquet | q23 | 0.126664 | 0.112404 | 0.006364 | 0.122916 | 0.477084 | 280.84 |
| 1m | rudb-parquet | q24 | 0.116010 | 0.120787 | 0.008657 | 0.134993 | 0.672008 | 339.66 |
| 1m | rudb-parquet | q25 | 0.015040 | 0.013938 | 0.000835 | 0.021336 | 0.058604 | 49.02 |
| 1m | rudb-parquet | q26 | 0.010305 | 0.010288 | 0.000759 | 0.016967 | 0.044654 | 31.97 |
| 1m | rudb-parquet | q27 | 0.014071 | 0.012444 | 0.001485 | 0.019505 | 0.055414 | 49.00 |
| 1m | rudb-parquet | q28 | 0.044332 | 0.043661 | 0.003828 | 0.053277 | 0.180321 | 171.05 |
| 1m | rudb-parquet | q29 | 0.107461 | 0.097943 | 0.005748 | 0.110372 | 0.407798 | 195.30 |
| 1m | rudb-parquet | q30 | 0.006955 | 0.006261 | 0.000476 | 0.013700 | 0.017105 | 18.84 |
| 1m | rudb-parquet | q31 | 0.016983 | 0.014841 | 0.001067 | 0.021697 | 0.061425 | 56.20 |
| 1m | rudb-parquet | q32 | 0.015149 | 0.015195 | 0.001575 | 0.022275 | 0.063130 | 67.20 |
| 1m | rudb-parquet | q33 | 0.014215 | 0.013026 | 0.000348 | 0.020476 | 0.062492 | 68.42 |
| 1m | rudb-parquet | q34 | 0.094364 | 0.094111 | 0.015431 | 0.108883 | 0.382992 | 273.19 |
| 1m | rudb-parquet | q35 | 0.090391 | 0.099814 | 0.014683 | 0.114797 | 0.393381 | 287.05 |
| 1m | rudb-parquet | q36 | 0.007360 | 0.006976 | 0.000196 | 0.014397 | 0.032909 | 39.61 |
| 1m | rudb-parquet | q37 | 0.034208 | 0.031006 | 0.005010 | 0.039818 | 0.058287 | 64.06 |
| 1m | rudb-parquet | q38 | 0.037756 | 0.039058 | 0.004424 | 0.045883 | 0.073835 | 59.84 |
| 1m | rudb-parquet | q39 | 0.029413 | 0.029187 | 0.001685 | 0.036260 | 0.052987 | 64.52 |
| 1m | rudb-parquet | q40 | 0.046449 | 0.048844 | 0.001324 | 0.057504 | 0.094331 | 108.77 |
| 1m | rudb-parquet | q41 | 0.004857 | 0.005782 | 0.002228 | 0.014413 | 0.016600 | 29.33 |
| 1m | rudb-parquet | q42 | 0.003884 | 0.004492 | 0.000406 | 0.011254 | 0.013128 | 26.33 |
| 1m | rudb-parquet | q43 | 0.005146 | 0.005068 | 0.001055 | 0.011925 | 0.014128 | 23.25 |
| 10m | duckdb-native | q1 | 0.003000 | 0.002000 | 0.001000 | 0.027733 | 0.026899 | 20.39 |
| 10m | duckdb-native | q2 | 0.009000 | 0.005000 | 0.001000 | 0.030604 | 0.036495 | 28.25 |
| 10m | duckdb-native | q3 | 0.016000 | 0.013000 | 0.000000 | 0.039313 | 0.082956 | 36.27 |
| 10m | duckdb-native | q4 | 0.020000 | 0.013000 | 0.001000 | 0.040923 | 0.080262 | 42.11 |
| 10m | duckdb-native | q5 | 0.056000 | 0.054000 | 0.009000 | 0.081522 | 0.308846 | 105.00 |
| 10m | duckdb-native | q6 | 0.109000 | 0.091000 | 0.009000 | 0.126621 | 0.522320 | 252.09 |
| 10m | duckdb-native | q7 | 0.004000 | 0.003000 | 0.000000 | 0.025741 | 0.024950 | 21.36 |
| 10m | duckdb-native | q8 | 0.007000 | 0.005000 | 0.001000 | 0.030771 | 0.042386 | 29.75 |
| 10m | duckdb-native | q9 | 0.072000 | 0.076000 | 0.008000 | 0.109314 | 0.405256 | 129.98 |
| 10m | duckdb-native | q10 | 0.147000 | 0.127000 | 0.019000 | 0.171539 | 0.704892 | 151.31 |
| 10m | duckdb-native | q11 | 0.023000 | 0.018000 | 0.001000 | 0.042299 | 0.115684 | 72.88 |
| 10m | duckdb-native | q12 | 0.022000 | 0.021000 | 0.001000 | 0.044098 | 0.125251 | 79.08 |
| 10m | duckdb-native | q13 | 0.075000 | 0.075000 | 0.008000 | 0.105749 | 0.439931 | 253.53 |
| 10m | duckdb-native | q14 | 0.135000 | 0.122000 | 0.015000 | 0.157177 | 0.696146 | 367.22 |
| 10m | duckdb-native | q15 | 0.098000 | 0.084000 | 0.008000 | 0.116487 | 0.477075 | 262.08 |
| 10m | duckdb-native | q16 | 0.060000 | 0.061000 | 0.004000 | 0.089406 | 0.356890 | 125.84 |
| 10m | duckdb-native | q17 | 0.146000 | 0.148000 | 0.009000 | 0.184732 | 0.838705 | 341.83 |
| 10m | duckdb-native | q18 | 0.149000 | 0.127000 | 0.014000 | 0.170033 | 0.661086 | 305.30 |
| 10m | duckdb-native | q19 | 0.438000 | 0.279000 | 0.017000 | 0.347931 | 1.578624 | 720.89 |
| 10m | duckdb-native | q20 | 0.005000 | 0.004000 | 0.000000 | 0.027227 | 0.032202 | 33.70 |
| 10m | duckdb-native | q21 | 0.274000 | 0.157000 | 0.051000 | 0.234340 | 0.854821 | 445.47 |
| 10m | duckdb-native | q22 | 0.138000 | 0.125000 | 0.020000 | 0.214038 | 0.711673 | 496.09 |
| 10m | duckdb-native | q23 | 0.337000 | 0.285000 | 0.149000 | 0.467511 | 1.301066 | 829.83 |
| 10m | duckdb-native | q24 | 0.171000 | 0.137000 | 0.015000 | 0.202581 | 0.728540 | 508.12 |
| 10m | duckdb-native | q25 | 0.009000 | 0.008000 | 0.000000 | 0.030534 | 0.050892 | 40.36 |
| 10m | duckdb-native | q26 | 0.033000 | 0.034000 | 0.002000 | 0.056816 | 0.199077 | 76.19 |
| 10m | duckdb-native | q27 | 0.009000 | 0.009000 | 0.002000 | 0.030433 | 0.051988 | 40.06 |
| 10m | duckdb-native | q28 | 0.157000 | 0.101000 | 0.021000 | 0.188265 | 0.589828 | 456.00 |
| 10m | duckdb-native | q29 | 1.898000 | 1.790000 | 0.067000 | 1.868185 | 10.110233 | 674.83 |
| 10m | duckdb-native | q30 | 0.015000 | 0.012000 | 0.001000 | 0.031269 | 0.057476 | 40.98 |
| 10m | duckdb-native | q31 | 0.063000 | 0.063000 | 0.007000 | 0.093046 | 0.373138 | 193.22 |
| 10m | duckdb-native | q32 | 0.102000 | 0.085000 | 0.014000 | 0.123754 | 0.493500 | 325.03 |
| 10m | duckdb-native | q33 | 0.399000 | 0.333000 | 0.037000 | 0.411109 | 1.826799 | 826.70 |
| 10m | duckdb-native | q34 | 0.395000 | 0.328000 | 0.022000 | 0.469386 | 1.882126 | 1115.09 |
| 10m | duckdb-native | q35 | 0.376000 | 0.385000 | 0.059000 | 0.534591 | 2.105395 | 1164.67 |
| 10m | duckdb-native | q36 | 0.074000 | 0.073000 | 0.004000 | 0.099402 | 0.427777 | 116.23 |
| 10m | duckdb-native | q37 | 0.027000 | 0.026000 | 0.002000 | 0.053584 | 0.140929 | 134.64 |
| 10m | duckdb-native | q38 | 0.009000 | 0.008000 | 0.001000 | 0.029561 | 0.048843 | 48.19 |
| 10m | duckdb-native | q39 | 0.012000 | 0.011000 | 0.002000 | 0.033454 | 0.062603 | 71.69 |
| 10m | duckdb-native | q40 | 0.061000 | 0.058000 | 0.005000 | 0.089683 | 0.290850 | 240.69 |
| 10m | duckdb-native | q41 | 0.009000 | 0.006000 | 0.001000 | 0.028864 | 0.044915 | 46.81 |
| 10m | duckdb-native | q42 | 0.008000 | 0.007000 | 0.004000 | 0.030679 | 0.045917 | 40.83 |
| 10m | duckdb-native | q43 | 0.008000 | 0.009000 | 0.002000 | 0.035394 | 0.056768 | 36.98 |
| 10m | rudb-native | q1 | 0.002064 | 0.000845 | 0.000177 | 0.010151 | 0.008962 | 11.42 |
| 10m | rudb-native | q2 | 0.001728 | 0.000937 | 0.000585 | 0.010967 | 0.009440 | 12.88 |
| 10m | rudb-native | q3 | 0.001264 | 0.000985 | 0.000077 | 0.010165 | 0.008922 | 11.59 |
| 10m | rudb-native | q4 | 0.000880 | 0.000856 | 0.000104 | 0.010316 | 0.009343 | 11.56 |
| 10m | rudb-native | q5 | 0.000824 | 0.000893 | 0.000127 | 0.010523 | 0.009363 | 11.23 |
| 10m | rudb-native | q6 | 0.000865 | 0.000927 | 0.000152 | 0.010158 | 0.008976 | 11.28 |
| 10m | rudb-native | q7 | 0.000898 | 0.000874 | 0.000061 | 0.009474 | 0.008244 | 11.39 |
| 10m | rudb-native | q8 | 0.006648 | 0.003889 | 0.000646 | 0.014676 | 0.024523 | 27.41 |
| 10m | rudb-native | q9 | 0.026500 | 0.026939 | 0.003066 | 0.040084 | 0.143861 | 73.55 |
| 10m | rudb-native | q10 | 0.038010 | 0.035188 | 0.005196 | 0.050132 | 0.187048 | 82.81 |
| 10m | rudb-native | q11 | 0.007390 | 0.007447 | 0.001023 | 0.017270 | 0.044477 | 37.19 |
| 10m | rudb-native | q12 | 0.008036 | 0.008522 | 0.000834 | 0.017762 | 0.049973 | 38.42 |
| 10m | rudb-native | q13 | 0.009205 | 0.008880 | 0.000429 | 0.019048 | 0.047194 | 37.80 |
| 10m | rudb-native | q14 | 0.025948 | 0.025325 | 0.004605 | 0.037153 | 0.140511 | 83.55 |
| 10m | rudb-native | q15 | 0.024479 | 0.020425 | 0.001801 | 0.031429 | 0.110311 | 81.62 |
| 10m | rudb-native | q16 | 0.000959 | 0.001161 | 0.000139 | 0.010544 | 0.009108 | 12.36 |
| 10m | rudb-native | q17 | 0.045624 | 0.037542 | 0.001271 | 0.050548 | 0.213346 | 125.80 |
| 10m | rudb-native | q18 | 0.005025 | 0.004453 | 0.000522 | 0.013980 | 0.012763 | 20.52 |
| 10m | rudb-native | q19 | 0.078388 | 0.072686 | 0.002599 | 0.087416 | 0.410091 | 221.73 |
| 10m | rudb-native | q20 | 0.003064 | 0.001332 | 0.000142 | 0.009618 | 0.010260 | 14.92 |
| 10m | rudb-native | q21 | 0.052530 | 0.058521 | 0.001001 | 0.070772 | 0.224509 | 81.05 |
| 10m | rudb-native | q22 | 0.074763 | 0.071744 | 0.003059 | 0.084329 | 0.242983 | 99.14 |
| 10m | rudb-native | q23 | 0.184526 | 0.185093 | 0.009296 | 0.200798 | 0.459678 | 190.17 |
| 10m | rudb-native | q24 | 0.090003 | 0.066604 | 0.004968 | 0.080393 | 0.214268 | 110.47 |
| 10m | rudb-native | q25 | 0.004368 | 0.003316 | 0.000497 | 0.011604 | 0.014444 | 20.64 |
| 10m | rudb-native | q26 | 0.022422 | 0.010856 | 0.000371 | 0.020218 | 0.046508 | 35.19 |
| 10m | rudb-native | q27 | 0.006699 | 0.006171 | 0.000643 | 0.014320 | 0.016877 | 24.38 |
| 10m | rudb-native | q28 | 0.026413 | 0.022182 | 0.003650 | 0.035033 | 0.093972 | 68.64 |
| 10m | rudb-native | q29 | 0.421961 | 0.331270 | 0.039107 | 0.347785 | 1.673031 | 279.86 |
| 10m | rudb-native | q30 | 0.003460 | 0.002972 | 0.000186 | 0.009898 | 0.008895 | 13.09 |
| 10m | rudb-native | q31 | 0.045878 | 0.025616 | 0.000880 | 0.037267 | 0.144598 | 94.86 |
| 10m | rudb-native | q32 | 0.002199 | 0.001812 | 0.000090 | 0.010215 | 0.008943 | 14.47 |
| 10m | rudb-native | q33 | 0.001262 | 0.001239 | 0.000243 | 0.010922 | 0.009654 | 12.42 |
| 10m | rudb-native | q34 | 0.001246 | 0.001352 | 0.000259 | 0.010703 | 0.009193 | 12.27 |
| 10m | rudb-native | q35 | 0.055865 | 0.024441 | 0.002652 | 0.037055 | 0.112668 | 96.28 |
| 10m | rudb-native | q36 | 0.001669 | 0.001193 | 0.000118 | 0.009730 | 0.008445 | 12.50 |
| 10m | rudb-native | q37 | 0.011760 | 0.007372 | 0.000444 | 0.016791 | 0.027929 | 41.36 |
| 10m | rudb-native | q38 | 0.008563 | 0.005950 | 0.000269 | 0.014921 | 0.024929 | 35.70 |
| 10m | rudb-native | q39 | 0.006535 | 0.006024 | 0.000545 | 0.015333 | 0.018659 | 28.77 |
| 10m | rudb-native | q40 | 0.034779 | 0.034140 | 0.005136 | 0.046731 | 0.152720 | 91.30 |
| 10m | rudb-native | q41 | 0.014330 | 0.006813 | 0.000400 | 0.015915 | 0.026045 | 40.69 |
| 10m | rudb-native | q42 | 0.005990 | 0.005361 | 0.003812 | 0.014600 | 0.022180 | 31.48 |
| 10m | rudb-native | q43 | 0.005369 | 0.005552 | 0.000961 | 0.015364 | 0.026471 | 26.69 |
| 10m | duckdb-parquet | q1 | 0.025000 | 0.021000 | 0.002000 | 0.060241 | 0.059414 | 36.08 |
| 10m | duckdb-parquet | q2 | 0.031000 | 0.029000 | 0.003000 | 0.068555 | 0.098519 | 35.22 |
| 10m | duckdb-parquet | q3 | 0.034000 | 0.034000 | 0.001000 | 0.073761 | 0.125275 | 38.25 |
| 10m | duckdb-parquet | q4 | 0.044000 | 0.034000 | 0.003000 | 0.075312 | 0.123181 | 55.53 |
| 10m | duckdb-parquet | q5 | 0.067000 | 0.061000 | 0.003000 | 0.101903 | 0.273642 | 113.73 |
| 10m | duckdb-parquet | q6 | 0.141000 | 0.122000 | 0.006000 | 0.165416 | 0.600064 | 253.41 |
| 10m | duckdb-parquet | q7 | 0.022000 | 0.020000 | 0.001000 | 0.055746 | 0.054510 | 35.97 |
| 10m | duckdb-parquet | q8 | 0.028000 | 0.034000 | 0.007000 | 0.072345 | 0.103569 | 38.70 |
| 10m | duckdb-parquet | q9 | 0.106000 | 0.110000 | 0.026000 | 0.160760 | 0.471253 | 133.00 |
| 10m | duckdb-parquet | q10 | 0.162000 | 0.147000 | 0.026000 | 0.198669 | 0.687281 | 138.78 |
| 10m | duckdb-parquet | q11 | 0.040000 | 0.036000 | 0.003000 | 0.070959 | 0.147725 | 75.16 |
| 10m | duckdb-parquet | q12 | 0.040000 | 0.039000 | 0.003000 | 0.073151 | 0.169171 | 74.69 |
| 10m | duckdb-parquet | q13 | 0.117000 | 0.126000 | 0.013000 | 0.165050 | 0.598186 | 259.94 |
| 10m | duckdb-parquet | q14 | 0.179000 | 0.165000 | 0.014000 | 0.207761 | 0.866845 | 337.42 |
| 10m | duckdb-parquet | q15 | 0.134000 | 0.128000 | 0.006000 | 0.170222 | 0.644594 | 266.03 |
| 10m | duckdb-parquet | q16 | 0.071000 | 0.071000 | 0.001000 | 0.110445 | 0.333705 | 132.44 |
| 10m | duckdb-parquet | q17 | 0.207000 | 0.204000 | 0.024000 | 0.252983 | 1.014930 | 327.88 |
| 10m | duckdb-parquet | q18 | 0.191000 | 0.179000 | 0.025000 | 0.225646 | 0.836035 | 313.42 |
| 10m | duckdb-parquet | q19 | 0.331000 | 0.346000 | 0.017000 | 0.403424 | 1.823365 | 653.92 |
| 10m | duckdb-parquet | q20 | 0.025000 | 0.022000 | 0.001000 | 0.057391 | 0.068438 | 46.62 |
| 10m | duckdb-parquet | q21 | 0.231000 | 0.225000 | 0.005000 | 0.266839 | 1.212173 | 318.09 |
| 10m | duckdb-parquet | q22 | 0.191000 | 0.196000 | 0.007000 | 0.243615 | 1.066752 | 408.38 |
| 10m | duckdb-parquet | q23 | 0.392000 | 0.437000 | 0.050000 | 0.493232 | 2.339916 | 645.97 |
| 10m | duckdb-parquet | q24 | 0.294000 | 0.286000 | 0.013000 | 0.333858 | 1.443635 | 450.48 |
| 10m | duckdb-parquet | q25 | 0.080000 | 0.070000 | 0.008000 | 0.102165 | 0.280001 | 106.42 |
| 10m | duckdb-parquet | q26 | 0.071000 | 0.073000 | 0.002000 | 0.106118 | 0.351189 | 93.08 |
| 10m | duckdb-parquet | q27 | 0.059000 | 0.060000 | 0.005000 | 0.095189 | 0.273036 | 108.41 |
| 10m | duckdb-parquet | q28 | 0.170000 | 0.177000 | 0.033000 | 0.221016 | 0.953273 | 405.25 |
| 10m | duckdb-parquet | q29 | 2.541000 | 2.212000 | 0.056000 | 2.262396 | 12.588640 | 625.39 |
| 10m | duckdb-parquet | q30 | 0.037000 | 0.025000 | 0.002000 | 0.054158 | 0.080717 | 47.02 |
| 10m | duckdb-parquet | q31 | 0.107000 | 0.112000 | 0.007000 | 0.147532 | 0.567588 | 168.03 |
| 10m | duckdb-parquet | q32 | 0.137000 | 0.133000 | 0.011000 | 0.172088 | 0.666703 | 223.25 |
| 10m | duckdb-parquet | q33 | 0.448000 | 0.358000 | 0.028000 | 0.416054 | 1.845596 | 749.61 |
| 10m | duckdb-parquet | q34 | 0.458000 | 0.439000 | 0.109000 | 0.502197 | 2.337042 | 936.48 |
| 10m | duckdb-parquet | q35 | 0.573000 | 0.461000 | 0.097000 | 0.521834 | 2.378329 | 995.81 |
| 10m | duckdb-parquet | q36 | 0.128000 | 0.091000 | 0.001000 | 0.129849 | 0.464523 | 116.08 |
| 10m | duckdb-parquet | q37 | 0.059000 | 0.053000 | 0.003000 | 0.092038 | 0.223950 | 205.08 |
| 10m | duckdb-parquet | q38 | 0.026000 | 0.027000 | 0.003000 | 0.060859 | 0.092120 | 60.34 |
| 10m | duckdb-parquet | q39 | 0.037000 | 0.035000 | 0.005000 | 0.070457 | 0.128532 | 135.52 |
| 10m | duckdb-parquet | q40 | 0.087000 | 0.095000 | 0.005000 | 0.138556 | 0.411746 | 313.47 |
| 10m | duckdb-parquet | q41 | 0.029000 | 0.025000 | 0.003000 | 0.059900 | 0.082560 | 59.36 |
| 10m | duckdb-parquet | q42 | 0.024000 | 0.027000 | 0.007000 | 0.065716 | 0.085707 | 56.97 |
| 10m | duckdb-parquet | q43 | 0.034000 | 0.031000 | 0.009000 | 0.069692 | 0.106869 | 55.52 |
| 10m | rudb-parquet | q1 | 0.017410 | 0.013971 | 0.000741 | 0.021246 | 0.024030 | 21.58 |
| 10m | rudb-parquet | q2 | 0.022689 | 0.021326 | 0.001397 | 0.029366 | 0.069106 | 29.52 |
| 10m | rudb-parquet | q3 | 0.029268 | 0.022017 | 0.001369 | 0.029825 | 0.072120 | 35.41 |
| 10m | rudb-parquet | q4 | 0.023867 | 0.024294 | 0.001023 | 0.032731 | 0.079270 | 45.92 |
| 10m | rudb-parquet | q5 | 0.071221 | 0.067752 | 0.003878 | 0.078074 | 0.311019 | 151.50 |
| 10m | rudb-parquet | q6 | 0.184386 | 0.188035 | 0.018767 | 0.199093 | 0.859677 | 209.02 |
| 10m | rudb-parquet | q7 | 0.006901 | 0.005860 | 0.000767 | 0.013083 | 0.011818 | 16.14 |
| 10m | rudb-parquet | q8 | 0.024567 | 0.021646 | 0.001061 | 0.029743 | 0.070619 | 32.92 |
| 10m | rudb-parquet | q9 | 0.048017 | 0.055913 | 0.002168 | 0.067328 | 0.231371 | 83.50 |
| 10m | rudb-parquet | q10 | 0.077832 | 0.074178 | 0.011893 | 0.088157 | 0.342862 | 94.95 |
| 10m | rudb-parquet | q11 | 0.052228 | 0.044245 | 0.002670 | 0.051842 | 0.203613 | 56.03 |
| 10m | rudb-parquet | q12 | 0.047988 | 0.047173 | 0.001344 | 0.054661 | 0.222188 | 61.00 |
| 10m | rudb-parquet | q13 | 0.127511 | 0.134982 | 0.003417 | 0.145893 | 0.677637 | 214.55 |
| 10m | rudb-parquet | q14 | 0.174857 | 0.175465 | 0.015217 | 0.188411 | 0.883245 | 296.73 |
| 10m | rudb-parquet | q15 | 0.163308 | 0.147849 | 0.003606 | 0.157519 | 0.749599 | 217.14 |
| 10m | rudb-parquet | q16 | 0.037804 | 0.034033 | 0.001238 | 0.042685 | 0.138347 | 80.45 |
| 10m | rudb-parquet | q17 | 0.279513 | 0.295994 | 0.012437 | 0.313660 | 1.383485 | 399.30 |
| 10m | rudb-parquet | q18 | 0.115524 | 0.115234 | 0.006088 | 0.124280 | 0.571094 | 132.72 |
| 10m | rudb-parquet | q19 | 0.530775 | 0.415970 | 0.017869 | 0.441441 | 2.230094 | 648.84 |
| 10m | rudb-parquet | q20 | 0.016847 | 0.015989 | 0.000973 | 0.022907 | 0.043958 | 41.64 |
| 10m | rudb-parquet | q21 | 0.278854 | 0.317290 | 0.015031 | 0.329993 | 1.734336 | 303.31 |
| 10m | rudb-parquet | q22 | 0.329715 | 0.328077 | 0.024070 | 0.344367 | 1.815267 | 356.80 |
| 10m | rudb-parquet | q23 | 0.834092 | 0.802504 | 0.010485 | 0.830038 | 4.432630 | 477.84 |
| 10m | rudb-parquet | q24 | 0.404379 | 0.402086 | 0.032847 | 0.417848 | 2.320949 | 365.95 |
| 10m | rudb-parquet | q25 | 0.114077 | 0.103473 | 0.006552 | 0.111579 | 0.528030 | 160.41 |
| 10m | rudb-parquet | q26 | 0.078513 | 0.079073 | 0.002788 | 0.086869 | 0.383352 | 136.83 |
| 10m | rudb-parquet | q27 | 0.111471 | 0.106152 | 0.008705 | 0.115311 | 0.539224 | 159.02 |
| 10m | rudb-parquet | q28 | 0.257168 | 0.232586 | 0.024598 | 0.244532 | 1.276325 | 286.78 |
| 10m | rudb-parquet | q29 | 0.694730 | 0.695631 | 0.070532 | 0.717644 | 3.742622 | 740.42 |
| 10m | rudb-parquet | q30 | 0.020261 | 0.015892 | 0.002255 | 0.021622 | 0.043871 | 27.64 |
| 10m | rudb-parquet | q31 | 0.108967 | 0.113701 | 0.007235 | 0.123548 | 0.577500 | 150.47 |
| 10m | rudb-parquet | q32 | 0.117765 | 0.117738 | 0.003752 | 0.127055 | 0.603548 | 167.08 |
| 10m | rudb-parquet | q33 | 0.110711 | 0.110331 | 0.006048 | 0.123460 | 0.564161 | 240.77 |
| 10m | rudb-parquet | q34 | 0.598831 | 0.608753 | 0.059801 | 0.642349 | 3.295828 | 804.72 |
| 10m | rudb-parquet | q35 | 0.736375 | 0.619028 | 0.075256 | 0.647437 | 3.274662 | 793.00 |
| 10m | rudb-parquet | q36 | 0.030800 | 0.032106 | 0.001917 | 0.039599 | 0.127565 | 68.98 |
| 10m | rudb-parquet | q37 | 0.063041 | 0.061637 | 0.003017 | 0.071908 | 0.238052 | 189.69 |
| 10m | rudb-parquet | q38 | 0.026827 | 0.025868 | 0.001719 | 0.032883 | 0.089758 | 60.11 |
| 10m | rudb-parquet | q39 | 0.041780 | 0.042143 | 0.007234 | 0.050695 | 0.131059 | 149.42 |
| 10m | rudb-parquet | q40 | 0.121824 | 0.133088 | 0.015965 | 0.150230 | 0.579432 | 350.23 |
| 10m | rudb-parquet | q41 | 0.020865 | 0.020450 | 0.001987 | 0.027881 | 0.053006 | 61.77 |
| 10m | rudb-parquet | q42 | 0.018008 | 0.023180 | 0.010382 | 0.031469 | 0.050456 | 56.84 |
| 10m | rudb-parquet | q43 | 0.034937 | 0.023565 | 0.005132 | 0.030967 | 0.065658 | 53.22 |
