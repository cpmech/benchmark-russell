use benchmark_russell::{AllResults, MATRICES, MATRICES_COMPLEX, StrError};
use benchmark_russell::{matrix_table_md, performance_table_md};
use russell_sparse::Genie;

const OS: &str = "Arch";
// const OS: &str = "Ubuntu";

fn print_table<'a>(caption: &str, data: &AllResults<'a>) -> Result<(), StrError> {
    let table = performance_table_md(&data)?;
    println!("\n\n### {}\n\n{}", caption, table);
    Ok(())
}

fn main() -> Result<(), StrError> {
    // results dir
    let results_dir = format!("results/{}", OS.to_lowercase());

    // ---- matrix table ----
    let txt_re = matrix_table_md(MATRICES)?;
    let txt_co = matrix_table_md(MATRICES_COMPLEX)?;
    println!("### Information about the tested matrices\n");
    println!("**Real-valued matrices:**\n{}", txt_re);
    println!("**Complex-valued matrices:**\n{}", txt_co);

    // ---- cuDSS and MUMPS ----
    let caption1 = format!("Calculations on {}. cuDSS and MUMPS. Real-valued matrices.", OS);
    let caption2 = format!("Calculations on {}. cuDSS and MUMPS. Complex-valued matrices.", OS);
    let cudss_mumps_re = AllResults::new(&[Genie::Cudss, Genie::Mumps], MATRICES, &results_dir)?;
    let cudss_mumps_co = AllResults::new(&[Genie::Cudss, Genie::Mumps], MATRICES_COMPLEX, &results_dir)?;
    cudss_mumps_re.check_number_of_runs()?;
    cudss_mumps_co.check_number_of_runs()?;
    print_table(&caption1, &cudss_mumps_re)?;
    print_table(&caption2, &cudss_mumps_co)?;

    // ---- UMFPACK and MUMPS ----
    let caption3 = format!("Calculations on {}. UMFPACK and MUMPS. Real-valued matrices.", OS);
    let caption4 = format!("Calculations on {}. UMFPACK and MUMPS. Complex-valued matrices.", OS);
    let umfpack_mumps_re = AllResults::new(&[Genie::Umfpack, Genie::Mumps], MATRICES, &results_dir)?;
    let umfpack_mumps_co = AllResults::new(&[Genie::Umfpack, Genie::Mumps], MATRICES_COMPLEX, &results_dir)?;
    umfpack_mumps_re.check_number_of_runs()?;
    umfpack_mumps_co.check_number_of_runs()?;
    print_table(&caption3, &umfpack_mumps_re)?;
    print_table(&caption4, &umfpack_mumps_co)?;

    // done
    Ok(())
}
