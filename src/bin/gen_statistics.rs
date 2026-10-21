use benchmark_russell::{AllResults, GENIES, MATRICES, StrError, genie_display_name};

const RESULTS_DIR: &str = "results/arch";

fn main() -> Result<(), StrError> {
    // read the results
    let data = AllResults::new(GENIES, MATRICES, RESULTS_DIR)?;

    // print statistics per solver
    for genie in GENIES {
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
