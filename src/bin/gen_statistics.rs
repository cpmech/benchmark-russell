use benchmark_russell::{AllResults, MATRICES, StrError};
use russell_sparse::Genie;

const RESULTS_DIR: &str = "results/arch";

fn main() -> Result<(), StrError> {
    // read the results
    let genies = &[Genie::Cudss, Genie::Mumps, Genie::Umfpack];
    let data = AllResults::new(genies, MATRICES, RESULTS_DIR)?;

    // print statistics: cuDSS
    println!("### cuDSS\n");
    println!("| Matrix | Min Time | Median Time | Max Time | Outlier |");
    println!("| --- | ---: | ---: | ---: | ---: |");
    for matrix in MATRICES {
        let row = data.get_statistics(Genie::Cudss, matrix)?;
        println!("{row}");
    }

    // print statistics: MUMPS
    println!("\n### MUMPS\n");
    println!("| Matrix | Min Time | Median Time | Max Time | Outlier |");
    println!("| --- | ---: | ---: | ---: | ---: |");
    for matrix in MATRICES {
        let row = data.get_statistics(Genie::Mumps, matrix)?;
        println!("{row}");
    }

    // print statistics: UMFPACK
    println!("\n### UMFPACK\n");
    println!("| Matrix | Min Time | Median Time | Max Time | Outlier |");
    println!("| --- | ---: | ---: | ---: | ---: |");
    for matrix in MATRICES {
        let row = data.get_statistics(Genie::Umfpack, matrix)?;
        println!("{row}");
    }

    Ok(())
}
