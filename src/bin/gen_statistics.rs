use benchmark_russell::{AllResults, GENIES, GENIES_ITERATIVE, MATRICES, StrError, genie_display_name};

const RESULTS_DIR: &str = "results/arch";

fn main() -> Result<(), StrError> {
    // read the results
    let data = AllResults::new(GENIES, MATRICES, RESULTS_DIR)?;
    let data_it = AllResults::new(GENIES_ITERATIVE, MATRICES, RESULTS_DIR)?;

    // print statistics per solver
    for genie in GENIES.iter().chain(GENIES_ITERATIVE.iter()) {
        let data = if GENIES_ITERATIVE.contains(genie) { &data_it } else { &data };
        println!("\n### {}\n", genie_display_name(*genie));
        println!("| Matrix | Min Time | Median Time | Max Time | Outlier |");
        println!("| --- | ---: | ---: | ---: | ---: |");
        for matrix in MATRICES {
            let row = data.get_statistics(*genie, *matrix)?;
            println!("{row}");
        }
    }

    Ok(())
}
