# Performance benchmarks using the Rust Scientific Library (Russell) <!-- omit from toc --> 

## Contents <!-- omit from toc --> 

- [Introduction](#introduction)
- [Solvers for large sparse linear systems](#solvers-for-large-sparse-linear-systems)
- [Results](#results)
  - [Information about the tested matrices](#information-about-the-tested-matrices)
  - [Calculations on Arch. cuDSS and MUMPS. Real-valued matrices.](#calculations-on-arch-cudss-and-mumps-real-valued-matrices)
  - [Calculations on Arch. cuDSS and MUMPS. Complex-valued matrices.](#calculations-on-arch-cudss-and-mumps-complex-valued-matrices)
  - [Calculations on Arch. UMFPACK and MUMPS. Real-valued matrices.](#calculations-on-arch-umfpack-and-mumps-real-valued-matrices)
  - [Calculations on Arch. UMFPACK and MUMPS. Complex-valued matrices.](#calculations-on-arch-umfpack-and-mumps-complex-valued-matrices)
- [System and libraries information](#system-and-libraries-information)



## Introduction

This project contains some performance benchmarks using the [Rust Scientific Library (Russell)](https://github.com/cpmech/russell).

Currently, we present some results involving the sparse linear solvers from [russell_sparse](https://github.com/cpmech/russell/tree/main/russell_sparse).

The computations presented here use all features (`intel_mkl`, `local_sparse`, and `cudss`). Thus, UMFPACK and MUMPS are compiled locally with Intel MKL. On the other hand, the Linux binary from [NVIDIA cuDSS](https://developer.nvidia.com/cudss) is employed for the calculations with cuDSS. See the [system and libraries information](#system-and-libraries-information) used in the benchmarks.



## Solvers for large sparse linear systems

Note that [NVIDIA cuDSS](https://developer.nvidia.com/cudss) is in **Preview Mode** 🚧.

The results are available in PDF format and also listed below.

* [Arch Linux. Results PDF file](results/arch-performance-tables.pdf)
* [Ubuntu Linux. Results PDF file](results/ubuntu-performance-tables.pdf)

We test the linear system **Ax = b** where **A** is the coefficient matrix, **x** is the solution vector, and **b** is the right-hand side vector. The coefficient matrix is set with matrices from the [SuiteSparse Matrix Collection](https://sparse.tamu.edu). The right-hand side vector is filled with ones, i.e., we study the solution of

**A x = 1**

The relative error is calculated as

```text
RelativeError = max(|A x - 1|) / max(|A| + 1)
```

The real-valued tested matrices are:

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

The complex-valued tested matrices are:

1. **mhd1280b** (Bai) -- Alfven spectra in magnetohydrodynamics (1994)
2. **mplate** (Cote) -- Vibro-acoustic problem (1997)
3. **RFdevice** (Rost) -- Semiconductor device simulation (2007)
4. **vfem** (CEMW) -- Electromagnetics, vector finite element (2008)
5. **fem_filter** (Lee) -- FEM band-pass microwave filter 500MHz (2008)
6. **Chevron4** (Chevron) -- Temporal freq domain seismic modeling (2012)
7. **mono_500Hz** (FreeFieldTechnologies) -- 3D vibro-acoustic problem, aircraft engine nacelle (2008)
8. **kim2** (Kim) -- 2D 676-by-676 complex mesh (2002)
9. **fem_hifreq_circuit** (Lee) -- FEM Maxwell equations for hi-freq circuit (2009)
10. **dielFilterV3clx** (Dziekonski) -- High-order vector finite element method in EM (2011)

**Additional notes:**

1. Each problem is solved ten times. The reported error is the maximum among runs. The reported computer time is the average without outliers.
2. Total time includes initialization (memory allocation + symbolic factorization), numeric factorization, and solve.
3. Column NNZ (number of non-zeros) corresponds to Pattern Entries reported by the [SuiteSparse Matrix Collection](https://sparse.tamu.edu).
4. Column Sym shows if symmetry information was provided to the solver. An asterisk means positive definite information was also given.
5. oom (out-of-memory) indicates that the symbolic factorization was terminated due to insufficient memory.
6. Bold values highlight significant results, such as large errors or high computation times.
7. The hybrid memory mode is enabled for the pres-cylin matrix and cuDSS.


## Results

### Information about the tested matrices

**Real-valued matrices:**
| Matrix       |      Nrow |         NNZ |  Sym  |
| ------------ | --------: | ----------: | :---: |
| bwm2000      |     2,000 |       7,996 |  No   |
| rdb5000      |     5,000 |      29,600 |  No   |
| Goodwin_040  |    17,922 |     561,677 |  No   |
| fp           |     7,548 |     848,553 |  No   |
| xenon1       |    48,600 |   1,181,120 |  No   |
| twotone      |   120,750 |   1,224,224 |  No   |
| Raj1         |   263,743 |   1,302,464 |  No   |
| boyd2        |   466,316 |   1,500,397 |  Yes  |
| Goodwin_071  |    56,021 |   1,797,934 |  No   |
| darcy003     |   389,874 |   2,101,242 |  Yes  |
| rma10        |    46,835 |   2,374,001 |  No   |
| helm2d03     |   392,257 |   2,741,935 |  Yes  |
| stomach      |   213,360 |   3,021,648 |  No   |
| oilpan       |    73,752 |   3,597,188 | Yes*  |
| ASIC_680k    |   682,862 |   3,871,773 |  No   |
| tmt_unsym    |   917,825 |   4,584,801 |  No   |
| Goodwin_127  |   178,437 |   5,778,545 |  No   |
| pre2         |   659,033 |   5,959,282 |  No   |
| marine1      |   400,320 |   6,226,538 |  No   |
| torso1       |   116,158 |   8,516,500 |  No   |
| atmosmodd    | 1,270,432 |   8,814,880 |  No   |
| atmosmodl    | 1,489,752 |  10,319,760 |  No   |
| memchip      | 2,707,524 |  14,810,202 |  No   |
| Freescale1   | 3,428,755 |  18,920,347 |  No   |
| rajat31      | 4,690,002 |  20,316,253 |  No   |
| Transport    | 1,602,111 |  23,500,731 |  No   |
| inline_1     |   503,712 |  36,816,342 | Yes*  |
| PFlow_742    |   742,793 |  37,138,461 | Yes*  |
| Emilia_923   |   923,136 |  41,005,206 | Yes*  |
| dielFilterV2 | 1,157,456 |  48,538,952 |  Yes  |
| Flan_1565    | 1,564,794 | 117,406,044 | Yes*  |
| pres-cylin   | 1,711,464 | 133,562,188 | Yes*  |

**Complex-valued matrices:**
| Matrix             |    Nrow |        NNZ |  Sym  |
| ------------------ | ------: | ---------: | :---: |
| mhd1280b           |   1,280 |     22,778 |  No   |
| mplate             |   5,962 |    142,190 |  Yes  |
| RFdevice           |  74,104 |    365,580 |  No   |
| vfem               |  93,476 |  1,434,636 |  No   |
| fem_filter         |  74,062 |  1,731,206 |  No   |
| Chevron4           | 711,450 |  6,376,412 |  No   |
| mono_500Hz         | 169,410 |  5,036,288 |  No   |
| kim2               | 456,976 | 11,330,020 |  No   |
| fem_hifreq_circuit | 491,100 | 20,239,237 |  No   |
| dielFilterV3clx    | 420,408 | 32,886,208 |  Yes  |



### Calculations on Arch. cuDSS and MUMPS. Real-valued matrices.

| Matrix       |      cuDSS MA |   cuDSS Time | cuDSS Error |    MUMPS Time | MUMPS Error |
| ------------ | ------------: | -----------: | ----------: | ------------: | ----------: |
| bwm2000      |          Auto |      8.633ms |    8.21e-16 |       6.260ms |    7.50e-16 |
| rdb5000      |          Auto |     30.087ms |    4.06e-13 |       7.442ms |    5.37e-13 |
| Goodwin_040  |    MaxMinDiag |     71.095ms |    2.82e-12 |     106.490ms |    1.65e-11 |
| fp           |          Auto |    155.376ms |     4.98e-9 |     164.943ms |    1.16e-20 |
| xenon1       |          Auto |    194.980ms |    1.32e-39 |     308.937ms |    6.74e-40 |
| twotone      |          Auto |    772.871ms |    4.73e-13 |     879.251ms |    1.93e-13 |
| Raj1         |          Auto |    883.834ms |    1.21e-11 |     901.208ms |    3.63e-13 |
| boyd2        |          None |    996.540ms |    3.24e-11 |     755.545ms |    8.98e-11 |
| Goodwin_071  |    MaxMinDiag |    174.813ms |    8.65e-12 |     258.507ms |    8.38e-11 |
| darcy003     | MaxMinDiagAlt |       2.438s |     5.64e-7 |       10.667s |    1.75e-10 |
| rma10        |          Auto |    241.711ms |    9.04e-16 |     150.619ms |    1.35e-16 |
| helm2d03     |          None |       1.239s |    1.68e-10 |        1.440s |    3.82e-10 |
| stomach      |          Auto |       1.280s |    2.71e-15 |        1.579s |    1.49e-15 |
| oilpan       |          None |    102.687ms |    4.65e-15 |     185.724ms |    2.22e-15 |
| ASIC_680k    |          Auto |       3.458s |    7.33e-11 | **1m17.353s** |    8.09e-11 |
| tmt_unsym    |          Auto |       2.569s |     4.95e-7 |        3.061s |     1.43e-7 |
| Goodwin_127  |    MaxMinDiag |    372.312ms |    6.83e-11 |     846.998ms |    6.20e-10 |
| pre2         |          Auto |       3.428s |    2.15e-14 |        4.383s |    2.60e-14 |
| marine1      |          Auto |       4.032s |     1.70e-8 |        5.352s |    6.25e-14 |
| torso1       |  MaxDiagCount |       1.050s |     5.90e-7 |        1.847s | **3.75e-6** |
| atmosmodd    |          Auto |      18.668s |    1.44e-16 |       42.718s |    2.48e-16 |
| atmosmodl    |          Auto |      18.165s |    9.34e-18 |       38.733s |    7.60e-18 |
| memchip      |          Auto |       6.941s |    3.23e-15 |        7.546s |    3.40e-15 |
| Freescale1   |          Auto |       8.076s |    9.39e-10 |        9.580s |    7.05e-10 |
| rajat31      |          Auto |      17.285s |    3.26e-14 |       15.175s |    8.73e-15 |
| Transport    |          Auto |      21.417s |    7.21e-10 |       41.188s |    3.81e-10 |
| inline_1     |          None |       1.762s |    5.86e-15 |        3.261s |    3.38e-15 |
| PFlow_742    |          None |       9.792s |    1.77e-10 |       10.706s |    4.17e-11 |
| Emilia_923   |          None |      15.903s |    1.50e-22 |       37.580s |    5.27e-23 |
| dielFilterV2 |          None |       7.007s |    1.38e-11 |       12.388s |    1.49e-11 |
| Flan_1565    |          None |      10.212s |    2.24e-15 |       19.735s |    9.98e-16 |
| pres-cylin   |          None | **2m4.270s** |    8.89e-13 | **1m18.368s** |    5.09e-13 |



### Calculations on Arch. cuDSS and MUMPS. Complex-valued matrices.

| Matrix             |   cuDSS MA | cuDSS Time | cuDSS Error | MUMPS Time | MUMPS Error |
| ------------------ | ---------: | ---------: | ----------: | ---------: | ----------: |
| mhd1280b           |       Auto |    7.281ms |    1.57e-15 |    4.551ms |    1.17e-15 |
| mplate             |       None |   78.587ms |    1.13e-10 |  167.040ms |    6.28e-11 |
| RFdevice           | MaxDiagSum |  570.478ms | **3.21e-2** |     1.548s |     1.56e-7 |
| vfem               |       Auto |  865.606ms |     5.29e-8 |     1.403s |     8.74e-8 |
| fem_filter         |       Auto |  713.189ms |    1.81e-10 |     1.039s |    1.44e-10 |
| Chevron4           |       Auto |     2.480s |    8.51e-11 |     3.139s |    4.16e-11 |
| mono_500Hz         |       Auto |     2.735s |    2.23e-10 |     4.830s |     5.99e-9 |
| kim2               |       Auto |    13.916s |      0.00e0 |     4.346s |    2.73e-18 |
| fem_hifreq_circuit |       Auto |     4.871s |    1.60e-10 |     9.018s |    1.28e-10 |
| dielFilterV3clx    |       None |     2.184s |    4.31e-10 |     3.969s |    1.73e-11 |



### Calculations on Arch. UMFPACK and MUMPS. Real-valued matrices.

| Matrix       |  UMFPACK Time | UMFPACK Error |    MUMPS Time | MUMPS Error |
| ------------ | ------------: | ------------: | ------------: | ----------: |
| bwm2000      |     713.627µs |      5.30e-16 |       6.260ms |    7.50e-16 |
| rdb5000      |      11.504ms |      1.93e-15 |       7.442ms |    5.37e-13 |
| Goodwin_040  |     104.256ms |      6.62e-13 |     106.490ms |    1.65e-11 |
| fp           |     149.956ms |      2.57e-21 |     164.943ms |    1.16e-20 |
| xenon1       |     582.982ms |      5.55e-40 |     308.937ms |    6.74e-40 |
| twotone      |     334.710ms |      1.13e-14 |     879.251ms |    1.93e-13 |
| Raj1         |           oom |           oom |     901.208ms |    3.63e-13 |
| boyd2        | **1m19.604s** |      2.86e-13 |     755.545ms |    8.98e-11 |
| Goodwin_071  |     418.565ms |      2.77e-12 |     258.507ms |    8.38e-11 |
| darcy003     |        1.072s |      6.95e-12 |       10.667s |    1.75e-10 |
| rma10        |     398.716ms |      1.57e-17 |     150.619ms |    1.35e-16 |
| helm2d03     |        1.375s |      2.37e-13 |        1.440s |    3.82e-10 |
| stomach      |        1.681s |      5.13e-16 |        1.579s |    1.49e-15 |
| oilpan       |     481.117ms |      1.26e-15 |     185.724ms |    2.22e-15 |
| ASIC_680k    |        1.242s |      2.20e-11 | **1m17.353s** |    8.09e-11 |
| tmt_unsym    |        3.655s |       5.96e-8 |        3.061s |     1.43e-7 |
| Goodwin_127  |        1.380s |      6.03e-12 |     846.998ms |    6.20e-10 |
| pre2         |           oom |           oom |        4.383s |    2.60e-14 |
| marine1      |           oom |           oom |        5.352s |    6.25e-14 |
| torso1       |        1.314s |       3.82e-8 |        1.847s | **3.75e-6** |
| atmosmodd    |           oom |           oom |       42.718s |    2.48e-16 |
| atmosmodl    |           oom |           oom |       38.733s |    7.60e-18 |
| memchip      |       28.947s |      2.04e-15 |        7.546s |    3.40e-15 |
| Freescale1   |       28.693s |      7.05e-10 |        9.580s |    7.05e-10 |
| rajat31      |           oom |           oom |       15.175s |    8.73e-15 |
| Transport    |           oom |           oom |       41.188s |    3.81e-10 |
| inline_1     |           oom |           oom |        3.261s |    3.38e-15 |
| PFlow_742    |           oom |           oom |       10.706s |    4.17e-11 |
| Emilia_923   |           oom |           oom |       37.580s |    5.27e-23 |
| dielFilterV2 |           oom |           oom |       12.388s |    1.49e-11 |
| Flan_1565    |           oom |           oom |       19.735s |    9.98e-16 |
| pres-cylin   |           oom |           oom | **1m18.368s** |    5.09e-13 |



### Calculations on Arch. UMFPACK and MUMPS. Complex-valued matrices.

| Matrix             | UMFPACK Time | UMFPACK Error | MUMPS Time | MUMPS Error |
| ------------------ | -----------: | ------------: | ---------: | ----------: |
| mhd1280b           |    345.764µs |      7.41e-16 |    4.551ms |    1.17e-15 |
| mplate             |    412.395ms |      2.32e-11 |  167.040ms |    6.28e-11 |
| RFdevice           |       3.901s |      9.16e-14 |     1.548s |     1.56e-7 |
| vfem               |       5.132s |       2.07e-8 |     1.403s |     8.74e-8 |
| fem_filter         |       2.366s |      6.32e-11 |     1.039s |    1.44e-10 |
| Chevron4           |       4.528s |      1.45e-11 |     3.139s |    4.16e-11 |
| mono_500Hz         |          oom |           oom |     4.830s |     5.99e-9 |
| kim2               |          oom |           oom |     4.346s |    2.73e-18 |
| fem_hifreq_circuit |          oom |           oom |     9.018s |    1.28e-10 |
| dielFilterV3clx    |          oom |           oom |     3.969s |    1.73e-11 |




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
