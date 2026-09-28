# ClickBench measurement audit

All 43 SQL queries are attempted on DuckDB native, rudb native, DuckDB Parquet, and rudb Parquet. 7 hot repetitions are required for a complete row. A failed repetition invalidates that query; successful fragments are never averaged into a result.

First execution is not disk-cold: the page cache is not flushed. Each repetition uses a fresh process. Query seconds are the CLI timer, including result rendering; wall and CPU seconds and peak RSS cover the whole child process. CPU and RSS come from wait4 for that child. RSS is the maximum resident set, not allocated bytes or an incremental memory delta. Both native rows open a loaded single-file database. Both Parquet rows query the same source file with native mirroring disabled; metadata-only paths remain available. These are sample results, not official ClickBench scores.

| Size | Engine | Load wall (s) | Load CPU (s) | Load peak RSS (MiB) | Native bytes |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 0.289614 | 0.401528 | 60.80 | 1060864 |
| 1k | rudb-native | 0.260565 | 0.219097 | 38.71 | 1876635 |
| 10k | duckdb-native | 0.940978 | 1.533986 | 72.79 | 4206592 |
| 10k | rudb-native | 1.728914 | 1.918577 | 81.50 | 7732602 |
| 1m | duckdb-native | 8.897799 | 23.456667 | 1884.60 | 599273472 |
| 1m | rudb-native | 7.629432 | 20.142203 | 1052.53 | 247058959 |
| 10m | duckdb-native | 72.604220 | 148.976178 | 4258.34 | 2909024256 |
| 10m | rudb-native | 47.723841 | 173.705269 | 4789.25 | 2028733078 |

