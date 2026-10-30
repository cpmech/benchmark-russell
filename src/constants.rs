use russell_sparse::{Genie, IterativeMethod};

/// Big relative error to be highlighted
pub const BIG_REL_ERROR: f64 = 2.5e-6;

/// Big total time value to be highlighted
pub const BIG_TIME_MIN: f64 = 1.0;

/// Direct solvers compared across the benchmark
pub const GENIES: &[Genie] = &[Genie::Direct, Genie::Umfpack, Genie::Mumps];

/// Iterative solvers compared across the benchmark
///
/// All three run on the AMG preconditioner with the default parameters of
/// `russell_sparse`, so the comparison is between the Krylov methods and not between settings.
pub const GENIES_ITERATIVE: &[Genie] = &[
    Genie::Iterative(IterativeMethod::Cg),
    Genie::Iterative(IterativeMethod::BiCgStab),
    Genie::Iterative(IterativeMethod::Gmres),
];

/// Matrix names used across benchmarks
pub const MATRICES: &[&str] = &[
    "bwm2000",
    "rdb5000",
    "Goodwin_040",
    "fp",
    "xenon1",
    "twotone",
    "Raj1",
    "boyd2",
    "Goodwin_071",
    "darcy003",
    "rma10",
    "helm2d03",
    "stomach",
    "oilpan",
    "ASIC_680k",
    "tmt_unsym",
    "Goodwin_127",
    "pre2",
    "marine1",
    "torso1",
    "atmosmodd",
    "atmosmodl",
    "memchip",
    "Freescale1",
    "rajat31",
    "Transport",
    "inline_1",
    "PFlow_742",
    "Emilia_923",
    "dielFilterV2real",
    "Flan_1565",
    "pres-cylin-3d-tet10-fine",
    "pmsim-hex8-27",
];

/// Complex matrices
pub const MATRICES_COMPLEX: &[&str] = &[
    "mhd1280b",
    "mplate",
    "RFdevice",
    "vfem",
    "fem_filter",
    "Chevron4",
    "mono_500Hz",
    "kim2",
    "fem_hifreq_circuit",
    "dielFilterV3clx",
];
