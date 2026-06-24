use benchmark_russell::{AllResults, MATRICES, MATRICES_UMFPACK, StrError, performance_table_md};
use russell_sparse::Genie;

const RESULTS_DIR: &str = "results/arch";

fn print_table<'a>(data: &AllResults<'a>) -> Result<(), StrError> {
    // get the number of runs from the first matrix (non-oom)
    let nrun_expected = data
        .matrices
        .iter()
        .find_map(|&m| {
            data.map
                .get(&(data.genies[0], m))
                .filter(|s| !s.main.out_of_memory)
                .map(|s| s.time_nanoseconds.total_ifs_array.len())
        })
        .unwrap_or(0);

    // check if the number of runs is consistent
    for (_, stats) in &data.map {
        if !stats.main.out_of_memory {
            if stats.time_nanoseconds.total_ifs_array.len() != nrun_expected {
                println!(
                    "({}, {}): number of runs = {}",
                    stats.main.solver,
                    stats.matrix.name,
                    stats.time_nanoseconds.total_ifs_array.len()
                );
                return Err("number of runs is not consistent");
            }
        }
    }

    // generate the table
    let table = performance_table_md(&data)?;
    println!("{}", table);
    Ok(())
}

fn main() -> Result<(), StrError> {
    // cuDSS and MUMPS
    let genies = &[Genie::Cudss, Genie::Mumps];
    let data = AllResults::new(genies, MATRICES, RESULTS_DIR)?;
    print_table(&data)?;

    // UMFPACK and MUMPS
    let genies = &[Genie::Umfpack, Genie::Mumps];
    let data = AllResults::new(genies, MATRICES_UMFPACK, RESULTS_DIR)?;
    println!("\n\n");
    print_table(&data)?;

    // done
    Ok(())
}
