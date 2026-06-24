use crate::constants::{BIG_REL_ERROR, BIG_TIME_MIN};
use crate::{AllResults, StrError};
use fancy_regex::Regex;
use russell_lab::format_nanoseconds_with_digits as fmt_nano;
use russell_sparse::Genie;
use std::fmt::Write;

/// Column label: two-line stacked "Matching / Algorithm"
const STR_M_ALGO: &str = "{\\shortstack[l]{Matching\\\\Algorithm}}";

/// Column label: two-line stacked "Total / Time"
const STR_TOT_TIME: &str = "{\\shortstack[l]{Total\\\\Time}}";

/// Column label: two-line stacked "Relative / Error"
const STR_REL_ERROR: &str = "{\\shortstack[l]{Relative\\\\Error}}";

/// Generates a LaTeX comparison table for the linear solvers
///
/// 1. Reads StatsLinSol JSON result files from `data/results/{genie}-{matrix}.json`.
/// 2. Builds a NiceTabular with alternating row colors and grouped multi-column headers (Total Time / Relative Error per solver).
/// 3. Times and errors exceeding the configured thresholds are rendered bold.
/// 4. Handles out-of-memory (oom) and not-available (n/a).
///
/// Returns the LaTeX code as a String
pub fn performance_table<'a>(
    caption: &str,
    label: &str,
    data: &AllResults<'a>,
    row_sep_pt: Option<f64>,
    col_sep_pt: Option<f64>,
    with_horiz_lines: bool,
    tight_times_op: bool,
) -> Result<String, StrError> {
    // ----- validate input -----
    if data.map.is_empty() {
        return Err("results map must not be empty");
    }

    // ----- setup: regex for thousand-separator insertion -----
    let re = Regex::new(r"\B(?=(\d{3})+(?!\d))").unwrap();

    // ----- begin building the LaTeX table header -----
    let mut buf = String::new();

    // optional column separation
    let str_col_sep = match col_sep_pt {
        Some(val) => &format!("\\setlength{{\\tabcolsep}}{{{}pt}}\n", val),
        None => "",
    };

    // spacing around \times in siunitx numbers
    let str_times_op = if tight_times_op {
        ", exponent-product={{\\!{{\\times}}\\!}}"
    } else {
        ", exponent-product={{\\mkern1mu{{\\times}}\\mkern1mu}}"
    };

    // open table environment with sisetup (thousand separator, exponent spacing)
    writeln!(
        &mut buf,
        "\\begin{{table}}[!h]\n\
         \\centering\n\
         \\caption{{{}}}\\label{{{}}}\n\
         \\small\n\
         {}\\sisetup{{group-separator={{,}}, detect-weight=true{}}}",
        caption, label, str_col_sep, str_times_op
    )
    .unwrap();

    // vertical cell padding (default 3 pt)
    let v_gap = match row_sep_pt {
        Some(x) => x,
        None => 3.0,
    };

    // optional horizontal rules (top/mid/bottom)
    let str_top_rule = if with_horiz_lines { "\n\\toprule" } else { "" };
    let str_mid_rule = if with_horiz_lines { "\\midrule" } else { "" };
    let str_bot_rule = if with_horiz_lines { "\\bottomrule\n" } else { "" };

    // build dynamic column specs: 4 fixed cols (l, r, r, c) + 2 per genie (l + S[1.2e-2])
    write!(
        &mut buf,
        "\\begin{{NiceTabular}}{{\n\
         \x20\x20 l\n\
         \x20\x20 r % ndim\n\
         \x20\x20 r % nnz\n\
         \x20\x20 c % sym\n"
    )
    .unwrap();
    for genie in data.genies {
        let name = genie.to_string();
        if *genie == Genie::Cudss {
            writeln!(
                &mut buf,
                "\x20\x20 l                                                                          % {name} extra\n\
                 \x20\x20 l                                                                          % {name} time\n\
                 \x20\x20 S[table-format=1.2e-2, exponent-mode=scientific, print-zero-exponent=true] % {name} error"
            )
            .unwrap();
        } else {
            writeln!(
                &mut buf,
                "\x20\x20 l                                                                          % {name} time\n\
                 \x20\x20 S[table-format=1.2e-2, exponent-mode=scientific, print-zero-exponent=true] % {name} error"
            )
            .unwrap();
        }
    }
    writeln!(
        &mut buf,
        " }}[\n\
        cell-space-limits={}pt,\n\
        code-before={{\n\
         \x20\x20 \\rowcolor{{gray!35}}{{1,2}}\n\
         \x20\x20 \\rowcolors{{3}}{{white}}{{gray!15}}\n\
        }}]{}",
        v_gap, str_top_rule
    )
    .unwrap();

    // ----- sub-header: grouped multi-column spans for each solver -----
    write!(&mut buf, "& & & & ").unwrap();
    for (k, genie) in data.genies.iter().enumerate() {
        let name = genie.to_string();
        let display: String = if name == "cudss" {
            "cuDSS".to_string()
        } else if name == "klu" {
            "KLU".to_string()
        } else {
            name.to_uppercase()
        };
        let ncol = if *genie == Genie::Cudss { 3 } else { 2 };
        write!(&mut buf, "\\multicolumn{{{ncol}}}{{c}}{{\\bfseries {display}}}").unwrap();
        if k < data.genies.len() - 1 {
            write!(&mut buf, " & ").unwrap();
        }
    }
    writeln!(&mut buf, " \\\\").unwrap();

    // cmidrules: span the time-error columns within each solver block
    let mut col = 5; // first solver column (1-indexed, after Matrix/Nrow/NNZ/Sym)
    for (k, genie) in data.genies.iter().enumerate() {
        let has_extra = *genie == Genie::Cudss;
        let start = col;
        let end = col + if has_extra { 2 } else { 1 };
        write!(&mut buf, "\\cmidrule(lr){{{start}-{end}}}").unwrap();
        col += if has_extra { 3 } else { 2 };
        if k < data.genies.len() - 1 {
            write!(&mut buf, " ").unwrap();
        }
    }
    writeln!(&mut buf).unwrap();

    // column labels: Matrix | Nrow | NNZ | Sym | {MA}|Time|Error per solver
    write!(&mut buf, "\\RowStyle{{\\bfseries}} Matrix & Nrow & NNZ & Sym").unwrap();
    for k in 0..data.genies.len() {
        if data.genies[k] == Genie::Cudss {
            write!(&mut buf, " & {} & {} & {}", STR_M_ALGO, STR_TOT_TIME, STR_REL_ERROR).unwrap();
        } else {
            write!(&mut buf, " & {} & {}", STR_TOT_TIME, STR_REL_ERROR).unwrap();
        }
        if k == data.genies.len() - 1 {
            write!(&mut buf, "\\\\{}", str_mid_rule).unwrap();
        }
    }
    writeln!(&mut buf).unwrap();

    // ----- process each matrix -----
    let i_last = data.matrices.len() - 1;

    for (i, matrix) in data.matrices.iter().enumerate() {
        // ----- fill solver columns -----
        for (j, genie) in data.genies.iter().enumerate() {
            // Get the specific results set
            let dat = data.map.get(&(*genie, matrix)).unwrap();

            // Write "Matrix & Nrow & NNZ & Sym"
            if j == 0 {
                // write the matrix name, with long-name abbreviations
                if *matrix == "pres-cylin-3d-tet10-fine-nu499" {
                    write!(&mut buf, "pres-cylinNI").unwrap();
                } else if *matrix == "pres-cylin-3d-tet10-fine" {
                    write!(&mut buf, "pres-cylin").unwrap();
                } else if *matrix == "dielFilterV2real" {
                    write!(&mut buf, "dielFilterV2").unwrap();
                } else {
                    write!(&mut buf, "{}", matrix.replace("_", "\\_")).unwrap();
                }

                let tmp_ndim = format!("{}", dat.matrix.nrow);
                let tmp_nnz = format!("{}", dat.matrix.nnz_actual);
                let str_ndim = re.replace_all(&tmp_ndim, ",");
                let str_nnz = re.replace_all(&tmp_nnz, ",");
                write!(&mut buf, "& {} & {}", str_ndim, str_nnz).unwrap();
                if dat.matrix.symmetric == "No" {
                    write!(&mut buf, " & No").unwrap();
                } else if dat.requests.positive_definite {
                    write!(&mut buf, " & Yes*").unwrap();
                } else {
                    write!(&mut buf, " & Yes").unwrap();
                }
            }

            // Write extra column if cuDSS
            if *genie == Genie::Cudss {
                write!(
                    &mut buf,
                    " & {}",
                    // get_key_for_matching_algorithm(&dat.requests.matching)
                    dat.requests.matching
                )
                .unwrap();
            }

            // Write " & Total Time & Relative Error"
            if dat.main.out_of_memory {
                // Handle out-of-memory (oom) case
                write!(&mut buf, "& oom & oom").unwrap();
            } else if dat.time_nanoseconds.total_ifs == 0 {
                // Handle not-available (n/a) case
                write!(&mut buf, "& n/a & n/a").unwrap();
            } else {
                // Set time string
                let time = data.recalculate_total_time_without_outliers(*genie, matrix).unwrap();
                let time_str = if (time as f64) / 6e+10 > BIG_TIME_MIN {
                    // bold if total time exceeds threshold
                    format!("\\bfseries {}", fmt_nano(time, 3))
                } else {
                    // normal font weight
                    fmt_nano(time, 3)
                };
                // Set error string
                let str_err = if dat.verify.relative_error > BIG_REL_ERROR {
                    // bold if error exceeds threshold
                    format!("\\bfseries {:.2e}", dat.verify.relative_error)
                } else {
                    // normal font weight
                    format!("{:.2e}", dat.verify.relative_error)
                };
                // Write Time and Error
                write!(&mut buf, " & {} & {}", time_str, str_err).unwrap();
            }
        }

        // end of row: last matrix gets newline so that the LaTeX code is pretty
        if i == i_last {
            writeln!(&mut buf, "\n\\\\").unwrap();
        } else {
            writeln!(&mut buf, "\\\\").unwrap();
        }
    }

    // ----- close table and compile to PDF -----
    writeln!(
        &mut buf,
        "{}\\end{{NiceTabular}}\n\
         \\end{{table}}",
        str_bot_rule,
    )
    .unwrap();

    Ok(buf)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::performance_table;
    use crate::{AllResults, call_latexmk};
    use russell_sparse::Genie;
    use std::fs;
    use std::path::Path;

    fn has_latexmk() -> bool {
        std::process::Command::new("latexmk")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    // Test: generates a performance table with 6 sample matrices and verifies
    // the PDF is produced by latexmk.
    #[test]
    fn performance_table_works() {
        if !has_latexmk() {
            return;
        }

        let stem = "performance-table";
        let tex_path = format!("/tmp/benchmark-russell/latex/{}.tex", stem);
        let pdf_path = format!("/tmp/benchmark-russell/latex/{}.pdf", stem);

        // clean up any leftover files from a previous run
        let _ = fs::remove_file(&tex_path);
        let _ = fs::remove_file(&pdf_path);

        // load results
        let data = AllResults::new(
            &[Genie::Umfpack, Genie::Mumps],
            &[
                "ASIC_680k",
                "bwm2000",
                "rdb5000",
                "Goodwin_040",
                "fp",
                "helm2d03",
                "pre2",
                "darcy003",
                "torso1",
            ],
            "results/arch",
        )
        .unwrap();

        // generate the table
        let buf = performance_table("Performance results.", "tab:1", &data, None, None, false, false).unwrap();

        // compile LaTeX via latexmk; output goes to /tmp/benchmark-russell/latex/
        call_latexmk("performance-table", &buf, true, true).unwrap();

        // verify PDF was produced
        assert!(Path::new(&pdf_path).exists(), "PDF was not generated");
    }
}
