# Performance benchmarks using the Rust Scientific Library (Russell) <!-- omit from toc --> 

## Contents <!-- omit from toc --> 

- [Introduction](#introduction)
- [Solvers for large sparse linear systems (`russell_sparse`)](#solvers-for-large-sparse-linear-systems-russell_sparse)
  - [Comparing cuDSS and MUMPS](#comparing-cudss-and-mumps)
- [Comparing KLU, UMFPACK and MUMPS](#comparing-klu-umfpack-and-mumps)
- [System and libraries information](#system-and-libraries-information)



## Introduction

This project contains some performance benchmarks using the [Rust Scientific Library (Russell)](https://github.com/cpmech/russell).

Currently, we present some results involving the sparse linear solvers from [russell_sparse](https://github.com/cpmech/russell/tree/main/russell_sparse).

The computations presented here use all features (`intel_mkl`, `local_sparse`, and `cudss`). Thus, KLU, UMFPACK, and MUMPS are compiled locally with Intel MKL. On the other hand, the Linux binary from [NVIDIA cuDSS](https://developer.nvidia.com/cudss) is employed for the calculations with cuDSS. See the [system and libraries information](#system-and-libraries-information) used in the benchmarks.



## Solvers for large sparse linear systems (`russell_sparse`)

We test the linear system **Ax = b** where **A** is the coefficient matrix, **x** is the solution vector, and **b** is the right-hand side vector. The coefficient matrix is set with matrices from the [SuiteSparse Matrix Collection](https://sparse.tamu.edu). The right-hand side vector is filled with ones, i.e., we study the solution of

**A x = 1**

The relative error is calculated as

```text
RelativeError = max(|A x - 1|) / max(|A| + 1)
```

The tested matrices are:

 1. **bwm2000** (Bai) -- Brusselator wave model in transport interaction of chemical solutions (1992)
 2. **rdb5000** (Bai) -- Reaction-diffusion brusselator model (1994)
 3. **Goodwin_040** (Goodwin) -- Finite element, Navier-Stokes & other transport equations (2018)
 4. **fp** (MKS) -- 2-D Fokker Planck eqn, electron dyn. in external field (2006)
 5. **xenon1** (Ronis) -- Complex zeolite, sodalite crystals (2001)
 6. **twotone** (ATandT) -- Harmonic balance method (2001)
 7. **Raj1** (Rajat) -- Circuit Simulation Problem (2007)
 8. **boyd2** (GHS_indef) -- Optimization problem (2004)
 9. **Goodwin_071** (Goodwin) -- Finite element, Navier-Stokes & other transport equations (2018)
 10. **darcy003** (GHS_indef) -- Discretization using mixed FE of Darcy (2002)
 11. **rma10** (Bova) -- 3D CFD model, Charleston harbor (1997)
 12. **helm2d03** (GHS_indef) -- Helmholtz eq on a unit square (2004)
 13. **stomach** (Norris) -- 3D electro-physical model of a duodenum (2003)
 14. **oilpan** (GHS_psdef) -- Structural problem (2004)
 15. **ASIC_680k** (Sandia) -- Circuit simulation matrix (2006)
 16. **tmt_unsym** (CEMW) -- Electromagnetics problem (2008)
 17. **Goodwin_127** (Goodwin) -- Finite element, Navier-Stokes & other transport equations (2018)
 18. **pre2** (ATandT) -- Harmonic balance method (2001)
 19. **marine1** (Martin) -- Chemical oceanography; a marine nitrogen cycle inverse model (2018)
 20. **torso1** (Norris) -- Finite differences and boundary element, 2D model of torso (2003)
 21. **atmosmodd** (Bourchtein) -- CFD analysis of atmospheric models (2009)
 22. **atmosmodl** (Bourchtein) -- CFD analysis of atmospheric models (2009)
 23. **memchip** (Freescale) -- Circuit simulation problem (2010)
 24. **Freescale1** (Freescale) -- Circuit simulation problem (2008)
 25. **rajat31** (Rajat) -- Circuit simulation problem (2006)
 26. **Transport** (Janna) -- 3D finite element flow and transport (2012)
 27. **inline_1** (GHS_psdef) -- Structural problem, stiffness matrix (2004)
 28. **PFlow_742** (Janna) -- 3D pressure-temperature evolution in porous media (2014)
 29. **Emilia_923** (Janna) -- Geomechanical model for C02 sequestration (2011)
 30. **dielFilterV2real** (Dziekonski) -- FEM in electromagnetics (2011)
 31. **Flan_1565** (Janna) -- Structural problem, 3D model of a steel flange (2011)
 32. **pres-cylin** (Pedroso) -- FEM stiffness matrix of a pressurized cylinder (Tet10 with 1,711,464 DOF). Not from the Collection. From [Pedroso DM (2024) Caveats of three direct linear solvers for finite element analyses](https://onlinelibrary.wiley.com/doi/10.1002/nme.7545).

Notes:

1. Each problem is solved ten times. The reported error is the maximum among runs. The reported computer time is the average without outliers.
2. Total time includes initialization (memory allocation + symbolic factorization), numeric factorization, and solve.
3. Column NNZ (number of non-zeros) corresponds to Pattern Entries reported by the [SuiteSparse Matrix Collection](https://sparse.tamu.edu).
4. Column Sym shows if symmetry information was provided to the solver. An asterisk means positive definite information was also given.
5. oom (out-of-memory) indicates that the symbolic factorization was terminated due to insufficient memory.
6. Bold values highlight significant results, such as large errors or high computation times.
7. The hybrid memory mode is enabled for the pres-cylin matrix and cuDSS.



### Comparing cuDSS and MUMPS

Note that [NVIDIA cuDSS](https://developer.nvidia.com/cudss) is in **Preview Mode** 🚧.

The [results are available in PDF format](pdfs/performance-table.pdf) and also listed below.

The results are given in the table below.

| Matrix       |      Nrow |         NNZ |  Sym  |      cuDSS MA |   cuDSS Time | cuDSS Error |    MUMPS Time | MUMPS Error |
| ------------ | --------: | ----------: | :---: | ------------: | -----------: | ----------: | ------------: | ----------: |
| bwm2000      |     2,000 |       7,996 |  No   |          Auto |      8.633ms |    8.21e-16 |       6.260ms |    7.50e-16 |
| rdb5000      |     5,000 |      29,600 |  No   |          Auto |     30.087ms |    4.06e-13 |       7.442ms |    5.37e-13 |
| Goodwin_040  |    17,922 |     561,677 |  No   |    MaxMinDiag |     71.095ms |    2.82e-12 |     106.490ms |    1.65e-11 |
| fp           |     7,548 |     848,553 |  No   |          Auto |    155.376ms |     4.98e-9 |     164.943ms |    1.16e-20 |
| xenon1       |    48,600 |   1,181,120 |  No   |          Auto |    194.980ms |    1.32e-39 |     308.937ms |    6.74e-40 |
| twotone      |   120,750 |   1,224,224 |  No   |          Auto |    772.871ms |    4.73e-13 |     879.251ms |    1.93e-13 |
| Raj1         |   263,743 |   1,302,464 |  No   |          Auto |    883.834ms |    1.21e-11 |     901.208ms |    3.63e-13 |
| boyd2        |   466,316 |   1,500,397 |  Yes  |          None |    996.540ms |    3.24e-11 |     755.545ms |    8.98e-11 |
| Goodwin_071  |    56,021 |   1,797,934 |  No   |    MaxMinDiag |    174.813ms |    8.65e-12 |     258.507ms |    8.38e-11 |
| darcy003     |   389,874 |   2,101,242 |  Yes  | MaxMinDiagAlt |       2.438s |     5.64e-7 |       10.667s |    1.75e-10 |
| rma10        |    46,835 |   2,374,001 |  No   |          Auto |    241.711ms |    9.04e-16 |     150.619ms |    1.35e-16 |
| helm2d03     |   392,257 |   2,741,935 |  Yes  |          None |       1.239s |    1.68e-10 |        1.440s |    3.82e-10 |
| stomach      |   213,360 |   3,021,648 |  No   |          Auto |       1.280s |    2.71e-15 |        1.579s |    1.49e-15 |
| oilpan       |    73,752 |   3,597,188 | Yes*  |          None |    102.687ms |    4.65e-15 |     185.724ms |    2.22e-15 |
| ASIC_680k    |   682,862 |   3,871,773 |  No   |          Auto |       3.458s |    7.33e-11 | **1m17.353s** |    8.09e-11 |
| tmt_unsym    |   917,825 |   4,584,801 |  No   |          Auto |       2.569s |     4.95e-7 |        3.061s |     1.43e-7 |
| Goodwin_127  |   178,437 |   5,778,545 |  No   |    MaxMinDiag |    372.312ms |    6.83e-11 |     846.998ms |    6.20e-10 |
| pre2         |   659,033 |   5,959,282 |  No   |          Auto |       3.428s |    2.15e-14 |        4.383s |    2.60e-14 |
| marine1      |   400,320 |   6,226,538 |  No   |          Auto |       4.032s |     1.70e-8 |        5.352s |    6.25e-14 |
| torso1       |   116,158 |   8,516,500 |  No   |  MaxDiagCount |       1.050s |     5.90e-7 |        1.847s | **3.75e-6** |
| atmosmodd    | 1,270,432 |   8,814,880 |  No   |          Auto |      18.668s |    1.44e-16 |       42.718s |    2.48e-16 |
| atmosmodl    | 1,489,752 |  10,319,760 |  No   |          Auto |      18.165s |    9.34e-18 |       38.733s |    7.60e-18 |
| memchip      | 2,707,524 |  14,810,202 |  No   |          Auto |       6.941s |    3.23e-15 |        7.546s |    3.40e-15 |
| Freescale1   | 3,428,755 |  18,920,347 |  No   |          Auto |       8.076s |    9.39e-10 |        9.580s |    7.05e-10 |
| rajat31      | 4,690,002 |  20,316,253 |  No   |          Auto |      17.285s |    3.26e-14 |       15.175s |    8.73e-15 |
| Transport    | 1,602,111 |  23,500,731 |  No   |          Auto |      21.417s |    7.21e-10 |       41.188s |    3.81e-10 |
| inline_1     |   503,712 |  36,816,342 | Yes*  |          None |       1.762s |    5.86e-15 |        3.261s |    3.38e-15 |
| PFlow_742    |   742,793 |  37,138,461 | Yes*  |          None |       9.792s |    1.77e-10 |       10.706s |    4.17e-11 |
| Emilia_923   |   923,136 |  41,005,206 | Yes*  |          None |      15.903s |    1.50e-22 |       37.580s |    5.27e-23 |
| dielFilterV2 | 1,157,456 |  48,538,952 |  Yes  |          None |       7.007s |    1.38e-11 |       12.388s |    1.49e-11 |
| Flan_1565    | 1,564,794 | 117,406,044 | Yes*  |          None |      10.212s |    2.24e-15 |       19.735s |    9.98e-16 |
| pres-cylin   | 1,711,464 | 133,562,188 | Yes*  |          None | **2m4.270s** |    8.89e-13 | **1m18.368s** |    5.09e-13 |



## Comparing KLU, UMFPACK and MUMPS

| Matrix      |      Nrow |        NNZ |  Sym  |      KLU Time |   KLU Error |  UMFPACK Time | UMFPACK Error |    MUMPS Time | MUMPS Error |
| ----------- | --------: | ---------: | :---: | ------------: | ----------: | ------------: | ------------: | ------------: | ----------: |
| bwm2000     |     2,000 |      7,996 |  No   |     263.664µs |    6.05e-16 |     713.627µs |      5.30e-16 |       6.260ms |    7.50e-16 |
| rdb5000     |     5,000 |     29,600 |  No   |       6.484ms |    1.68e-12 |      11.504ms |      1.93e-15 |       7.442ms |    5.37e-13 |
| Goodwin_040 |    17,922 |    561,677 |  No   |        4.679s |  **3.17e3** |     104.256ms |      6.62e-13 |     106.490ms |    1.65e-11 |
| fp          |     7,548 |    848,553 |  No   |     228.149ms |    7.32e-20 |     149.956ms |      2.57e-21 |     164.943ms |    1.16e-20 |
| xenon1      |    48,600 |  1,181,120 |  No   |        4.716s |    4.23e-39 |     582.982ms |      5.55e-40 |     308.937ms |    6.74e-40 |
| twotone     |   120,750 |  1,224,224 |  No   |       18.129s |     1.32e-6 |     334.710ms |      1.13e-14 |     879.251ms |    1.93e-13 |
| Raj1        |   263,743 |  1,302,464 |  No   |       21.521s | **1.64e-5** |           oom |           oom |     901.208ms |    3.63e-13 |
| boyd2       |   466,316 |  1,500,397 |  Yes  | **2m44.974s** |    1.26e-12 | **1m19.604s** |      2.86e-13 |     755.545ms |    8.98e-11 |
| Goodwin_071 |    56,021 |  1,797,934 |  No   |       36.348s |  **3.79e8** |     418.565ms |      2.77e-12 |     258.507ms |    8.38e-11 |
| darcy003    |   389,874 |  2,101,242 |  Yes  |        4.868s | **9.15e-4** |        1.072s |      6.95e-12 |       10.667s |    1.75e-10 |
| rma10       |    46,835 |  2,374,001 |  No   |     453.514ms |    3.13e-17 |     398.716ms |      1.57e-17 |     150.619ms |    1.35e-16 |
| helm2d03    |   392,257 |  2,741,935 |  Yes  |        6.913s |     2.82e-9 |        1.375s |      2.37e-13 |        1.440s |    3.82e-10 |
| stomach     |   213,360 |  3,021,648 |  No   |       20.335s |    5.04e-15 |        1.681s |      5.13e-16 |        1.579s |    1.49e-15 |
| oilpan      |    73,752 |  3,597,188 | Yes*  |        1.816s |    1.82e-14 |     481.117ms |      1.26e-15 |     185.724ms |    2.22e-15 |
| ASIC_680k   |   682,862 |  3,871,773 |  No   |     537.826ms |    1.99e-11 |        1.242s |      2.20e-11 | **1m17.353s** |    8.09e-11 |
| tmt_unsym   |   917,825 |  4,584,801 |  No   |       16.892s |     2.37e-6 |        3.655s |       5.96e-8 |        3.061s |     1.43e-7 |
| Goodwin_127 |   178,437 |  5,778,545 |  No   | **7m39.232s** | **3.06e13** |        1.380s |      6.03e-12 |     846.998ms |    6.20e-10 |
| marine1     |   400,320 |  6,226,538 |  No   | **8m54.468s** |    2.70e-13 |           oom |           oom |        5.352s |    6.25e-14 |
| torso1      |   116,158 |  8,516,500 |  No   |        4.876s | **6.01e-6** |        1.314s |       3.82e-8 |        1.847s | **3.75e-6** |
| memchip     | 2,707,524 | 14,810,202 |  No   | **1m39.551s** |    2.16e-14 |       28.947s |      2.04e-15 |        7.546s |    3.40e-15 |
| Freescale1  | 3,428,755 | 18,920,347 |  No   |        3.417s |    7.05e-10 |       28.693s |      7.05e-10 |        9.580s |    7.05e-10 |
| rajat31     | 4,690,002 | 20,316,253 |  No   | **2m16.058s** |    1.22e-14 |           oom |           oom |       15.175s |    8.73e-15 |



## System and libraries information

System information:

```
--- OS ---
NAME="Arch Linux"
KERNEL=7.0.9-arch2-1

--- GPU ---
GPU[0]: NVIDIA GeForce RTX 4090, 595.71.05, 24564 MiB

--- CPU ---
Architecture:        x86_64
CPU(s):              32
On-line CPU(s) list: 0-31
Model name:          13th Gen Intel(R) Core(TM) i9-13900KF
Thread(s) per core:  2
Core(s) per socket:  24
Socket(s):           1
CPU(s) scaling MHz:  32%
CPU max MHz:         5800.0000
CPU min MHz:         800.0000
BogoMIPS:            5990.40
L1d cache:           896 KiB (24 instances)
L1i cache:           1.3 MiB (24 instances)
L2 cache:            32 MiB (12 instances)
L3 cache:            36 MiB (1 instance)
NUMA node0 CPU(s):   0-31
Vulnerability L1tf:  Not affected

--- Memory ---
MemTotal:       32616244 kB
MemFree:         5895504 kB
MemAvailable:   23187532 kB
SwapTotal:      36806824 kB
```

Libraries information:

```
cuDSS: 0.8.0.10 (CUDA 13)
MUMPS: 5.9.0
SuiteSparse: latest (from GitHub)
```
