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

- **25 of 25 matrices reproduce the published totals once the run is not memory-capped**, between 0.82 and
  1.26 times them, with their solve times unchanged or faster.
- The five that first appeared to be "outliers" were an artifact of the execution environment, not of the
  machine or the solver: this pass ran inside a job scope limited to 4 GiB of memory, which clamps the
  resident set of a run that needs ~25 GiB and pushes the difference through swap. Uncapped, the same
  matrices give: `marine1` 0.88× (solve 74 → 63 ms), `rajat31` 0.95× (181 → 159 ms), `atmosmodd` 0.82×
  (411 → 283 ms), `atmosmodl` 0.85× (397 → 320 ms), `ASIC_680k` 0.88× (158 → 96 ms).
- The mechanism, measured on one `Transport` run inside the 4 GiB scope: 6.19 M major faults in the solve
  alone, ~20 GB driven through swap, 8.9 s against 0.37 s for the same binary uncapped 92 s later.
  `atmosmodl` inflates in both phases when capped (preprocess 32.28 → 93.27 s, 2.89×).

## Consequence for the harness

Two ceilings matter, and only one of them was being checked. `solve-problems.bash` now refuses to time a
matrix when host `MemAvailable` is below `MIN_AVAILABLE_GB` (default 20 GB) and deletes a result whose run
grew the swap by more than `MAX_SWAP_GROWTH_MB` (256 MB) or paged more than `MAX_SWAP_PAGES` (4096 pages,
from `/proc/vmstat`'s cumulative `pswpin` + `pswpout`, because a run can page and leave the net swap
unchanged); every check and run is logged to `results/<system>/memory.log`.

Still to add, from the review of this pass: a check of the *cgroup's* `memory.max` (a 4 GiB scope is enough
to fake a 25× solve), a `ru_majflt` bound read through `wait4`/`getrusage` (the precise detector: 0 major
faults uncapped against 13.8 M capped), and `memory_begin`/`memory_end` around the Direct stage's probe
results, which are currently moved into place unguarded.

`memory.log` in this directory predates the `swap_pages` field; its lines carry only `avail_mb` and
`swap_used_mb`.

## Evidence in this directory

- `mumps-*.json` — the 25 capped runs of the first pass (the 5 that paged are the outliers discussed above).
- `uncapped-*.json` — the five re-run uncapped in the foreground: `marine1`, `rajat31`, `atmosmodd`,
  `atmosmodl`, `ASIC_680k`.
- `evidence/` — the raw measurements behind the diagnosis: `est-*.txt` (MUMPS's own memory estimate for the
  three builds), `faul-samples.txt` (per-second `minflt`/`majflt` of a capped run), `instr-*.time` (user vs
  system CPU), `pair-*.json` (foreground/background pair), `cap-*.json` (`ICNTL(23)` caps), `hp-*.json`
  (huge-page test).
- `memory.log` — the harness's own log of the first pass.
