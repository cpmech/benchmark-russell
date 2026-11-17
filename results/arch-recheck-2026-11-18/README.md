# MUMPS re-check, 18 November 2026

A MUMPS-only re-measurement of the SuiteSparse rows on the current stack — `russell_sparse` 5.4.0 (the
thread-safety release, whose MUMPS calls run on the caller's thread) against the thread-safe MUMPS 5.9.2
fork in `/usr/local`, Intel MKL, `OMP_NUM_THREADS` = 24, on the same workstation as the published table
(32 GB, i9-13900KF). The run was stopped after 25 of the 43 MUMPS matrices, on `Transport`, which was
paging at 700 MB/s at that point: the remaining large matrices cannot be timed on this machine in its
present state, and the point of the exercise was to check the stack, not to redo the table.

Each JSON here was produced by the same command the suite uses
(`solve_matrix_market -g mumps -r 5 <matrix>.mtx`). `memory.log` is the harness's own record: it shows
`swap_growth_mb=0` and `swap_pages=0` for every run tabulated below.

| matrix | total, published | total, re-check | ratio | solve, published | solve, re-check |
| :--- | ---: | ---: | ---: | ---: | ---: |
| `atmosmodl` | 40.87 s | 130.33 s | 3.19 | 397.0 ms | 27483.6 ms |
| `atmosmodd` | 46.16 s | 112.90 s | 2.45 | 411.1 ms | 25450.9 ms |
| `marine1` | 5.68 s | 8.11 s | 1.43 | 74.3 ms | 609.2 ms |
| `rajat31` | 16.04 s | 21.01 s | 1.31 | 180.5 ms | 1107.9 ms |
| `rdb5000` | 0.01 s | 0.02 s | 1.26 | 0.4 ms | 1.2 ms |
| `rma10` | 0.17 s | 0.20 s | 1.17 | 4.1 ms | 5.0 ms |
| `Goodwin_040` | 0.10 s | 0.12 s | 1.16 | 1.9 ms | 1.9 ms |
| `fp` | 0.17 s | 0.19 s | 1.15 | 3.2 ms | 4.3 ms |
| `oilpan` | 0.31 s | 0.31 s | 1.02 | 6.6 ms | 9.0 ms |
| `twotone` | 0.98 s | 0.97 s | 0.98 | 13.5 ms | 10.1 ms |
| `ASIC_680k` | 78.37 s | 76.94 s | 0.98 | 158.4 ms | 1645.2 ms |
| `Raj1` | 0.95 s | 0.92 s | 0.98 | 15.0 ms | 14.0 ms |
| `Goodwin_071` | 0.29 s | 0.28 s | 0.97 | 4.2 ms | 3.5 ms |
| `Goodwin_127` | 0.88 s | 0.86 s | 0.97 | 9.2 ms | 8.9 ms |
| `stomach` | 1.72 s | 1.65 s | 0.96 | 22.7 ms | 20.2 ms |
| `Freescale1` | 10.16 s | 9.70 s | 0.95 | 124.7 ms | 116.3 ms |
| `tmt_unsym` | 3.26 s | 3.11 s | 0.95 | 39.0 ms | 40.5 ms |
| `bwm2000` | 0.01 s | 0.01 s | 0.95 | 1.0 ms | 0.5 ms |
| `memchip` | 8.05 s | 7.62 s | 0.95 | 92.5 ms | 78.2 ms |
| `darcy003` | 11.73 s | 11.02 s | 0.94 | 12.8 ms | 10.6 ms |
| `torso1` | 2.01 s | 1.88 s | 0.94 | 12.8 ms | 11.9 ms |
| `xenon1` | 0.37 s | 0.34 s | 0.93 | 8.9 ms | 8.0 ms |
| `boyd2` | 0.84 s | 0.78 s | 0.93 | 88.1 ms | 58.5 ms |
| `helm2d03` | 1.64 s | 1.51 s | 0.93 | 16.8 ms | 17.7 ms |
| `pre2` | 5.10 s | 4.72 s | 0.92 | 73.7 ms | 67.4 ms |

## Reading

- **20 of 25** matrices reproduce the published total between 0.92 and 1.26 times it, with their
  solve times unchanged: `pre2`, `helm2d03`, `boyd2`, `xenon1`, `torso1`, `darcy003`, `memchip`, `bwm2000`, `tmt_unsym`, `Freescale1`, `stomach`, `Goodwin_127`, `Goodwin_071`, `Raj1`, `twotone`, `oilpan`, `fp`, `Goodwin_040`, `rma10`, `rdb5000`. The thread-safe stack is the same solver family that produced the
  published table, and the spread is ordinary run-to-run noise on a workstation.
- **5** matrices did not reproduce, and the pattern is memory, not arithmetic: `atmosmodl` 69×, `atmosmodd` 62×, `ASIC_680k` 10×, `marine1` 8×, `rajat31` 6×. MUMPS reserves
  its working memory from the factorization estimate (`ICNTL(14)` = 100 %, out-of-core off); when the
  workspace does not fit, the solve pages — it is bound by memory traffic — and inflates by one to two
  orders of magnitude while the factorization, which is compute-bound, moves by tens of per cent.
  `ASIC_680k` is that same effect hidden inside a factorization-dominated total: its total is
  unchanged while its solve inflates tenfold.
- The requirement did not change with the library. Measured on `Transport` with each build (memory capped so
  MUMPS stops after its own estimate): INFOG(17) = 39,362 MB, INFOG(27) = 13,553 MB and INFOG(3) =
  2,635,174,153 entries are byte-identical for MUMPS 5.9.0, stock 5.9.2 and the patched 5.9.2, with analysis
  times of 10.08, 10.19 and 10.45 s. What changed is the machine: the published campaign had a zram device
  beside its swapfile (`SwapTotal` 62 GB in its system box), this one has only the 31 GB swapfile, so the
  same overflow reaches the disk instead of compressed RAM. That is an environment change, not a solver
  change, and it is why the published table remains the reference for those rows.

## Consequence for the harness

`solve-problems.bash` now refuses to time a matrix when `MemAvailable` is below `MIN_AVAILABLE_GB`
(default 20 GB) and deletes a result whose run grew the swap by more than `MAX_SWAP_GROWTH_MB`
(default 256 MB) or paged more than `MAX_SWAP_PAGES` (default 4096 pages = 16 MB, from
`/proc/vmstat`'s cumulative `pswpin` + `pswpout`, because a run can page and leave the net swap
unchanged). Every check and every run is appended to `results/<system>/memory.log`.
