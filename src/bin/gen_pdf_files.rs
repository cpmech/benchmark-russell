use benchmark_russell::{AllResults, GENIES, MATRICES, MATRICES_COMPLEX, StrError};
use benchmark_russell::{call_latexmk, get_matrix_list_latex, performance_table};
use russell_sparse::{get_library_versions, get_system_info_linux};

const OS: &str = "Arch";
// const OS: &str = "Ubuntu";

fn main() -> Result<(), StrError> {
    // results dir
    let results_dir = format!("results/{}", OS.to_lowercase());

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

    // write information
    let mut text = "\n\
        \\noindent We test the linear system $\\bm{A}\\,\\bm{x}=\\bm{b}$ where $\\bm{A}$ is the coefficient matrix, $\\bm{x}$ is the solution vector, and $\\bm{b}$ is the right-hand side vector. The coefficient matrix is set with matrices from the \\href{https://sparse.tamu.edu}{SuiteSparse Matrix Collection}. The right-hand side vector is filled with ones, i.e., we study the solution of\n\
        \n\
        \\begin{equation}\n\
        \\bm{A}\\,\\bm{x} = \\bm{1}\n\
        \\label{eq:axb-linear-system}\n\
        \\end{equation}\n\
        \n\
        \\noindent The relative error is calculated as\n\
        \n\
        \\begin{equation}\n\
        \\texttt{RelativeError} = \\frac{\\max(|\\bm{A}\\,\\bm{x} - \\bm{1}|)}{\\max(|\\bm{A}| + 1)}\n\
        \\end{equation}\n".to_string();

    // generate list of matrices (real)
    text += "\n\\noindent The tested real-valued matrices are:\n\\begin{itemize}\n";
    text += &get_matrix_list_latex(MATRICES)?;
    text += "\\end{itemize}\n";

    // generate list of matrices (complex)
    text += "\n\\noindent The tested complex-valued matrices are:\n\\begin{itemize}\n";
    text += &get_matrix_list_latex(MATRICES_COMPLEX)?;
    text += "\\end{itemize}\n";

    // write notes
    text += "\n\\noindent Additional notes:\\begin{itemize}\n\
        \x20\x20 \\item Each problem is solved five times. The reported error is the maximum among runs. The reported computer time is the average without outliers.\n\
        \x20\x20 \\item Total time includes initialization (memory allocation + symbolic factorization), numeric factorization, and solve.\n\
        \x20\x20 \\item Column NNZ (number of non-zeros) corresponds to Pattern Entries reported by the \\href{https://sparse.tamu.edu}{SuiteSparse Matrix Collection}.\n\
        \x20\x20 \\item Column Sym shows if symmetry information was provided to the solver. An asterisk means positive definite information was also given.\n\
        \x20\x20 \\item oom (out-of-memory) indicates that the symbolic factorization was terminated due to insufficient memory.\n\
        \x20\x20 \\item n/a indicates that the run was not attempted, or that the Native solver exceeded its time or memory budget.\n\
        \x20\x20 \\item Bold values highlight significant results, such as large errors or high computation times.\n\
        \x20\x20 \\item The Native solver is pure Rust; it does not compute determinants, error estimates or condition numbers.\n\
    \\end{itemize}\n";

    // write information about the system
    text += "\n\\noindent Information about the system:\\begin{verbatim}\n";
    text += &get_system_info_linux();
    text += "\\end{verbatim}\n";

    // write information about the libraries
    text += "\n\\noindent Information about the libraries:\\begin{verbatim}\n";
    text += &get_library_versions();
    text += "\\end{verbatim}\n";

    // generate tables of results
    text += "\n\\noindent The results are given in Tables \\ref{tab:1} and \\ref{tab:2}.\n";
    let table1 = performance_table(
        &caption1,
        "tab:1",
        &native_umfpack_mumps_re,
        None,
        Some(4.0),
        false,
        false,
    )?;
    let table2 = performance_table(
        &caption2,
        "tab:2",
        &native_umfpack_mumps_co,
        None,
        Some(4.0),
        false,
        false,
    )?;
    text += table1.as_str();
    text += table2.as_str();

    // generate the PDF file
    let filename_stem = format!("{}-performance-tables", OS.to_lowercase());
    call_latexmk(&filename_stem, &text, true, true)?;
    println!("\n{} PDF file generated", filename_stem);

    // done
    Ok(())
}
