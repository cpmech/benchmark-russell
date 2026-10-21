use benchmark_russell::{AllResults, GENIES, MATRICES, MATRICES_COMPLEX, StrError};
use benchmark_russell::{matrix_table_md, performance_table_md};

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

    // ---- Native, UMFPACK and MUMPS ----
    let caption1 = format!(
        "Calculations on {}. Native, UMFPACK and MUMPS. Real-valued matrices.",
        OS
    );
    let caption2 = format!(
        "Calculations on {}. Native, UMFPACK and MUMPS. Complex-valued matrices.",
        OS
    );
    let native_umfpack_mumps_re = AllResults::new(GENIES, MATRICES, &results_dir)?;
    let native_umfpack_mumps_co = AllResults::new(GENIES, MATRICES_COMPLEX, &results_dir)?;
    native_umfpack_mumps_re.print_run_counts();
    native_umfpack_mumps_co.print_run_counts();
    print_table(&caption1, &native_umfpack_mumps_re)?;
    print_table(&caption2, &native_umfpack_mumps_co)?;

    // done
    Ok(())
}