| Size | Engine | Complete / 43 | Query median sum (s) | Process wall median sum (s) | CPU median sum (s) | Peak RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 43 | 0.244000 | 2.614400 | 1.965996 | 34.31 |
| 1k | rudb-native | 43 | 0.076444 | 0.343751 | 0.237103 | 14.34 |
| 1k | duckdb-parquet | 43 | 0.375000 | 2.697640 | 1.987732 | 32.48 |
| 1k | rudb-parquet | 43 | 0.093753 | 0.364395 | 0.262621 | 13.61 |
| 10k | duckdb-native | 43 | 0.361000 | 3.059029 | 2.005169 | 36.79 |
| 10k | rudb-native | 43 | 0.165264 | 0.551537 | 0.303780 | 15.09 |
| 10k | duckdb-parquet | 43 | 0.577000 | 3.300448 | 2.335850 | 37.24 |
| 10k | rudb-parquet | 43 | 0.423041 | 0.694090 | 0.448411 | 16.29 |
| 1m | duckdb-native | 43 | 4.996000 | 8.933839 | 10.445359 | 235.23 |
| 1m | rudb-native | 43 | 1.105475 | 1.805969 | 1.635250 | 104.94 |
| 1m | duckdb-parquet | 43 | 6.664000 | 10.774208 | 14.901548 | 357.64 |
| 1m | rudb-parquet | 43 | 8.893831 | 9.837422 | 16.364106 | 294.38 |
| 10m | duckdb-native | 43 | 24.620000 | 30.094071 | 57.943101 | 1497.30 |
| 10m | rudb-native | 43 | 4.893704 | 6.021492 | 10.253081 | 276.22 |
| 10m | duckdb-parquet | 43 | 36.299000 | 55.489084 | 85.817913 | 1209.98 |
| 10m | rudb-parquet | 43 | 37.261992 | 39.477074 | 62.378416 | 1094.44 |

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
| 1k | duckdb-native | q1 | 0.000000 | 0.001000 | 0.001000 | 0.073558 | 0.053742 | 26.74 |
| 1k | duckdb-native | q2 | 0.001000 | 0.001000 | 0.008000 | 0.054760 | 0.029920 | 27.97 |
| 1k | duckdb-native | q3 | 0.001000 | 0.002000 | 0.012000 | 0.043650 | 0.030285 | 27.73 |
| 1k | duckdb-native | q4 | 0.002000 | 0.001000 | 0.000000 | 0.033957 | 0.027901 | 27.32 |
| 1k | duckdb-native | q5 | 0.002000 | 0.002000 | 0.005000 | 0.035738 | 0.026735 | 28.60 |
| 1k | duckdb-native | q6 | 0.002000 | 0.004000 | 0.009000 | 0.037923 | 0.035400 | 28.95 |
| 1k | duckdb-native | q7 | 0.000000 | 0.001000 | 0.001000 | 0.079680 | 0.072224 | 27.18 |
| 1k | duckdb-native | q8 | 0.015000 | 0.002000 | 0.004000 | 0.083034 | 0.072981 | 29.49 |
| 1k | duckdb-native | q9 | 0.003000 | 0.005000 | 0.015000 | 0.093575 | 0.073192 | 32.00 |
| 1k | duckdb-native | q10 | 0.008000 | 0.010000 | 0.005000 | 0.052178 | 0.029073 | 32.24 |
| 1k | duckdb-native | q11 | 0.016000 | 0.004000 | 0.010000 | 0.112205 | 0.079994 | 31.49 |
| 1k | duckdb-native | q12 | 0.008000 | 0.012000 | 0.023000 | 0.096263 | 0.048247 | 32.24 |
| 1k | duckdb-native | q13 | 0.005000 | 0.007000 | 0.036000 | 0.129611 | 0.077824 | 29.63 |
| 1k | duckdb-native | q14 | 0.016000 | 0.005000 | 0.023000 | 0.095640 | 0.051404 | 32.70 |
| 1k | duckdb-native | q15 | 0.017000 | 0.007000 | 0.013000 | 0.066164 | 0.042222 | 30.25 |
| 1k | duckdb-native | q16 | 0.004000 | 0.005000 | 0.004000 | 0.069217 | 0.032578 | 31.14 |
| 1k | duckdb-native | q17 | 0.008000 | 0.006000 | 0.014000 | 0.063838 | 0.031266 | 32.11 |
| 1k | duckdb-native | q18 | 0.002000 | 0.004000 | 0.009000 | 0.087267 | 0.053415 | 31.31 |
| 1k | duckdb-native | q19 | 0.009000 | 0.009000 | 0.004000 | 0.067985 | 0.036803 | 32.43 |
| 1k | duckdb-native | q20 | 0.001000 | 0.001000 | 0.002000 | 0.044628 | 0.029393 | 27.57 |
| 1k | duckdb-native | q21 | 0.003000 | 0.002000 | 0.003000 | 0.039028 | 0.023854 | 27.69 |
| 1k | duckdb-native | q22 | 0.004000 | 0.002000 | 0.002000 | 0.045089 | 0.039845 | 28.99 |
| 1k | duckdb-native | q23 | 0.002000 | 0.004000 | 0.005000 | 0.031041 | 0.026942 | 29.00 |
| 1k | duckdb-native | q24 | 0.012000 | 0.011000 | 0.004000 | 0.044521 | 0.034840 | 34.31 |
| 1k | duckdb-native | q25 | 0.003000 | 0.006000 | 0.005000 | 0.042839 | 0.032714 | 30.71 |
| 1k | duckdb-native | q26 | 0.001000 | 0.003000 | 0.007000 | 0.044177 | 0.036664 | 27.63 |
| 1k | duckdb-native | q27 | 0.002000 | 0.002000 | 0.001000 | 0.035137 | 0.031801 | 28.66 |
| 1k | duckdb-native | q28 | 0.016000 | 0.005000 | 0.013000 | 0.051724 | 0.044414 | 30.25 |
| 1k | duckdb-native | q29 | 0.006000 | 0.009000 | 0.023000 | 0.104354 | 0.104554 | 30.49 |
| 1k | duckdb-native | q30 | 0.008000 | 0.036000 | 0.037000 | 0.101648 | 0.098591 | 31.23 |
| 1k | duckdb-native | q31 | 0.027000 | 0.013000 | 0.011000 | 0.092382 | 0.094467 | 32.57 |
| 1k | duckdb-native | q32 | 0.005000 | 0.005000 | 0.011000 | 0.032312 | 0.032009 | 32.59 |
| 1k | duckdb-native | q33 | 0.014000 | 0.004000 | 0.006000 | 0.031602 | 0.028553 | 32.55 |
| 1k | duckdb-native | q34 | 0.003000 | 0.004000 | 0.009000 | 0.051859 | 0.037769 | 30.50 |
| 1k | duckdb-native | q35 | 0.003000 | 0.006000 | 0.006000 | 0.042901 | 0.035329 | 30.75 |
| 1k | duckdb-native | q36 | 0.004000 | 0.007000 | 0.012000 | 0.054414 | 0.046181 | 32.32 |
| 1k | duckdb-native | q37 | 0.002000 | 0.005000 | 0.005000 | 0.041587 | 0.034287 | 30.24 |
| 1k | duckdb-native | q38 | 0.002000 | 0.009000 | 0.014000 | 0.056067 | 0.037591 | 30.14 |
| 1k | duckdb-native | q39 | 0.004000 | 0.002000 | 0.010000 | 0.063242 | 0.057772 | 28.23 |
| 1k | duckdb-native | q40 | 0.008000 | 0.008000 | 0.013000 | 0.066821 | 0.044791 | 30.75 |
| 1k | duckdb-native | q41 | 0.004000 | 0.003000 | 0.006000 | 0.058135 | 0.052860 | 31.49 |
| 1k | duckdb-native | q42 | 0.005000 | 0.004000 | 0.004000 | 0.030368 | 0.026057 | 30.50 |
| 1k | duckdb-native | q43 | 0.002000 | 0.005000 | 0.003000 | 0.032281 | 0.029512 | 29.97 |
| 1k | rudb-native | q1 | 0.000644 | 0.000588 | 0.000098 | 0.010747 | 0.010418 | 11.27 |
| 1k | rudb-native | q2 | 0.000682 | 0.000726 | 0.000278 | 0.004187 | 0.003965 | 11.64 |
| 1k | rudb-native | q3 | 0.000671 | 0.000637 | 0.000040 | 0.003639 | 0.003207 | 11.88 |
| 1k | rudb-native | q4 | 0.000610 | 0.000648 | 0.000213 | 0.003502 | 0.003328 | 11.57 |
| 1k | rudb-native | q5 | 0.000546 | 0.000612 | 0.000137 | 0.003589 | 0.003340 | 11.01 |
| 1k | rudb-native | q6 | 0.000723 | 0.000620 | 0.000058 | 0.003677 | 0.003180 | 11.09 |
| 1k | rudb-native | q7 | 0.000624 | 0.000636 | 0.000138 | 0.003512 | 0.003388 | 11.43 |
| 1k | rudb-native | q8 | 0.000996 | 0.001115 | 0.001957 | 0.008145 | 0.003756 | 13.39 |
| 1k | rudb-native | q9 | 0.006638 | 0.001015 | 0.000127 | 0.007109 | 0.003515 | 13.08 |
| 1k | rudb-native | q10 | 0.001412 | 0.001519 | 0.002522 | 0.008678 | 0.004388 | 13.89 |
| 1k | rudb-native | q11 | 0.001229 | 0.001117 | 0.003774 | 0.006428 | 0.003966 | 13.40 |
| 1k | rudb-native | q12 | 0.001078 | 0.001095 | 0.002232 | 0.009473 | 0.003898 | 13.39 |
| 1k | rudb-native | q13 | 0.002619 | 0.004419 | 0.007409 | 0.029926 | 0.009591 | 13.93 |
| 1k | rudb-native | q14 | 0.001438 | 0.003247 | 0.008063 | 0.014986 | 0.007006 | 13.59 |
| 1k | rudb-native | q15 | 0.001304 | 0.001569 | 0.000412 | 0.015267 | 0.005077 | 13.77 |
| 1k | rudb-native | q16 | 0.012933 | 0.000696 | 0.000137 | 0.007872 | 0.003877 | 12.07 |
| 1k | rudb-native | q17 | 0.014327 | 0.000899 | 0.003325 | 0.010087 | 0.004024 | 12.29 |
| 1k | rudb-native | q18 | 0.002873 | 0.000772 | 0.002326 | 0.005816 | 0.003562 | 12.30 |
| 1k | rudb-native | q19 | 0.007600 | 0.005091 | 0.005021 | 0.010287 | 0.004123 | 12.75 |
| 1k | rudb-native | q20 | 0.000769 | 0.000664 | 0.000125 | 0.003995 | 0.003542 | 11.58 |
| 1k | rudb-native | q21 | 0.000852 | 0.000969 | 0.000472 | 0.004691 | 0.003836 | 13.24 |
| 1k | rudb-native | q22 | 0.000961 | 0.001165 | 0.000149 | 0.004339 | 0.004185 | 13.37 |
| 1k | rudb-native | q23 | 0.001285 | 0.001266 | 0.000189 | 0.006133 | 0.004343 | 13.65 |
| 1k | rudb-native | q24 | 0.001541 | 0.001170 | 0.000136 | 0.004078 | 0.003915 | 12.48 |
| 1k | rudb-native | q25 | 0.000919 | 0.001087 | 0.000087 | 0.004659 | 0.004286 | 12.63 |
| 1k | rudb-native | q26 | 0.000994 | 0.000975 | 0.000224 | 0.007184 | 0.007027 | 12.67 |
| 1k | rudb-native | q27 | 0.001122 | 0.000951 | 0.000113 | 0.008298 | 0.006503 | 12.61 |
| 1k | rudb-native | q28 | 0.001806 | 0.001592 | 0.002237 | 0.006762 | 0.006529 | 13.90 |
| 1k | rudb-native | q29 | 0.001797 | 0.001737 | 0.000738 | 0.008408 | 0.007968 | 14.09 |
| 1k | rudb-native | q30 | 0.003150 | 0.013761 | 0.012316 | 0.019452 | 0.017068 | 13.16 |
| 1k | rudb-native | q31 | 0.001595 | 0.002437 | 0.009032 | 0.011477 | 0.009694 | 13.78 |
| 1k | rudb-native | q32 | 0.001144 | 0.001289 | 0.001028 | 0.004795 | 0.004513 | 12.82 |
| 1k | rudb-native | q33 | 0.000951 | 0.000956 | 0.000321 | 0.004140 | 0.004021 | 12.29 |
| 1k | rudb-native | q34 | 0.011425 | 0.001020 | 0.000259 | 0.004753 | 0.004032 | 12.88 |
| 1k | rudb-native | q35 | 0.001612 | 0.004251 | 0.004091 | 0.008072 | 0.006190 | 13.82 |
| 1k | rudb-native | q36 | 0.001229 | 0.001575 | 0.003767 | 0.008471 | 0.008301 | 13.41 |
| 1k | rudb-native | q37 | 0.001927 | 0.003547 | 0.006548 | 0.012714 | 0.011081 | 14.31 |
| 1k | rudb-native | q38 | 0.001746 | 0.002921 | 0.003778 | 0.013615 | 0.010689 | 14.34 |
| 1k | rudb-native | q39 | 0.003474 | 0.001208 | 0.000153 | 0.009648 | 0.005173 | 13.37 |
| 1k | rudb-native | q40 | 0.006957 | 0.001184 | 0.000230 | 0.008263 | 0.004357 | 13.84 |
| 1k | rudb-native | q41 | 0.001437 | 0.001244 | 0.006480 | 0.004081 | 0.003954 | 13.66 |
| 1k | rudb-native | q42 | 0.001067 | 0.001225 | 0.000215 | 0.004395 | 0.004056 | 13.48 |
| 1k | rudb-native | q43 | 0.001032 | 0.001228 | 0.000089 | 0.004401 | 0.004231 | 13.76 |
| 1k | duckdb-parquet | q1 | 0.002000 | 0.002000 | 0.010000 | 0.073975 | 0.068656 | 27.21 |
| 1k | duckdb-parquet | q2 | 0.011000 | 0.004000 | 0.012000 | 0.042913 | 0.027930 | 27.72 |
| 1k | duckdb-parquet | q3 | 0.002000 | 0.004000 | 0.012000 | 0.038463 | 0.025394 | 28.01 |
| 1k | duckdb-parquet | q4 | 0.002000 | 0.003000 | 0.012000 | 0.046741 | 0.049405 | 27.27 |
| 1k | duckdb-parquet | q5 | 0.002000 | 0.006000 | 0.010000 | 0.040544 | 0.025716 | 28.24 |
| 1k | duckdb-parquet | q6 | 0.042000 | 0.005000 | 0.012000 | 0.038029 | 0.028581 | 28.92 |
| 1k | duckdb-parquet | q7 | 0.003000 | 0.003000 | 0.010000 | 0.043098 | 0.044373 | 27.63 |
| 1k | duckdb-parquet | q8 | 0.002000 | 0.010000 | 0.006000 | 0.070031 | 0.037686 | 29.50 |
| 1k | duckdb-parquet | q9 | 0.032000 | 0.016000 | 0.023000 | 0.096003 | 0.051621 | 31.75 |
| 1k | duckdb-parquet | q10 | 0.047000 | 0.010000 | 0.011000 | 0.063130 | 0.035849 | 32.43 |
| 1k | duckdb-parquet | q11 | 0.009000 | 0.008000 | 0.014000 | 0.067813 | 0.032420 | 30.50 |
| 1k | duckdb-parquet | q12 | 0.012000 | 0.015000 | 0.023000 | 0.127291 | 0.065000 | 31.75 |
| 1k | duckdb-parquet | q13 | 0.007000 | 0.012000 | 0.025000 | 0.121350 | 0.076171 | 29.32 |
| 1k | duckdb-parquet | q14 | 0.005000 | 0.019000 | 0.031000 | 0.095134 | 0.062909 | 32.25 |
| 1k | duckdb-parquet | q15 | 0.008000 | 0.009000 | 0.011000 | 0.076306 | 0.027487 | 30.18 |
| 1k | duckdb-parquet | q16 | 0.060000 | 0.008000 | 0.011000 | 0.064357 | 0.034185 | 30.84 |
| 1k | duckdb-parquet | q17 | 0.032000 | 0.010000 | 0.021000 | 0.077535 | 0.031721 | 31.35 |
| 1k | duckdb-parquet | q18 | 0.016000 | 0.008000 | 0.013000 | 0.060680 | 0.030489 | 30.84 |
| 1k | duckdb-parquet | q19 | 0.045000 | 0.008000 | 0.018000 | 0.084456 | 0.047504 | 31.56 |
| 1k | duckdb-parquet | q20 | 0.002000 | 0.006000 | 0.012000 | 0.049290 | 0.040269 | 27.50 |
| 1k | duckdb-parquet | q21 | 0.002000 | 0.002000 | 0.003000 | 0.039892 | 0.025762 | 28.00 |
| 1k | duckdb-parquet | q22 | 0.025000 | 0.004000 | 0.003000 | 0.041610 | 0.032259 | 29.00 |
| 1k | duckdb-parquet | q23 | 0.005000 | 0.004000 | 0.002000 | 0.048963 | 0.045257 | 28.78 |
| 1k | duckdb-parquet | q24 | 0.012000 | 0.013000 | 0.005000 | 0.043663 | 0.043485 | 32.48 |
| 1k | duckdb-parquet | q25 | 0.007000 | 0.010000 | 0.008000 | 0.047719 | 0.040918 | 30.44 |
| 1k | duckdb-parquet | q26 | 0.010000 | 0.003000 | 0.011000 | 0.108216 | 0.112128 | 28.21 |
| 1k | duckdb-parquet | q27 | 0.003000 | 0.004000 | 0.026000 | 0.071479 | 0.058318 | 28.01 |
| 1k | duckdb-parquet | q28 | 0.017000 | 0.013000 | 0.012000 | 0.079248 | 0.057259 | 29.51 |
| 1k | duckdb-parquet | q29 | 0.022000 | 0.010000 | 0.009000 | 0.076019 | 0.055289 | 30.23 |
| 1k | duckdb-parquet | q30 | 0.059000 | 0.045000 | 0.034000 | 0.111731 | 0.127032 | 30.88 |
| 1k | duckdb-parquet | q31 | 0.015000 | 0.014000 | 0.011000 | 0.043548 | 0.037790 | 32.32 |
| 1k | duckdb-parquet | q32 | 0.030000 | 0.006000 | 0.013000 | 0.037480 | 0.038026 | 31.82 |
| 1k | duckdb-parquet | q33 | 0.016000 | 0.006000 | 0.004000 | 0.036931 | 0.029232 | 31.66 |
| 1k | duckdb-parquet | q34 | 0.006000 | 0.004000 | 0.021000 | 0.075929 | 0.062525 | 30.17 |
| 1k | duckdb-parquet | q35 | 0.009000 | 0.005000 | 0.003000 | 0.044357 | 0.037264 | 30.49 |
| 1k | duckdb-parquet | q36 | 0.005000 | 0.009000 | 0.008000 | 0.046344 | 0.046821 | 31.08 |
| 1k | duckdb-parquet | q37 | 0.016000 | 0.004000 | 0.006000 | 0.031703 | 0.024350 | 29.93 |
| 1k | duckdb-parquet | q38 | 0.009000 | 0.009000 | 0.010000 | 0.055441 | 0.042013 | 29.74 |
| 1k | duckdb-parquet | q39 | 0.003000 | 0.003000 | 0.001000 | 0.051834 | 0.033205 | 28.00 |
| 1k | duckdb-parquet | q40 | 0.010000 | 0.016000 | 0.025000 | 0.080267 | 0.059111 | 31.00 |
| 1k | duckdb-parquet | q41 | 0.004000 | 0.013000 | 0.015000 | 0.062087 | 0.061998 | 31.06 |
| 1k | duckdb-parquet | q42 | 0.003000 | 0.008000 | 0.011000 | 0.045459 | 0.037897 | 30.00 |
| 1k | duckdb-parquet | q43 | 0.003000 | 0.004000 | 0.004000 | 0.040579 | 0.036427 | 29.73 |
| 1k | rudb-parquet | q1 | 0.001121 | 0.001044 | 0.000255 | 0.003999 | 0.003833 | 11.78 |
| 1k | rudb-parquet | q2 | 0.001138 | 0.001639 | 0.005011 | 0.003917 | 0.003789 | 12.66 |
| 1k | rudb-parquet | q3 | 0.000951 | 0.001183 | 0.000076 | 0.004228 | 0.003804 | 11.91 |
| 1k | rudb-parquet | q4 | 0.000857 | 0.001057 | 0.000153 | 0.010626 | 0.004011 | 12.36 |
| 1k | rudb-parquet | q5 | 0.001019 | 0.001107 | 0.000882 | 0.003730 | 0.003454 | 11.90 |
| 1k | rudb-parquet | q6 | 0.001412 | 0.001818 | 0.001134 | 0.005693 | 0.004689 | 12.23 |
| 1k | rudb-parquet | q7 | 0.001168 | 0.001203 | 0.010256 | 0.012073 | 0.011075 | 12.81 |
| 1k | rudb-parquet | q8 | 0.001272 | 0.001320 | 0.002639 | 0.007309 | 0.003841 | 12.87 |
| 1k | rudb-parquet | q9 | 0.001132 | 0.001451 | 0.003357 | 0.009951 | 0.004353 | 12.12 |
| 1k | rudb-parquet | q10 | 0.001743 | 0.001857 | 0.002403 | 0.010523 | 0.005115 | 12.93 |
| 1k | rudb-parquet | q11 | 0.001348 | 0.001657 | 0.013366 | 0.014827 | 0.004481 | 12.36 |
| 1k | rudb-parquet | q12 | 0.001309 | 0.001262 | 0.003674 | 0.007448 | 0.004752 | 12.42 |
| 1k | rudb-parquet | q13 | 0.001121 | 0.001453 | 0.006069 | 0.010331 | 0.004262 | 12.40 |
| 1k | rudb-parquet | q14 | 0.001432 | 0.001619 | 0.004644 | 0.010829 | 0.004383 | 12.41 |
| 1k | rudb-parquet | q15 | 0.001265 | 0.001578 | 0.002186 | 0.015088 | 0.004428 | 12.33 |
| 1k | rudb-parquet | q16 | 0.001345 | 0.003645 | 0.002323 | 0.009465 | 0.004723 | 12.12 |
| 1k | rudb-parquet | q17 | 0.003122 | 0.003072 | 0.005510 | 0.008054 | 0.004081 | 12.18 |
| 1k | rudb-parquet | q18 | 0.001184 | 0.002377 | 0.005931 | 0.012251 | 0.004013 | 12.12 |
| 1k | rudb-parquet | q19 | 0.001459 | 0.001528 | 0.000350 | 0.005082 | 0.003929 | 13.07 |
| 1k | rudb-parquet | q20 | 0.001006 | 0.000857 | 0.000132 | 0.005620 | 0.003462 | 11.86 |
| 1k | rudb-parquet | q21 | 0.001625 | 0.001866 | 0.000319 | 0.004523 | 0.004200 | 12.75 |
| 1k | rudb-parquet | q22 | 0.002033 | 0.002285 | 0.000882 | 0.006020 | 0.005837 | 12.83 |
| 1k | rudb-parquet | q23 | 0.002857 | 0.004436 | 0.003017 | 0.007470 | 0.007358 | 13.04 |
| 1k | rudb-parquet | q24 | 0.002355 | 0.001951 | 0.000417 | 0.005522 | 0.005198 | 12.17 |
| 1k | rudb-parquet | q25 | 0.001362 | 0.001618 | 0.001352 | 0.005204 | 0.004887 | 12.49 |
| 1k | rudb-parquet | q26 | 0.004610 | 0.001197 | 0.000166 | 0.004150 | 0.004061 | 11.92 |
| 1k | rudb-parquet | q27 | 0.002764 | 0.001410 | 0.000283 | 0.014302 | 0.006513 | 12.49 |
| 1k | rudb-parquet | q28 | 0.015112 | 0.003185 | 0.012709 | 0.006543 | 0.006467 | 13.03 |
| 1k | rudb-parquet | q29 | 0.001980 | 0.002198 | 0.013204 | 0.029802 | 0.029464 | 13.28 |
| 1k | rudb-parquet | q30 | 0.003905 | 0.004955 | 0.012454 | 0.010019 | 0.009919 | 13.23 |
| 1k | rudb-parquet | q31 | 0.014680 | 0.001804 | 0.000146 | 0.013174 | 0.013100 | 12.60 |
| 1k | rudb-parquet | q32 | 0.001613 | 0.001732 | 0.000877 | 0.005528 | 0.004453 | 12.58 |
| 1k | rudb-parquet | q33 | 0.002053 | 0.001660 | 0.001153 | 0.004986 | 0.004374 | 12.64 |
| 1k | rudb-parquet | q34 | 0.001691 | 0.001688 | 0.001383 | 0.004608 | 0.004546 | 12.42 |
| 1k | rudb-parquet | q35 | 0.004680 | 0.002753 | 0.001053 | 0.006335 | 0.006060 | 12.43 |
| 1k | rudb-parquet | q36 | 0.004374 | 0.001594 | 0.000165 | 0.006757 | 0.004525 | 12.50 |
| 1k | rudb-parquet | q37 | 0.002819 | 0.009522 | 0.010599 | 0.012504 | 0.012399 | 13.20 |
| 1k | rudb-parquet | q38 | 0.002090 | 0.002746 | 0.002199 | 0.008006 | 0.007216 | 13.22 |
| 1k | rudb-parquet | q39 | 0.006715 | 0.001992 | 0.000251 | 0.013606 | 0.009998 | 12.98 |
| 1k | rudb-parquet | q40 | 0.002599 | 0.005678 | 0.009258 | 0.009675 | 0.008580 | 13.59 |
| 1k | rudb-parquet | q41 | 0.001682 | 0.001557 | 0.001390 | 0.005053 | 0.004819 | 13.32 |
| 1k | rudb-parquet | q42 | 0.001397 | 0.001671 | 0.000183 | 0.005736 | 0.004485 | 13.29 |
| 1k | rudb-parquet | q43 | 0.001341 | 0.001529 | 0.000253 | 0.003830 | 0.003684 | 13.61 |
| 10k | duckdb-native | q1 | 0.000000 | 0.001000 | 0.000000 | 0.037906 | 0.031316 | 26.81 |
| 10k | duckdb-native | q2 | 0.001000 | 0.001000 | 0.009000 | 0.086223 | 0.058609 | 27.93 |
| 10k | duckdb-native | q3 | 0.001000 | 0.002000 | 0.007000 | 0.043973 | 0.036470 | 28.48 |
| 10k | duckdb-native | q4 | 0.001000 | 0.001000 | 0.001000 | 0.066205 | 0.036729 | 27.73 |
| 10k | duckdb-native | q5 | 0.011000 | 0.008000 | 0.012000 | 0.063412 | 0.041982 | 29.71 |
| 10k | duckdb-native | q6 | 0.002000 | 0.003000 | 0.014000 | 0.078356 | 0.051061 | 29.73 |
| 10k | duckdb-native | q7 | 0.001000 | 0.001000 | 0.002000 | 0.033009 | 0.024869 | 27.17 |
| 10k | duckdb-native | q8 | 0.003000 | 0.002000 | 0.000000 | 0.038579 | 0.026295 | 29.71 |
| 10k | duckdb-native | q9 | 0.004000 | 0.009000 | 0.016000 | 0.048814 | 0.034386 | 32.68 |
| 10k | duckdb-native | q10 | 0.018000 | 0.006000 | 0.004000 | 0.036090 | 0.033046 | 34.24 |
| 10k | duckdb-native | q11 | 0.003000 | 0.006000 | 0.017000 | 0.063990 | 0.042083 | 32.24 |
| 10k | duckdb-native | q12 | 0.003000 | 0.018000 | 0.020000 | 0.078624 | 0.063061 | 33.25 |
| 10k | duckdb-native | q13 | 0.011000 | 0.002000 | 0.002000 | 0.030835 | 0.025943 | 30.50 |
| 10k | duckdb-native | q14 | 0.004000 | 0.007000 | 0.007000 | 0.047648 | 0.039479 | 33.49 |
| 10k | duckdb-native | q15 | 0.007000 | 0.006000 | 0.008000 | 0.093663 | 0.072962 | 31.25 |
| 10k | duckdb-native | q16 | 0.007000 | 0.004000 | 0.026000 | 0.065087 | 0.045427 | 32.32 |
| 10k | duckdb-native | q17 | 0.004000 | 0.005000 | 0.002000 | 0.045136 | 0.027118 | 33.75 |
| 10k | duckdb-native | q18 | 0.005000 | 0.004000 | 0.003000 | 0.039996 | 0.026400 | 33.55 |
| 10k | duckdb-native | q19 | 0.008000 | 0.013000 | 0.029000 | 0.059028 | 0.072547 | 34.98 |
| 10k | duckdb-native | q20 | 0.001000 | 0.001000 | 0.000000 | 0.036728 | 0.029547 | 26.99 |
| 10k | duckdb-native | q21 | 0.003000 | 0.015000 | 0.008000 | 0.122043 | 0.100072 | 28.45 |
| 10k | duckdb-native | q22 | 0.011000 | 0.003000 | 0.002000 | 0.036714 | 0.027387 | 29.69 |
| 10k | duckdb-native | q23 | 0.010000 | 0.021000 | 0.016000 | 0.136441 | 0.129943 | 32.64 |
| 10k | duckdb-native | q24 | 0.020000 | 0.017000 | 0.033000 | 0.055529 | 0.049223 | 36.79 |
| 10k | duckdb-native | q25 | 0.003000 | 0.003000 | 0.004000 | 0.036088 | 0.025690 | 30.34 |
| 10k | duckdb-native | q26 | 0.005000 | 0.006000 | 0.024000 | 0.106221 | 0.068148 | 27.40 |
| 10k | duckdb-native | q27 | 0.002000 | 0.002000 | 0.000000 | 0.078236 | 0.040722 | 28.13 |
| 10k | duckdb-native | q28 | 0.009000 | 0.008000 | 0.027000 | 0.098080 | 0.043498 | 31.50 |
| 10k | duckdb-native | q29 | 0.061000 | 0.032000 | 0.041000 | 0.079825 | 0.064101 | 31.97 |
| 10k | duckdb-native | q30 | 0.015000 | 0.016000 | 0.015000 | 0.057041 | 0.049055 | 30.98 |
| 10k | duckdb-native | q31 | 0.016000 | 0.008000 | 0.014000 | 0.079120 | 0.035132 | 32.74 |
| 10k | duckdb-native | q32 | 0.033000 | 0.007000 | 0.014000 | 0.053856 | 0.025288 | 33.06 |
| 10k | duckdb-native | q33 | 0.007000 | 0.009000 | 0.008000 | 0.065824 | 0.032002 | 33.34 |
| 10k | duckdb-native | q34 | 0.012000 | 0.013000 | 0.029000 | 0.065354 | 0.051524 | 33.18 |
| 10k | duckdb-native | q35 | 0.036000 | 0.011000 | 0.012000 | 0.066901 | 0.035728 | 33.68 |
| 10k | duckdb-native | q36 | 0.004000 | 0.015000 | 0.009000 | 0.055796 | 0.034155 | 33.30 |
| 10k | duckdb-native | q37 | 0.011000 | 0.008000 | 0.027000 | 0.067673 | 0.036732 | 32.00 |
| 10k | duckdb-native | q38 | 0.010000 | 0.010000 | 0.025000 | 0.090977 | 0.035447 | 31.13 |
| 10k | duckdb-native | q39 | 0.007000 | 0.024000 | 0.040000 | 0.182776 | 0.089073 | 30.75 |
| 10k | duckdb-native | q40 | 0.006000 | 0.009000 | 0.026000 | 0.064110 | 0.034350 | 33.25 |
| 10k | duckdb-native | q41 | 0.007000 | 0.004000 | 0.029000 | 0.155513 | 0.073642 | 32.00 |
| 10k | duckdb-native | q42 | 0.008000 | 0.016000 | 0.022000 | 0.142090 | 0.071360 | 31.35 |
| 10k | duckdb-native | q43 | 0.032000 | 0.004000 | 0.008000 | 0.069518 | 0.033567 | 30.23 |
| 10k | rudb-native | q1 | 0.000590 | 0.000585 | 0.000062 | 0.005741 | 0.003989 | 11.27 |
| 10k | rudb-native | q2 | 0.000637 | 0.000644 | 0.000359 | 0.004796 | 0.003681 | 11.63 |
| 10k | rudb-native | q3 | 0.000677 | 0.000695 | 0.000090 | 0.008699 | 0.005588 | 11.86 |
| 10k | rudb-native | q4 | 0.000647 | 0.000643 | 0.001458 | 0.005766 | 0.004342 | 11.57 |
| 10k | rudb-native | q5 | 0.000541 | 0.000637 | 0.000098 | 0.006209 | 0.004094 | 11.03 |
| 10k | rudb-native | q6 | 0.000617 | 0.000632 | 0.000066 | 0.004515 | 0.003976 | 11.11 |
| 10k | rudb-native | q7 | 0.000636 | 0.000659 | 0.000100 | 0.004182 | 0.003943 | 11.44 |
| 10k | rudb-native | q8 | 0.001117 | 0.001700 | 0.001104 | 0.007486 | 0.005213 | 13.50 |
| 10k | rudb-native | q9 | 0.001228 | 0.001755 | 0.000628 | 0.006062 | 0.005821 | 13.82 |
| 10k | rudb-native | q10 | 0.002019 | 0.001998 | 0.000143 | 0.007150 | 0.005974 | 14.45 |
| 10k | rudb-native | q11 | 0.001318 | 0.002104 | 0.005613 | 0.010231 | 0.005747 | 13.82 |
| 10k | rudb-native | q12 | 0.005113 | 0.003330 | 0.003342 | 0.021817 | 0.008105 | 13.90 |
| 10k | rudb-native | q13 | 0.002092 | 0.001825 | 0.001862 | 0.009184 | 0.006046 | 14.28 |
| 10k | rudb-native | q14 | 0.002334 | 0.002969 | 0.001429 | 0.006119 | 0.006071 | 14.22 |
| 10k | rudb-native | q15 | 0.002425 | 0.004859 | 0.006047 | 0.014732 | 0.006938 | 14.55 |
| 10k | rudb-native | q16 | 0.001537 | 0.001973 | 0.007604 | 0.009677 | 0.006868 | 13.45 |
| 10k | rudb-native | q17 | 0.002282 | 0.011958 | 0.011703 | 0.015533 | 0.016634 | 14.35 |
| 10k | rudb-native | q18 | 0.001254 | 0.001256 | 0.000164 | 0.004889 | 0.004462 | 12.72 |
| 10k | rudb-native | q19 | 0.002073 | 0.004212 | 0.009666 | 0.012640 | 0.006666 | 14.83 |
| 10k | rudb-native | q20 | 0.000833 | 0.000787 | 0.000146 | 0.007664 | 0.005680 | 11.77 |
| 10k | rudb-native | q21 | 0.012227 | 0.001489 | 0.000201 | 0.017203 | 0.015763 | 13.24 |
| 10k | rudb-native | q22 | 0.002746 | 0.001981 | 0.000304 | 0.006581 | 0.005548 | 13.58 |
| 10k | rudb-native | q23 | 0.012138 | 0.030936 | 0.025744 | 0.042799 | 0.034462 | 14.99 |
| 10k | rudb-native | q24 | 0.010080 | 0.001528 | 0.000449 | 0.009281 | 0.009126 | 12.92 |
| 10k | rudb-native | q25 | 0.001291 | 0.001270 | 0.000086 | 0.004551 | 0.004357 | 13.14 |
| 10k | rudb-native | q26 | 0.001177 | 0.001301 | 0.009887 | 0.015340 | 0.005564 | 12.91 |
| 10k | rudb-native | q27 | 0.010636 | 0.005381 | 0.005046 | 0.015274 | 0.005663 | 13.12 |
| 10k | rudb-native | q28 | 0.005328 | 0.004056 | 0.006926 | 0.015680 | 0.008123 | 14.95 |
| 10k | rudb-native | q29 | 0.005240 | 0.005174 | 0.010652 | 0.012055 | 0.009430 | 15.09 |
| 10k | rudb-native | q30 | 0.003709 | 0.007503 | 0.011231 | 0.015330 | 0.014977 | 12.98 |
| 10k | rudb-native | q31 | 0.014359 | 0.005661 | 0.004685 | 0.017371 | 0.005705 | 14.25 |
| 10k | rudb-native | q32 | 0.003339 | 0.001210 | 0.006190 | 0.011988 | 0.004832 | 13.04 |
| 10k | rudb-native | q33 | 0.005437 | 0.000936 | 0.002826 | 0.015512 | 0.004641 | 12.62 |
| 10k | rudb-native | q34 | 0.006020 | 0.000993 | 0.001542 | 0.014400 | 0.005030 | 12.71 |
| 10k | rudb-native | q35 | 0.017032 | 0.003089 | 0.003104 | 0.012528 | 0.005809 | 14.30 |
| 10k | rudb-native | q36 | 0.000880 | 0.000996 | 0.026881 | 0.015335 | 0.004437 | 12.80 |
| 10k | rudb-native | q37 | 0.015914 | 0.003049 | 0.009401 | 0.017709 | 0.006000 | 14.84 |
| 10k | rudb-native | q38 | 0.007639 | 0.009391 | 0.010177 | 0.022597 | 0.007290 | 14.87 |
| 10k | rudb-native | q39 | 0.029888 | 0.012927 | 0.014536 | 0.032880 | 0.006174 | 14.17 |
| 10k | rudb-native | q40 | 0.013560 | 0.009260 | 0.009294 | 0.016576 | 0.005710 | 14.20 |
| 10k | rudb-native | q41 | 0.007608 | 0.002541 | 0.002147 | 0.016864 | 0.005677 | 14.43 |
| 10k | rudb-native | q42 | 0.015640 | 0.005182 | 0.005933 | 0.014719 | 0.004632 | 14.67 |
| 10k | rudb-native | q43 | 0.004727 | 0.004187 | 0.010080 | 0.015875 | 0.004992 | 14.34 |
| 10k | duckdb-parquet | q1 | 0.001000 | 0.002000 | 0.000000 | 0.038638 | 0.024735 | 26.83 |
| 10k | duckdb-parquet | q2 | 0.004000 | 0.003000 | 0.003000 | 0.063259 | 0.042428 | 27.68 |
| 10k | duckdb-parquet | q3 | 0.006000 | 0.005000 | 0.003000 | 0.040351 | 0.027440 | 27.76 |
| 10k | duckdb-parquet | q4 | 0.003000 | 0.005000 | 0.013000 | 0.077786 | 0.056897 | 27.48 |
| 10k | duckdb-parquet | q5 | 0.004000 | 0.006000 | 0.004000 | 0.050042 | 0.028500 | 29.66 |
| 10k | duckdb-parquet | q6 | 0.017000 | 0.009000 | 0.021000 | 0.102596 | 0.096082 | 28.90 |
| 10k | duckdb-parquet | q7 | 0.004000 | 0.003000 | 0.001000 | 0.035468 | 0.027213 | 27.72 |
| 10k | duckdb-parquet | q8 | 0.014000 | 0.006000 | 0.004000 | 0.034669 | 0.028801 | 29.51 |
| 10k | duckdb-parquet | q9 | 0.012000 | 0.015000 | 0.032000 | 0.056539 | 0.050097 | 33.00 |
| 10k | duckdb-parquet | q10 | 0.014000 | 0.012000 | 0.010000 | 0.033277 | 0.030776 | 33.73 |
| 10k | duckdb-parquet | q11 | 0.003000 | 0.016000 | 0.021000 | 0.084183 | 0.065235 | 31.00 |
| 10k | duckdb-parquet | q12 | 0.015000 | 0.015000 | 0.011000 | 0.069187 | 0.038573 | 31.75 |
| 10k | duckdb-parquet | q13 | 0.006000 | 0.006000 | 0.007000 | 0.033491 | 0.030361 | 30.69 |
| 10k | duckdb-parquet | q14 | 0.008000 | 0.008000 | 0.009000 | 0.039079 | 0.037580 | 33.75 |
| 10k | duckdb-parquet | q15 | 0.009000 | 0.015000 | 0.049000 | 0.126197 | 0.096337 | 30.49 |
| 10k | duckdb-parquet | q16 | 0.009000 | 0.010000 | 0.024000 | 0.063783 | 0.058851 | 32.06 |
| 10k | duckdb-parquet | q17 | 0.015000 | 0.019000 | 0.017000 | 0.094510 | 0.091237 | 33.32 |
| 10k | duckdb-parquet | q18 | 0.006000 | 0.009000 | 0.005000 | 0.040098 | 0.030769 | 33.48 |
| 10k | duckdb-parquet | q19 | 0.014000 | 0.016000 | 0.021000 | 0.082212 | 0.083839 | 34.34 |
| 10k | duckdb-parquet | q20 | 0.002000 | 0.002000 | 0.001000 | 0.068646 | 0.045965 | 27.43 |
| 10k | duckdb-parquet | q21 | 0.034000 | 0.012000 | 0.010000 | 0.096060 | 0.091752 | 28.73 |
| 10k | duckdb-parquet | q22 | 0.031000 | 0.007000 | 0.006000 | 0.044601 | 0.031466 | 29.24 |
| 10k | duckdb-parquet | q23 | 0.012000 | 0.017000 | 0.031000 | 0.145345 | 0.137686 | 32.00 |
| 10k | duckdb-parquet | q24 | 0.058000 | 0.044000 | 0.039000 | 0.092075 | 0.085704 | 37.24 |
| 10k | duckdb-parquet | q25 | 0.006000 | 0.010000 | 0.006000 | 0.046278 | 0.034252 | 30.42 |
| 10k | duckdb-parquet | q26 | 0.029000 | 0.007000 | 0.006000 | 0.073034 | 0.034171 | 27.48 |
| 10k | duckdb-parquet | q27 | 0.032000 | 0.006000 | 0.009000 | 0.073240 | 0.034525 | 27.72 |
| 10k | duckdb-parquet | q28 | 0.072000 | 0.010000 | 0.026000 | 0.204286 | 0.113038 | 30.51 |
| 10k | duckdb-parquet | q29 | 0.086000 | 0.042000 | 0.069000 | 0.091251 | 0.104800 | 31.98 |
| 10k | duckdb-parquet | q30 | 0.010000 | 0.033000 | 0.047000 | 0.097554 | 0.079679 | 30.93 |
| 10k | duckdb-parquet | q31 | 0.025000 | 0.013000 | 0.014000 | 0.101793 | 0.053476 | 32.07 |
| 10k | duckdb-parquet | q32 | 0.006000 | 0.009000 | 0.038000 | 0.063710 | 0.032540 | 31.94 |
| 10k | duckdb-parquet | q33 | 0.008000 | 0.009000 | 0.003000 | 0.053917 | 0.029680 | 33.59 |
| 10k | duckdb-parquet | q34 | 0.064000 | 0.024000 | 0.018000 | 0.062355 | 0.041591 | 33.49 |
| 10k | duckdb-parquet | q35 | 0.010000 | 0.021000 | 0.024000 | 0.083105 | 0.055206 | 33.21 |
| 10k | duckdb-parquet | q36 | 0.003000 | 0.029000 | 0.015000 | 0.114717 | 0.073421 | 31.60 |
| 10k | duckdb-parquet | q37 | 0.013000 | 0.019000 | 0.036000 | 0.079345 | 0.041672 | 31.00 |
| 10k | duckdb-parquet | q38 | 0.062000 | 0.014000 | 0.011000 | 0.086893 | 0.056471 | 30.74 |
| 10k | duckdb-parquet | q39 | 0.010000 | 0.014000 | 0.051000 | 0.093878 | 0.039357 | 30.50 |
| 10k | duckdb-parquet | q40 | 0.016000 | 0.018000 | 0.014000 | 0.075233 | 0.041784 | 32.67 |
| 10k | duckdb-parquet | q41 | 0.004000 | 0.009000 | 0.040000 | 0.078814 | 0.036458 | 31.55 |
| 10k | duckdb-parquet | q42 | 0.004000 | 0.013000 | 0.005000 | 0.100987 | 0.050612 | 30.47 |
| 10k | duckdb-parquet | q43 | 0.030000 | 0.015000 | 0.021000 | 0.107967 | 0.044793 | 29.71 |
| 10k | rudb-parquet | q1 | 0.001037 | 0.000955 | 0.000126 | 0.003558 | 0.003470 | 11.78 |
| 10k | rudb-parquet | q2 | 0.001043 | 0.001237 | 0.010284 | 0.006130 | 0.003756 | 12.63 |
| 10k | rudb-parquet | q3 | 0.001446 | 0.001253 | 0.000257 | 0.004711 | 0.003720 | 12.16 |
| 10k | rudb-parquet | q4 | 0.001048 | 0.001257 | 0.000397 | 0.004182 | 0.003697 | 12.37 |
| 10k | rudb-parquet | q5 | 0.002036 | 0.001326 | 0.000464 | 0.004309 | 0.004099 | 12.17 |
| 10k | rudb-parquet | q6 | 0.002085 | 0.013601 | 0.012821 | 0.016527 | 0.016300 | 12.73 |
| 10k | rudb-parquet | q7 | 0.004962 | 0.001258 | 0.000936 | 0.005810 | 0.005472 | 12.74 |
| 10k | rudb-parquet | q8 | 0.013526 | 0.001287 | 0.000560 | 0.004262 | 0.003704 | 12.84 |
| 10k | rudb-parquet | q9 | 0.001887 | 0.002078 | 0.002507 | 0.005335 | 0.005056 | 12.65 |
| 10k | rudb-parquet | q10 | 0.002745 | 0.002836 | 0.000947 | 0.008205 | 0.005990 | 13.49 |
| 10k | rudb-parquet | q11 | 0.002024 | 0.001697 | 0.000325 | 0.007002 | 0.004665 | 12.68 |
| 10k | rudb-parquet | q12 | 0.004733 | 0.001925 | 0.002178 | 0.007668 | 0.005096 | 13.00 |
| 10k | rudb-parquet | q13 | 0.002392 | 0.002287 | 0.002308 | 0.006593 | 0.005810 | 12.66 |
| 10k | rudb-parquet | q14 | 0.002344 | 0.003699 | 0.005933 | 0.007599 | 0.006328 | 12.91 |
| 10k | rudb-parquet | q15 | 0.002450 | 0.005816 | 0.015677 | 0.017856 | 0.009240 | 12.58 |
| 10k | rudb-parquet | q16 | 0.001911 | 0.002683 | 0.001570 | 0.005511 | 0.005160 | 12.39 |
| 10k | rudb-parquet | q17 | 0.011137 | 0.004028 | 0.002513 | 0.007140 | 0.006978 | 13.47 |
| 10k | rudb-parquet | q18 | 0.002029 | 0.002251 | 0.000521 | 0.004880 | 0.004720 | 12.64 |
| 10k | rudb-parquet | q19 | 0.009247 | 0.005459 | 0.005074 | 0.010798 | 0.010686 | 14.81 |
| 10k | rudb-parquet | q20 | 0.001236 | 0.001081 | 0.003401 | 0.008289 | 0.003801 | 11.77 |
| 10k | rudb-parquet | q21 | 0.014283 | 0.033112 | 0.005160 | 0.044619 | 0.041645 | 14.65 |
| 10k | rudb-parquet | q22 | 0.032668 | 0.008087 | 0.002775 | 0.010835 | 0.010525 | 14.73 |
| 10k | rudb-parquet | q23 | 0.013684 | 0.078839 | 0.029168 | 0.082335 | 0.079684 | 16.29 |
| 10k | rudb-parquet | q24 | 0.031888 | 0.011219 | 0.004934 | 0.015369 | 0.014079 | 14.43 |
| 10k | rudb-parquet | q25 | 0.002086 | 0.002731 | 0.001887 | 0.005753 | 0.005525 | 12.68 |
| 10k | rudb-parquet | q26 | 0.005038 | 0.008415 | 0.004639 | 0.016301 | 0.005370 | 11.89 |
| 10k | rudb-parquet | q27 | 0.014079 | 0.006206 | 0.004997 | 0.013547 | 0.005427 | 12.68 |
| 10k | rudb-parquet | q28 | 0.032016 | 0.029632 | 0.018153 | 0.040404 | 0.018195 | 14.51 |
| 10k | rudb-parquet | q29 | 0.013931 | 0.014838 | 0.017008 | 0.022183 | 0.013928 | 14.85 |
| 10k | rudb-parquet | q30 | 0.003618 | 0.008606 | 0.010637 | 0.018007 | 0.017553 | 13.37 |
| 10k | rudb-parquet | q31 | 0.015209 | 0.007756 | 0.007021 | 0.010548 | 0.005720 | 13.26 |
| 10k | rudb-parquet | q32 | 0.005722 | 0.008002 | 0.027386 | 0.015337 | 0.007998 | 13.26 |
| 10k | rudb-parquet | q33 | 0.011867 | 0.007410 | 0.008907 | 0.011302 | 0.005032 | 13.35 |
| 10k | rudb-parquet | q34 | 0.018586 | 0.014897 | 0.022537 | 0.034092 | 0.020028 | 15.43 |
| 10k | rudb-parquet | q35 | 0.014329 | 0.017766 | 0.020369 | 0.022769 | 0.012895 | 15.41 |
| 10k | rudb-parquet | q36 | 0.002254 | 0.015896 | 0.012658 | 0.029146 | 0.004611 | 12.90 |
| 10k | rudb-parquet | q37 | 0.086938 | 0.015223 | 0.009869 | 0.028668 | 0.009465 | 14.80 |
| 10k | rudb-parquet | q38 | 0.015487 | 0.014492 | 0.009235 | 0.021706 | 0.009212 | 14.77 |
| 10k | rudb-parquet | q39 | 0.024828 | 0.018348 | 0.030516 | 0.023432 | 0.010152 | 14.52 |
| 10k | rudb-parquet | q40 | 0.015389 | 0.030256 | 0.029661 | 0.033800 | 0.016162 | 15.48 |
| 10k | rudb-parquet | q41 | 0.002310 | 0.006656 | 0.005178 | 0.015202 | 0.005285 | 13.72 |
| 10k | rudb-parquet | q42 | 0.005563 | 0.004419 | 0.003945 | 0.015381 | 0.007392 | 13.54 |
| 10k | rudb-parquet | q43 | 0.007359 | 0.002221 | 0.003212 | 0.016991 | 0.004780 | 13.68 |
| 1m | duckdb-native | q1 | 0.001000 | 0.001000 | 0.002000 | 0.050361 | 0.032957 | 26.97 |
| 1m | duckdb-native | q2 | 0.033000 | 0.012000 | 0.015000 | 0.132546 | 0.075176 | 30.48 |
| 1m | duckdb-native | q3 | 0.005000 | 0.007000 | 0.013000 | 0.079325 | 0.042815 | 33.24 |
| 1m | duckdb-native | q4 | 0.014000 | 0.014000 | 0.004000 | 0.097056 | 0.042951 | 36.67 |
| 1m | duckdb-native | q5 | 0.124000 | 0.107000 | 0.083000 | 0.171458 | 0.204176 | 66.41 |
| 1m | duckdb-native | q6 | 0.138000 | 0.056000 | 0.055000 | 0.129984 | 0.147485 | 57.13 |
| 1m | duckdb-native | q7 | 0.001000 | 0.001000 | 0.001000 | 0.082874 | 0.050131 | 27.47 |
| 1m | duckdb-native | q8 | 0.032000 | 0.008000 | 0.020000 | 0.082485 | 0.067408 | 32.24 |
| 1m | duckdb-native | q9 | 0.110000 | 0.114000 | 0.032000 | 0.205095 | 0.260105 | 77.99 |
| 1m | duckdb-native | q10 | 0.115000 | 0.160000 | 0.070000 | 0.236898 | 0.307799 | 85.03 |
| 1m | duckdb-native | q11 | 0.043000 | 0.049000 | 0.058000 | 0.157092 | 0.160477 | 51.99 |
| 1m | duckdb-native | q12 | 0.028000 | 0.049000 | 0.063000 | 0.114183 | 0.127333 | 53.75 |
| 1m | duckdb-native | q13 | 0.047000 | 0.056000 | 0.028000 | 0.107436 | 0.121241 | 59.57 |
| 1m | duckdb-native | q14 | 0.244000 | 0.090000 | 0.076000 | 0.182080 | 0.182234 | 86.17 |
| 1m | duckdb-native | q15 | 0.118000 | 0.112000 | 0.074000 | 0.267159 | 0.201880 | 63.98 |
| 1m | duckdb-native | q16 | 0.051000 | 0.136000 | 0.155000 | 0.215618 | 0.190151 | 81.00 |
| 1m | duckdb-native | q17 | 0.090000 | 0.155000 | 0.151000 | 0.226272 | 0.387425 | 124.35 |
| 1m | duckdb-native | q18 | 0.178000 | 0.195000 | 0.062000 | 0.303788 | 0.384186 | 120.81 |
| 1m | duckdb-native | q19 | 0.131000 | 0.238000 | 0.089000 | 0.434461 | 0.509392 | 139.30 |
| 1m | duckdb-native | q20 | 0.015000 | 0.014000 | 0.021000 | 0.113170 | 0.050035 | 35.98 |
| 1m | duckdb-native | q21 | 0.069000 | 0.140000 | 0.087000 | 0.220276 | 0.261614 | 91.48 |
| 1m | duckdb-native | q22 | 0.093000 | 0.173000 | 0.104000 | 0.237823 | 0.266004 | 103.18 |
| 1m | duckdb-native | q23 | 0.259000 | 0.154000 | 0.117000 | 0.244987 | 0.262105 | 121.78 |
| 1m | duckdb-native | q24 | 0.493000 | 0.257000 | 0.039000 | 0.314586 | 0.358736 | 203.38 |
| 1m | duckdb-native | q25 | 0.026000 | 0.063000 | 0.047000 | 0.156911 | 0.134373 | 41.91 |
| 1m | duckdb-native | q26 | 0.104000 | 0.047000 | 0.042000 | 0.140941 | 0.149564 | 37.45 |
| 1m | duckdb-native | q27 | 0.020000 | 0.031000 | 0.031000 | 0.093836 | 0.062719 | 39.46 |
| 1m | duckdb-native | q28 | 0.235000 | 0.156000 | 0.071000 | 0.233555 | 0.248947 | 98.97 |
| 1m | duckdb-native | q29 | 1.237000 | 0.907000 | 0.360000 | 1.035370 | 1.899422 | 138.02 |
| 1m | duckdb-native | q30 | 0.015000 | 0.023000 | 0.036000 | 0.111168 | 0.082493 | 33.75 |
| 1m | duckdb-native | q31 | 0.089000 | 0.146000 | 0.108000 | 0.210430 | 0.228104 | 68.92 |
| 1m | duckdb-native | q32 | 0.122000 | 0.108000 | 0.155000 | 0.208735 | 0.171405 | 76.03 |
| 1m | duckdb-native | q33 | 0.237000 | 0.135000 | 0.073000 | 0.289873 | 0.341214 | 132.78 |
| 1m | duckdb-native | q34 | 0.401000 | 0.350000 | 0.378000 | 0.480165 | 0.669754 | 235.23 |
| 1m | duckdb-native | q35 | 0.202000 | 0.419000 | 0.168000 | 0.607594 | 1.020967 | 234.05 |
| 1m | duckdb-native | q36 | 0.054000 | 0.147000 | 0.063000 | 0.206358 | 0.294844 | 76.34 |
| 1m | duckdb-native | q37 | 0.009000 | 0.024000 | 0.016000 | 0.096686 | 0.054788 | 36.26 |
| 1m | duckdb-native | q38 | 0.011000 | 0.038000 | 0.049000 | 0.173402 | 0.105442 | 34.33 |
| 1m | duckdb-native | q39 | 0.012000 | 0.010000 | 0.011000 | 0.076830 | 0.042958 | 34.47 |
| 1m | duckdb-native | q40 | 0.087000 | 0.054000 | 0.026000 | 0.159394 | 0.124104 | 42.04 |
| 1m | duckdb-native | q41 | 0.064000 | 0.014000 | 0.016000 | 0.066732 | 0.036935 | 35.01 |
| 1m | duckdb-native | q42 | 0.013000 | 0.018000 | 0.016000 | 0.078084 | 0.031984 | 34.03 |
| 1m | duckdb-native | q43 | 0.023000 | 0.008000 | 0.016000 | 0.100765 | 0.051526 | 32.53 |
| 1m | rudb-native | q1 | 0.006049 | 0.000606 | 0.004115 | 0.012213 | 0.005428 | 11.28 |
| 1m | rudb-native | q2 | 0.000610 | 0.000654 | 0.000095 | 0.015247 | 0.003945 | 11.47 |
| 1m | rudb-native | q3 | 0.000591 | 0.000655 | 0.005338 | 0.012301 | 0.003620 | 11.59 |
| 1m | rudb-native | q4 | 0.000639 | 0.000675 | 0.000168 | 0.013931 | 0.005943 | 11.29 |
| 1m | rudb-native | q5 | 0.000527 | 0.000652 | 0.004240 | 0.007441 | 0.005309 | 11.31 |
| 1m | rudb-native | q6 | 0.004418 | 0.000602 | 0.000720 | 0.008949 | 0.003966 | 11.37 |
| 1m | rudb-native | q7 | 0.000662 | 0.000590 | 0.000117 | 0.014245 | 0.003863 | 11.56 |
| 1m | rudb-native | q8 | 0.011495 | 0.002453 | 0.005694 | 0.016348 | 0.007572 | 20.16 |
| 1m | rudb-native | q9 | 0.021271 | 0.034247 | 0.029988 | 0.063737 | 0.076410 | 46.59 |
| 1m | rudb-native | q10 | 0.041919 | 0.062396 | 0.033447 | 0.079121 | 0.095336 | 65.25 |
| 1m | rudb-native | q11 | 0.009465 | 0.008274 | 0.013997 | 0.018851 | 0.016092 | 37.12 |
| 1m | rudb-native | q12 | 0.011051 | 0.014360 | 0.004873 | 0.025379 | 0.019170 | 39.10 |
| 1m | rudb-native | q13 | 0.017771 | 0.004896 | 0.005790 | 0.016720 | 0.009691 | 27.29 |
| 1m | rudb-native | q14 | 0.060559 | 0.030225 | 0.034618 | 0.047651 | 0.040996 | 46.61 |
| 1m | rudb-native | q15 | 0.033300 | 0.021035 | 0.014162 | 0.030028 | 0.024872 | 30.36 |
| 1m | rudb-native | q16 | 0.007232 | 0.000994 | 0.000156 | 0.011799 | 0.004584 | 12.45 |
| 1m | rudb-native | q17 | 0.026805 | 0.061658 | 0.085602 | 0.094200 | 0.105654 | 57.96 |
| 1m | rudb-native | q18 | 0.002629 | 0.002621 | 0.001932 | 0.011965 | 0.008252 | 45.16 |
| 1m | rudb-native | q19 | 0.070733 | 0.108066 | 0.081584 | 0.121537 | 0.138805 | 63.82 |
| 1m | rudb-native | q20 | 0.006565 | 0.002483 | 0.006267 | 0.024702 | 0.006260 | 19.99 |
| 1m | rudb-native | q21 | 0.047570 | 0.053052 | 0.065808 | 0.058506 | 0.084030 | 48.90 |
| 1m | rudb-native | q22 | 0.088956 | 0.077926 | 0.016754 | 0.093848 | 0.134373 | 69.78 |
| 1m | rudb-native | q23 | 0.074534 | 0.080766 | 0.098007 | 0.125973 | 0.107358 | 73.77 |
| 1m | rudb-native | q24 | 0.203018 | 0.067862 | 0.032997 | 0.094203 | 0.103002 | 104.94 |
| 1m | rudb-native | q25 | 0.018816 | 0.007525 | 0.009100 | 0.027804 | 0.008297 | 36.57 |
| 1m | rudb-native | q26 | 0.035953 | 0.029349 | 0.027397 | 0.057085 | 0.044383 | 28.57 |
| 1m | rudb-native | q27 | 0.018242 | 0.011215 | 0.015097 | 0.031538 | 0.015604 | 36.98 |
| 1m | rudb-native | q28 | 0.015915 | 0.022762 | 0.011665 | 0.035935 | 0.031288 | 49.11 |
| 1m | rudb-native | q29 | 0.360633 | 0.227104 | 0.146802 | 0.260521 | 0.331580 | 69.41 |
| 1m | rudb-native | q30 | 0.007913 | 0.006671 | 0.012709 | 0.021584 | 0.008144 | 13.24 |
| 1m | rudb-native | q31 | 0.016482 | 0.038003 | 0.024169 | 0.056044 | 0.048392 | 51.59 |
| 1m | rudb-native | q32 | 0.013950 | 0.001607 | 0.002230 | 0.013189 | 0.005608 | 13.59 |
| 1m | rudb-native | q33 | 0.000819 | 0.001206 | 0.005259 | 0.015495 | 0.005314 | 12.73 |
| 1m | rudb-native | q34 | 0.000992 | 0.001201 | 0.000780 | 0.009024 | 0.004627 | 12.52 |
| 1m | rudb-native | q35 | 0.029462 | 0.033495 | 0.013199 | 0.046580 | 0.026989 | 22.32 |
| 1m | rudb-native | q36 | 0.001175 | 0.002237 | 0.003840 | 0.016431 | 0.007350 | 12.79 |
| 1m | rudb-native | q37 | 0.036465 | 0.010167 | 0.022536 | 0.031422 | 0.011267 | 17.34 |
| 1m | rudb-native | q38 | 0.032075 | 0.012811 | 0.012977 | 0.032705 | 0.011700 | 16.48 |
| 1m | rudb-native | q39 | 0.006328 | 0.007829 | 0.003764 | 0.024543 | 0.009336 | 16.51 |
| 1m | rudb-native | q40 | 0.035788 | 0.036792 | 0.025101 | 0.044263 | 0.024693 | 20.91 |
| 1m | rudb-native | q41 | 0.005994 | 0.005016 | 0.005433 | 0.015670 | 0.006548 | 16.25 |
| 1m | rudb-native | q42 | 0.004271 | 0.004301 | 0.003070 | 0.015371 | 0.006458 | 16.01 |
| 1m | rudb-native | q43 | 0.045267 | 0.008436 | 0.007684 | 0.031873 | 0.013141 | 15.90 |
| 1m | duckdb-parquet | q1 | 0.004000 | 0.014000 | 0.025000 | 0.063501 | 0.048163 | 27.17 |
| 1m | duckdb-parquet | q2 | 0.023000 | 0.015000 | 0.024000 | 0.103965 | 0.102425 | 29.29 |
| 1m | duckdb-parquet | q3 | 0.010000 | 0.010000 | 0.033000 | 0.062227 | 0.035967 | 30.17 |
| 1m | duckdb-parquet | q4 | 0.025000 | 0.024000 | 0.020000 | 0.094359 | 0.057951 | 38.80 |
| 1m | duckdb-parquet | q5 | 0.096000 | 0.100000 | 0.087000 | 0.219551 | 0.305363 | 61.11 |
| 1m | duckdb-parquet | q6 | 0.097000 | 0.053000 | 0.063000 | 0.142634 | 0.140183 | 55.66 |
| 1m | duckdb-parquet | q7 | 0.030000 | 0.030000 | 0.035000 | 0.125049 | 0.102882 | 29.07 |
| 1m | duckdb-parquet | q8 | 0.011000 | 0.015000 | 0.012000 | 0.107598 | 0.050096 | 30.75 |
| 1m | duckdb-parquet | q9 | 0.065000 | 0.102000 | 0.117000 | 0.190543 | 0.293534 | 69.89 |
| 1m | duckdb-parquet | q10 | 0.105000 | 0.140000 | 0.091000 | 0.216572 | 0.299784 | 80.76 |
| 1m | duckdb-parquet | q11 | 0.114000 | 0.049000 | 0.098000 | 0.144251 | 0.098776 | 49.50 |
| 1m | duckdb-parquet | q12 | 0.019000 | 0.053000 | 0.103000 | 0.124705 | 0.143281 | 50.92 |
| 1m | duckdb-parquet | q13 | 0.111000 | 0.070000 | 0.032000 | 0.150432 | 0.141519 | 60.70 |
| 1m | duckdb-parquet | q14 | 0.149000 | 0.142000 | 0.136000 | 0.221127 | 0.253732 | 82.25 |
| 1m | duckdb-parquet | q15 | 0.144000 | 0.127000 | 0.043000 | 0.255149 | 0.204099 | 63.47 |
| 1m | duckdb-parquet | q16 | 0.072000 | 0.141000 | 0.119000 | 0.210132 | 0.214213 | 79.20 |
| 1m | duckdb-parquet | q17 | 0.101000 | 0.144000 | 0.118000 | 0.223219 | 0.318994 | 120.73 |
| 1m | duckdb-parquet | q18 | 0.115000 | 0.183000 | 0.133000 | 0.263768 | 0.412978 | 115.53 |
| 1m | duckdb-parquet | q19 | 0.302000 | 0.249000 | 0.143000 | 0.327718 | 0.572697 | 133.86 |
| 1m | duckdb-parquet | q20 | 0.022000 | 0.033000 | 0.021000 | 0.079098 | 0.071478 | 39.48 |
| 1m | duckdb-parquet | q21 | 0.289000 | 0.254000 | 0.242000 | 0.331486 | 0.450269 | 122.73 |
| 1m | duckdb-parquet | q22 | 0.403000 | 0.221000 | 0.111000 | 0.309585 | 0.539005 | 148.74 |
| 1m | duckdb-parquet | q23 | 0.341000 | 0.396000 | 0.081000 | 0.544374 | 1.040369 | 252.02 |
| 1m | duckdb-parquet | q24 | 0.783000 | 0.628000 | 0.390000 | 0.777202 | 1.520155 | 357.64 |
| 1m | duckdb-parquet | q25 | 0.141000 | 0.161000 | 0.125000 | 0.228001 | 0.337443 | 56.30 |
| 1m | duckdb-parquet | q26 | 0.077000 | 0.068000 | 0.053000 | 0.172078 | 0.139393 | 39.89 |
| 1m | duckdb-parquet | q27 | 0.147000 | 0.073000 | 0.043000 | 0.138634 | 0.161666 | 50.24 |
| 1m | duckdb-parquet | q28 | 0.138000 | 0.182000 | 0.105000 | 0.286750 | 0.448098 | 141.75 |
| 1m | duckdb-parquet | q29 | 0.732000 | 0.799000 | 0.466000 | 0.953429 | 1.822725 | 164.53 |
| 1m | duckdb-parquet | q30 | 0.072000 | 0.072000 | 0.068000 | 0.155350 | 0.144427 | 32.48 |
| 1m | duckdb-parquet | q31 | 0.211000 | 0.158000 | 0.113000 | 0.308515 | 0.341122 | 62.41 |
| 1m | duckdb-parquet | q32 | 0.208000 | 0.138000 | 0.054000 | 0.297503 | 0.305088 | 69.43 |
| 1m | duckdb-parquet | q33 | 0.245000 | 0.163000 | 0.086000 | 0.291023 | 0.428560 | 121.10 |
| 1m | duckdb-parquet | q34 | 0.268000 | 0.343000 | 0.066000 | 0.465509 | 0.840766 | 245.00 |
| 1m | duckdb-parquet | q35 | 0.205000 | 0.422000 | 0.197000 | 0.511296 | 1.021419 | 250.35 |
| 1m | duckdb-parquet | q36 | 0.136000 | 0.121000 | 0.268000 | 0.237789 | 0.270152 | 76.70 |
| 1m | duckdb-parquet | q37 | 0.111000 | 0.141000 | 0.143000 | 0.278823 | 0.285320 | 63.77 |
| 1m | duckdb-parquet | q38 | 0.082000 | 0.095000 | 0.077000 | 0.174201 | 0.153602 | 59.82 |
| 1m | duckdb-parquet | q39 | 0.095000 | 0.125000 | 0.133000 | 0.233689 | 0.187772 | 63.24 |
| 1m | duckdb-parquet | q40 | 0.145000 | 0.272000 | 0.244000 | 0.328190 | 0.323122 | 87.57 |
| 1m | duckdb-parquet | q41 | 0.045000 | 0.046000 | 0.054000 | 0.135434 | 0.077997 | 38.41 |
| 1m | duckdb-parquet | q42 | 0.042000 | 0.048000 | 0.033000 | 0.144626 | 0.106600 | 35.80 |
| 1m | duckdb-parquet | q43 | 0.018000 | 0.044000 | 0.020000 | 0.145120 | 0.088363 | 33.98 |
| 1m | rudb-parquet | q1 | 0.015315 | 0.011624 | 0.008674 | 0.044952 | 0.009264 | 12.76 |
| 1m | rudb-parquet | q2 | 0.038368 | 0.040013 | 0.033165 | 0.067776 | 0.023190 | 16.61 |
| 1m | rudb-parquet | q3 | 0.011007 | 0.022130 | 0.031974 | 0.030806 | 0.028784 | 20.29 |
| 1m | rudb-parquet | q4 | 0.033773 | 0.018464 | 0.015562 | 0.029032 | 0.022834 | 30.59 |
| 1m | rudb-parquet | q5 | 0.023037 | 0.031498 | 0.044621 | 0.040378 | 0.069137 | 33.25 |
| 1m | rudb-parquet | q6 | 0.219135 | 0.136146 | 0.069260 | 0.157062 | 0.229139 | 45.58 |
| 1m | rudb-parquet | q7 | 0.012327 | 0.013333 | 0.005851 | 0.019072 | 0.012335 | 16.26 |
| 1m | rudb-parquet | q8 | 0.016384 | 0.019233 | 0.020947 | 0.036223 | 0.022906 | 17.37 |
| 1m | rudb-parquet | q9 | 0.033394 | 0.077400 | 0.011006 | 0.086539 | 0.147412 | 45.79 |
| 1m | rudb-parquet | q10 | 0.047430 | 0.080033 | 0.050943 | 0.111819 | 0.158299 | 54.72 |
| 1m | rudb-parquet | q11 | 0.036420 | 0.048012 | 0.027890 | 0.059072 | 0.069105 | 32.95 |
| 1m | rudb-parquet | q12 | 0.025366 | 0.029017 | 0.045420 | 0.033023 | 0.054721 | 35.46 |
| 1m | rudb-parquet | q13 | 0.063296 | 0.079837 | 0.044716 | 0.086318 | 0.148768 | 45.98 |
| 1m | rudb-parquet | q14 | 0.149350 | 0.166169 | 0.288587 | 0.219556 | 0.267549 | 69.45 |
| 1m | rudb-parquet | q15 | 0.110787 | 0.127995 | 0.041531 | 0.147784 | 0.221328 | 49.58 |
| 1m | rudb-parquet | q16 | 0.028854 | 0.032154 | 0.048453 | 0.040476 | 0.053702 | 38.52 |
| 1m | rudb-parquet | q17 | 0.310440 | 0.282836 | 0.157931 | 0.309275 | 0.521460 | 108.23 |
| 1m | rudb-parquet | q18 | 0.121228 | 0.080252 | 0.072908 | 0.095250 | 0.139538 | 39.21 |
| 1m | rudb-parquet | q19 | 0.250647 | 0.320889 | 0.221472 | 0.349356 | 0.541331 | 130.90 |
| 1m | rudb-parquet | q20 | 0.018520 | 0.014661 | 0.005701 | 0.021466 | 0.016852 | 28.28 |
| 1m | rudb-parquet | q21 | 0.188579 | 0.506011 | 0.355793 | 0.525284 | 0.760024 | 125.89 |
| 1m | rudb-parquet | q22 | 0.302578 | 0.417163 | 0.258070 | 0.430584 | 0.900047 | 104.64 |
| 1m | rudb-parquet | q23 | 0.669049 | 0.548583 | 0.317576 | 0.578706 | 1.438200 | 203.96 |
| 1m | rudb-parquet | q24 | 0.935089 | 0.757242 | 0.261145 | 0.824802 | 2.554937 | 294.38 |
| 1m | rudb-parquet | q25 | 0.166093 | 0.111123 | 0.129655 | 0.128318 | 0.209397 | 40.38 |
| 1m | rudb-parquet | q26 | 0.046118 | 0.096044 | 0.030181 | 0.122174 | 0.143743 | 27.61 |
| 1m | rudb-parquet | q27 | 0.150986 | 0.140106 | 0.102289 | 0.165762 | 0.204178 | 39.61 |
| 1m | rudb-parquet | q28 | 0.190203 | 0.456986 | 0.276532 | 0.492481 | 0.955738 | 119.73 |
| 1m | rudb-parquet | q29 | 0.551558 | 0.719155 | 0.367841 | 0.767091 | 1.188731 | 146.90 |
| 1m | rudb-parquet | q30 | 0.021164 | 0.028648 | 0.058170 | 0.040070 | 0.027505 | 17.98 |
| 1m | rudb-parquet | q31 | 0.073669 | 0.129933 | 0.061233 | 0.144419 | 0.227368 | 47.38 |
| 1m | rudb-parquet | q32 | 0.236354 | 0.111146 | 0.059692 | 0.126860 | 0.226177 | 54.98 |
| 1m | rudb-parquet | q33 | 0.108068 | 0.109283 | 0.054180 | 0.152338 | 0.164043 | 56.48 |
| 1m | rudb-parquet | q34 | 0.830533 | 0.756154 | 0.208329 | 0.800920 | 1.411752 | 228.23 |
| 1m | rudb-parquet | q35 | 0.476688 | 0.779299 | 0.389163 | 0.806800 | 1.579435 | 231.57 |
| 1m | rudb-parquet | q36 | 0.034099 | 0.066629 | 0.043067 | 0.073575 | 0.076035 | 32.95 |
| 1m | rudb-parquet | q37 | 0.122721 | 0.321797 | 0.412028 | 0.339760 | 0.252774 | 56.05 |
| 1m | rudb-parquet | q38 | 0.348322 | 0.408315 | 0.200624 | 0.443329 | 0.403241 | 50.12 |
| 1m | rudb-parquet | q39 | 0.362481 | 0.226585 | 0.320075 | 0.244099 | 0.217956 | 55.85 |
| 1m | rudb-parquet | q40 | 0.604401 | 0.481626 | 0.322650 | 0.499563 | 0.572418 | 78.49 |
| 1m | rudb-parquet | q41 | 0.030187 | 0.030403 | 0.048267 | 0.048420 | 0.035308 | 28.78 |
| 1m | rudb-parquet | q42 | 0.031828 | 0.031547 | 0.117511 | 0.052348 | 0.035946 | 26.04 |
| 1m | rudb-parquet | q43 | 0.060724 | 0.028359 | 0.054792 | 0.044483 | 0.021499 | 22.30 |
| 10m | duckdb-native | q1 | 0.002000 | 0.002000 | 0.000000 | 0.100000 | 0.057246 | 27.59 |
| 10m | duckdb-native | q2 | 0.064000 | 0.038000 | 0.021000 | 0.084567 | 0.059949 | 51.02 |
| 10m | duckdb-native | q3 | 0.074000 | 0.068000 | 0.041000 | 0.149517 | 0.157693 | 73.55 |
| 10m | duckdb-native | q4 | 0.177000 | 0.077000 | 0.098000 | 0.158047 | 0.239629 | 98.51 |
| 10m | duckdb-native | q5 | 0.314000 | 0.321000 | 0.042000 | 0.413017 | 0.862621 | 220.64 |
| 10m | duckdb-native | q6 | 0.298000 | 0.281000 | 0.142000 | 0.353860 | 0.790714 | 220.10 |
| 10m | duckdb-native | q7 | 0.010000 | 0.008000 | 0.011000 | 0.059841 | 0.027333 | 29.34 |
| 10m | duckdb-native | q8 | 0.083000 | 0.038000 | 0.014000 | 0.129478 | 0.115670 | 53.06 |
| 10m | duckdb-native | q9 | 0.639000 | 0.477000 | 0.173000 | 0.563670 | 1.334160 | 271.69 |
| 10m | duckdb-native | q10 | 0.674000 | 0.693000 | 0.201000 | 0.992587 | 2.251097 | 320.27 |
| 10m | duckdb-native | q11 | 0.311000 | 0.235000 | 0.152000 | 0.312068 | 0.430118 | 157.99 |
| 10m | duckdb-native | q12 | 0.465000 | 0.253000 | 0.102000 | 0.379917 | 0.469377 | 167.31 |
| 10m | duckdb-native | q13 | 0.236000 | 0.282000 | 0.101000 | 0.353930 | 0.680659 | 227.30 |
| 10m | duckdb-native | q14 | 0.565000 | 0.722000 | 0.675000 | 0.916445 | 1.703695 | 394.87 |
| 10m | duckdb-native | q15 | 0.446000 | 0.369000 | 0.443000 | 0.472608 | 0.780426 | 245.65 |
| 10m | duckdb-native | q16 | 0.372000 | 0.360000 | 0.176000 | 0.500994 | 1.019775 | 272.04 |
| 10m | duckdb-native | q17 | 1.041000 | 1.068000 | 0.308000 | 1.320055 | 2.422120 | 610.81 |
| 10m | duckdb-native | q18 | 1.603000 | 0.906000 | 0.407000 | 1.155670 | 1.886303 | 635.81 |
| 10m | duckdb-native | q19 | 1.113000 | 1.092000 | 0.245000 | 1.423503 | 2.959806 | 938.97 |
| 10m | duckdb-native | q20 | 0.105000 | 0.098000 | 0.027000 | 0.185910 | 0.206837 | 96.83 |
| 10m | duckdb-native | q21 | 2.019000 | 1.017000 | 0.213000 | 1.196124 | 1.830978 | 624.18 |
| 10m | duckdb-native | q22 | 1.175000 | 2.044000 | 1.567000 | 2.449052 | 2.946975 | 685.08 |
| 10m | duckdb-native | q23 | 2.089000 | 1.256000 | 0.943000 | 1.395556 | 2.168541 | 983.22 |
| 10m | duckdb-native | q24 | 1.056000 | 0.898000 | 0.447000 | 1.089097 | 1.707435 | 561.95 |
| 10m | duckdb-native | q25 | 0.067000 | 0.111000 | 0.053000 | 0.165778 | 0.160003 | 65.22 |
| 10m | duckdb-native | q26 | 0.109000 | 0.125000 | 0.047000 | 0.215019 | 0.318555 | 89.43 |
| 10m | duckdb-native | q27 | 0.044000 | 0.062000 | 0.096000 | 0.144800 | 0.129748 | 63.78 |
| 10m | duckdb-native | q28 | 0.631000 | 0.961000 | 0.464000 | 1.132566 | 1.504576 | 643.93 |
| 10m | duckdb-native | q29 | 6.146000 | 4.718000 | 2.031000 | 4.863744 | 13.038922 | 764.10 |
| 10m | duckdb-native | q30 | 0.071000 | 0.066000 | 0.053000 | 0.139745 | 0.107214 | 54.36 |
| 10m | duckdb-native | q31 | 0.592000 | 0.359000 | 0.105000 | 0.448101 | 0.707692 | 267.69 |
| 10m | duckdb-native | q32 | 0.452000 | 0.510000 | 0.424000 | 0.609009 | 0.888288 | 367.66 |
| 10m | duckdb-native | q33 | 0.853000 | 1.020000 | 0.303000 | 1.181958 | 2.665619 | 867.29 |
| 10m | duckdb-native | q34 | 2.851000 | 2.210000 | 0.994000 | 2.608792 | 5.010644 | 1468.58 |
| 10m | duckdb-native | q35 | 3.914000 | 1.484000 | 0.795000 | 1.798358 | 4.457594 | 1497.30 |
| 10m | duckdb-native | q36 | 0.213000 | 0.279000 | 0.073000 | 0.319191 | 1.414207 | 353.38 |
| 10m | duckdb-native | q37 | 0.024000 | 0.021000 | 0.012000 | 0.040636 | 0.063473 | 53.74 |
| 10m | duckdb-native | q38 | 0.014000 | 0.016000 | 0.029000 | 0.041445 | 0.049908 | 42.80 |
| 10m | duckdb-native | q39 | 0.040000 | 0.010000 | 0.003000 | 0.028500 | 0.041418 | 42.50 |
| 10m | duckdb-native | q40 | 0.027000 | 0.037000 | 0.022000 | 0.076048 | 0.126063 | 74.84 |
| 10m | duckdb-native | q41 | 0.012000 | 0.007000 | 0.003000 | 0.027052 | 0.035663 | 41.38 |
| 10m | duckdb-native | q42 | 0.017000 | 0.013000 | 0.012000 | 0.037967 | 0.046128 | 39.74 |
| 10m | duckdb-native | q43 | 0.023000 | 0.008000 | 0.006000 | 0.059852 | 0.068229 | 36.92 |
| 10m | rudb-native | q1 | 0.036373 | 0.000743 | 0.001308 | 0.017182 | 0.005916 | 12.42 |
| 10m | rudb-native | q2 | 0.002882 | 0.000821 | 0.001499 | 0.019108 | 0.007654 | 12.63 |
| 10m | rudb-native | q3 | 0.027549 | 0.000802 | 0.000109 | 0.008640 | 0.006517 | 12.59 |
| 10m | rudb-native | q4 | 0.001999 | 0.000799 | 0.000128 | 0.010756 | 0.006120 | 12.59 |
| 10m | rudb-native | q5 | 0.000698 | 0.000732 | 0.003461 | 0.020843 | 0.013798 | 12.45 |
| 10m | rudb-native | q6 | 0.000688 | 0.000744 | 0.000081 | 0.007726 | 0.006369 | 12.58 |
| 10m | rudb-native | q7 | 0.002505 | 0.000842 | 0.007947 | 0.015363 | 0.007320 | 12.63 |
| 10m | rudb-native | q8 | 0.016117 | 0.011645 | 0.034281 | 0.027542 | 0.016146 | 29.45 |
| 10m | rudb-native | q9 | 0.178140 | 0.203642 | 0.066741 | 0.231980 | 0.489797 | 134.66 |
| 10m | rudb-native | q10 | 0.338418 | 0.222358 | 0.156142 | 0.259981 | 0.569926 | 161.64 |
| 10m | rudb-native | q11 | 0.031325 | 0.038950 | 0.018021 | 0.067179 | 0.088861 | 53.66 |
| 10m | rudb-native | q12 | 0.063165 | 0.069032 | 0.041867 | 0.081760 | 0.205182 | 56.12 |
| 10m | rudb-native | q13 | 0.076550 | 0.046501 | 0.010737 | 0.066228 | 0.070641 | 36.70 |
| 10m | rudb-native | q14 | 0.110724 | 0.099580 | 0.020469 | 0.148352 | 0.283537 | 74.29 |
| 10m | rudb-native | q15 | 0.146268 | 0.131707 | 0.054037 | 0.164031 | 0.213853 | 63.18 |
| 10m | rudb-native | q16 | 0.001154 | 0.002079 | 0.002528 | 0.014772 | 0.008260 | 13.40 |
| 10m | rudb-native | q17 | 0.178846 | 0.255430 | 0.149584 | 0.303487 | 0.663960 | 200.38 |
| 10m | rudb-native | q18 | 0.013832 | 0.017983 | 0.029235 | 0.036578 | 0.019073 | 20.16 |
| 10m | rudb-native | q19 | 0.335649 | 0.383007 | 0.295763 | 0.481825 | 1.107854 | 276.22 |
| 10m | rudb-native | q20 | 0.010825 | 0.010815 | 0.007177 | 0.030482 | 0.015388 | 21.02 |
| 10m | rudb-native | q21 | 0.446872 | 0.357813 | 0.114103 | 0.399943 | 0.714587 | 100.00 |
| 10m | rudb-native | q22 | 0.313959 | 0.429207 | 1.009185 | 0.540252 | 0.805404 | 109.13 |
| 10m | rudb-native | q23 | 0.644433 | 0.399827 | 0.113434 | 0.435946 | 0.496214 | 127.57 |
| 10m | rudb-native | q24 | 0.360520 | 0.358187 | 0.218086 | 0.414931 | 0.483069 | 129.77 |
| 10m | rudb-native | q25 | 0.020754 | 0.029823 | 0.021347 | 0.071005 | 0.052083 | 24.06 |
| 10m | rudb-native | q26 | 0.102419 | 0.073165 | 0.054589 | 0.096799 | 0.106979 | 30.27 |
| 10m | rudb-native | q27 | 0.025360 | 0.036269 | 0.021718 | 0.059832 | 0.025638 | 27.55 |
| 10m | rudb-native | q28 | 0.088455 | 0.074166 | 0.032445 | 0.098369 | 0.136165 | 57.91 |
| 10m | rudb-native | q29 | 1.561734 | 1.355124 | 0.930894 | 1.398738 | 2.932322 | 275.55 |
| 10m | rudb-native | q30 | 0.007405 | 0.013353 | 0.005555 | 0.029039 | 0.013468 | 13.89 |
| 10m | rudb-native | q31 | 0.110101 | 0.118796 | 0.058455 | 0.173369 | 0.298169 | 69.25 |
| 10m | rudb-native | q32 | 0.002718 | 0.003788 | 0.012141 | 0.032685 | 0.032400 | 16.16 |
| 10m | rudb-native | q33 | 0.001337 | 0.001255 | 0.000193 | 0.018390 | 0.017002 | 13.89 |
| 10m | rudb-native | q34 | 0.001280 | 0.001176 | 0.002510 | 0.011529 | 0.007934 | 13.61 |
| 10m | rudb-native | q35 | 0.360459 | 0.064255 | 0.035394 | 0.091847 | 0.151030 | 80.61 |
| 10m | rudb-native | q36 | 0.001967 | 0.001119 | 0.001027 | 0.007975 | 0.007928 | 13.98 |
| 10m | rudb-native | q37 | 0.026268 | 0.009152 | 0.004815 | 0.015427 | 0.019561 | 29.09 |
| 10m | rudb-native | q38 | 0.011659 | 0.009498 | 0.003439 | 0.015006 | 0.015863 | 24.17 |
| 10m | rudb-native | q39 | 0.009691 | 0.008006 | 0.002492 | 0.014748 | 0.016371 | 27.26 |
| 10m | rudb-native | q40 | 0.039220 | 0.027410 | 0.021511 | 0.035028 | 0.057690 | 44.98 |
| 10m | rudb-native | q41 | 0.010623 | 0.002908 | 0.000153 | 0.008284 | 0.010615 | 20.18 |
| 10m | rudb-native | q42 | 0.008553 | 0.007997 | 0.010470 | 0.020010 | 0.022359 | 21.11 |
| 10m | rudb-native | q43 | 0.014116 | 0.013198 | 0.011194 | 0.018525 | 0.024058 | 20.96 |
| 10m | duckdb-parquet | q1 | 0.244000 | 0.230000 | 0.101000 | 0.864328 | 0.823000 | 150.52 |
| 10m | duckdb-parquet | q2 | 1.051000 | 0.569000 | 0.532000 | 1.013725 | 0.878910 | 159.36 |
| 10m | duckdb-parquet | q3 | 0.375000 | 0.514000 | 0.276000 | 0.827641 | 0.894576 | 174.82 |
| 10m | duckdb-parquet | q4 | 0.276000 | 0.340000 | 0.694000 | 0.979287 | 0.921710 | 159.34 |
| 10m | duckdb-parquet | q5 | 1.048000 | 0.907000 | 0.641000 | 1.594999 | 1.820428 | 279.97 |
| 10m | duckdb-parquet | q6 | 0.982000 | 0.551000 | 0.383000 | 0.782866 | 1.114299 | 312.91 |
| 10m | duckdb-parquet | q7 | 0.718000 | 0.424000 | 0.125000 | 0.801480 | 0.756928 | 163.50 |
| 10m | duckdb-parquet | q8 | 0.801000 | 0.488000 | 0.365000 | 1.038911 | 1.023572 | 161.15 |
| 10m | duckdb-parquet | q9 | 1.174000 | 1.116000 | 0.886000 | 1.584329 | 1.649999 | 318.99 |
| 10m | duckdb-parquet | q10 | 1.560000 | 1.178000 | 0.511000 | 1.464819 | 2.421664 | 377.32 |
| 10m | duckdb-parquet | q11 | 0.602000 | 0.474000 | 0.267000 | 0.873097 | 1.039167 | 210.57 |
| 10m | duckdb-parquet | q12 | 0.620000 | 0.813000 | 0.268000 | 1.242353 | 1.392881 | 224.43 |
| 10m | duckdb-parquet | q13 | 0.500000 | 0.801000 | 0.633000 | 1.319366 | 1.639353 | 314.29 |
| 10m | duckdb-parquet | q14 | 0.755000 | 0.786000 | 1.697000 | 1.267178 | 1.986255 | 417.06 |
| 10m | duckdb-parquet | q15 | 1.268000 | 1.159000 | 1.253000 | 1.624844 | 1.546483 | 332.56 |
| 10m | duckdb-parquet | q16 | 0.798000 | 0.624000 | 0.298000 | 1.269573 | 1.668273 | 367.27 |
| 10m | duckdb-parquet | q17 | 0.920000 | 1.997000 | 1.284000 | 3.060075 | 3.785998 | 655.36 |
| 10m | duckdb-parquet | q18 | 1.020000 | 1.286000 | 0.448000 | 1.803856 | 2.319015 | 651.07 |
| 10m | duckdb-parquet | q19 | 2.490000 | 1.783000 | 1.178000 | 2.633156 | 4.360237 | 944.19 |
| 10m | duckdb-parquet | q20 | 0.347000 | 0.399000 | 0.213000 | 0.755622 | 0.722992 | 156.48 |
| 10m | duckdb-parquet | q21 | 1.419000 | 0.792000 | 0.703000 | 1.367888 | 2.104188 | 216.73 |
| 10m | duckdb-parquet | q22 | 0.891000 | 1.110000 | 0.494000 | 1.603673 | 2.117384 | 252.27 |
| 10m | duckdb-parquet | q23 | 0.982000 | 1.113000 | 0.385000 | 1.400794 | 2.658231 | 237.34 |
| 10m | duckdb-parquet | q24 | 1.258000 | 0.979000 | 0.556000 | 1.328510 | 2.882799 | 351.40 |
| 10m | duckdb-parquet | q25 | 0.858000 | 0.795000 | 0.289000 | 1.253453 | 1.573218 | 290.88 |
| 10m | duckdb-parquet | q26 | 0.374000 | 0.570000 | 0.387000 | 1.244657 | 1.343385 | 171.32 |
| 10m | duckdb-parquet | q27 | 0.855000 | 0.605000 | 1.003000 | 0.917392 | 1.118030 | 179.24 |
| 10m | duckdb-parquet | q28 | 0.960000 | 0.747000 | 0.347000 | 1.464772 | 1.944945 | 260.45 |
| 10m | duckdb-parquet | q29 | 4.064000 | 3.393000 | 0.917000 | 3.764473 | 12.644142 | 519.92 |
| 10m | duckdb-parquet | q30 | 0.409000 | 0.572000 | 0.377000 | 1.045044 | 0.979998 | 165.64 |
| 10m | duckdb-parquet | q31 | 0.983000 | 1.173000 | 0.973000 | 1.490841 | 2.128830 | 293.83 |
| 10m | duckdb-parquet | q32 | 1.347000 | 1.091000 | 0.328000 | 1.738305 | 1.929046 | 317.22 |
| 10m | duckdb-parquet | q33 | 1.341000 | 1.421000 | 0.751000 | 1.754540 | 3.521925 | 859.48 |
| 10m | duckdb-parquet | q34 | 2.146000 | 1.906000 | 0.712000 | 2.432147 | 4.768650 | 1155.99 |
| 10m | duckdb-parquet | q35 | 2.121000 | 1.155000 | 1.100000 | 1.541210 | 5.154434 | 1209.98 |
| 10m | duckdb-parquet | q36 | 0.428000 | 0.454000 | 0.309000 | 0.685332 | 1.777018 | 440.23 |
| 10m | duckdb-parquet | q37 | 0.309000 | 0.295000 | 0.105000 | 0.497051 | 0.636850 | 185.56 |
| 10m | duckdb-parquet | q38 | 0.252000 | 0.266000 | 0.021000 | 0.475251 | 0.523443 | 172.78 |
| 10m | duckdb-parquet | q39 | 0.254000 | 0.285000 | 0.020000 | 0.481757 | 0.599201 | 175.30 |
| 10m | duckdb-parquet | q40 | 0.292000 | 0.269000 | 0.064000 | 0.481055 | 0.584486 | 200.84 |
| 10m | duckdb-parquet | q41 | 0.262000 | 0.294000 | 0.171000 | 0.512795 | 0.624089 | 165.39 |
| 10m | duckdb-parquet | q42 | 0.275000 | 0.285000 | 0.128000 | 0.486055 | 0.561238 | 167.06 |
| 10m | duckdb-parquet | q43 | 0.292000 | 0.290000 | 0.069000 | 0.720588 | 0.876643 | 167.03 |
| 10m | rudb-parquet | q1 | 0.446668 | 0.391884 | 0.887676 | 0.400613 | 0.244734 | 118.12 |
| 10m | rudb-parquet | q2 | 0.950471 | 0.436283 | 0.437839 | 0.527057 | 0.412718 | 118.11 |
| 10m | rudb-parquet | q3 | 0.788864 | 0.263786 | 0.100186 | 0.296933 | 0.299545 | 118.16 |
| 10m | rudb-parquet | q4 | 0.237626 | 0.879123 | 0.629622 | 0.914239 | 0.737720 | 118.15 |
| 10m | rudb-parquet | q5 | 1.030820 | 0.499127 | 0.237607 | 0.567231 | 0.811038 | 234.21 |
| 10m | rudb-parquet | q6 | 1.478226 | 1.042940 | 0.503848 | 1.080631 | 1.625136 | 263.14 |
| 10m | rudb-parquet | q7 | 0.357390 | 0.229774 | 0.139198 | 0.246033 | 0.233365 | 118.12 |
| 10m | rudb-parquet | q8 | 0.624352 | 0.405600 | 0.176827 | 0.443291 | 0.319965 | 118.15 |
| 10m | rudb-parquet | q9 | 0.440114 | 0.374145 | 0.159552 | 0.423212 | 0.720507 | 215.99 |
| 10m | rudb-parquet | q10 | 1.372829 | 0.647116 | 0.485920 | 0.722720 | 1.059438 | 220.58 |
| 10m | rudb-parquet | q11 | 0.534520 | 0.348199 | 0.181546 | 0.386776 | 0.538833 | 128.70 |
| 10m | rudb-parquet | q12 | 0.409046 | 0.440791 | 0.234685 | 0.506851 | 0.688440 | 126.36 |
| 10m | rudb-parquet | q13 | 0.539664 | 1.090958 | 0.269164 | 1.132174 | 1.442469 | 239.89 |
| 10m | rudb-parquet | q14 | 1.830889 | 1.472247 | 0.996056 | 1.519099 | 2.266157 | 323.61 |
| 10m | rudb-parquet | q15 | 3.212588 | 1.576824 | 1.158270 | 1.643492 | 1.963786 | 248.36 |
| 10m | rudb-parquet | q16 | 0.550176 | 0.495715 | 0.607610 | 0.547503 | 0.644300 | 213.93 |
| 10m | rudb-parquet | q17 | 3.265326 | 3.147364 | 1.771244 | 3.413284 | 4.109620 | 605.02 |
| 10m | rudb-parquet | q18 | 0.588278 | 0.878391 | 0.692031 | 0.919435 | 0.878167 | 118.14 |
| 10m | rudb-parquet | q19 | 4.312373 | 3.498281 | 0.879588 | 3.619459 | 4.811628 | 1094.44 |
| 10m | rudb-parquet | q20 | 0.538017 | 0.357575 | 0.096317 | 0.374005 | 0.230717 | 118.13 |
| 10m | rudb-parquet | q21 | 1.005511 | 0.807446 | 0.728337 | 0.851251 | 2.076989 | 186.03 |
| 10m | rudb-parquet | q22 | 1.187385 | 0.964186 | 0.889209 | 0.999695 | 2.005007 | 164.31 |
| 10m | rudb-parquet | q23 | 3.331737 | 1.436881 | 0.589666 | 1.461498 | 4.232938 | 169.23 |
| 10m | rudb-parquet | q24 | 1.738912 | 1.005283 | 0.662899 | 1.072733 | 2.700908 | 241.15 |
| 10m | rudb-parquet | q25 | 0.677312 | 0.456498 | 0.197549 | 0.472629 | 0.738574 | 123.28 |
| 10m | rudb-parquet | q26 | 0.348271 | 0.437379 | 0.317637 | 0.465784 | 0.646222 | 130.61 |
| 10m | rudb-parquet | q27 | 0.497434 | 0.510319 | 0.201033 | 0.543394 | 0.803554 | 122.67 |
| 10m | rudb-parquet | q28 | 0.997116 | 0.854894 | 0.470593 | 0.884050 | 1.812464 | 170.89 |
| 10m | rudb-parquet | q29 | 2.456604 | 2.598441 | 0.544496 | 2.881984 | 5.433496 | 461.56 |
| 10m | rudb-parquet | q30 | 0.523096 | 0.457377 | 1.230226 | 0.486246 | 0.370039 | 118.44 |
| 10m | rudb-parquet | q31 | 0.930867 | 0.633638 | 0.292804 | 0.670600 | 0.992139 | 152.42 |
| 10m | rudb-parquet | q32 | 0.730711 | 0.622748 | 0.585956 | 0.676506 | 1.093539 | 153.39 |
| 10m | rudb-parquet | q33 | 1.043652 | 0.488520 | 0.155498 | 0.565039 | 0.815114 | 290.14 |
| 10m | rudb-parquet | q34 | 3.166022 | 3.414202 | 0.919918 | 3.545905 | 6.497069 | 993.25 |
| 10m | rudb-parquet | q35 | 3.917678 | 2.385747 | 1.580002 | 2.442041 | 5.940434 | 972.57 |
| 10m | rudb-parquet | q36 | 0.234567 | 0.363521 | 0.139624 | 0.382570 | 0.613282 | 218.23 |
| 10m | rudb-parquet | q37 | 0.204680 | 0.189204 | 0.107112 | 0.191662 | 0.228908 | 135.08 |
| 10m | rudb-parquet | q38 | 0.175764 | 0.180845 | 0.061669 | 0.182891 | 0.199948 | 119.59 |
| 10m | rudb-parquet | q39 | 0.220076 | 0.170807 | 0.005522 | 0.173467 | 0.186880 | 119.99 |
| 10m | rudb-parquet | q40 | 0.207352 | 0.209095 | 0.070782 | 0.221281 | 0.316544 | 163.91 |
| 10m | rudb-parquet | q41 | 0.173063 | 0.174852 | 0.006146 | 0.176877 | 0.181546 | 118.40 |
| 10m | rudb-parquet | q42 | 0.245287 | 0.193392 | 0.088182 | 0.196144 | 0.198881 | 118.41 |
| 10m | rudb-parquet | q43 | 0.262675 | 0.230595 | 0.120497 | 0.248790 | 0.255658 | 118.14 |
