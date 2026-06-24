use crate::{AllResults, StrError};
use crate::constants::{BIG_REL_ERROR, BIG_TIME_MIN};
use fancy_regex::Regex;
use russell_lab::format_nanoseconds_with_digits as fmt_nano;
use russell_sparse::Genie;
use std::fmt::Write;

/// Generates a Markdown comparison table for the linear solvers
///
/// Times and errors exceeding the configured thresholds are rendered **bold**.
/// Handles out-of-memory (oom) and not-available (n/a).
pub fn performance_table_md<'a>(data: &AllResults<'a>) -> Result<String, StrError> {
    // ----- validate input -----
    if data.map.is_empty() {
        return Err("results map must not be empty");
    }

    // ----- setup: regex for thousand-separator insertion -----
    let re = Regex::new(r"\B(?=(\d{3})+(?!\d))").unwrap();

    // ----- build header row: solver names spanning columns -----
    let mut buf = String::new();
    write!(&mut buf, "| Matrix | Nrow | NNZ | Sym |").unwrap();
    for genie in data.genies {
        let name = genie.to_string();
        let display: String = if name == "cudss" {
            "cuDSS".to_string()
        } else if name == "klu" {
            "KLU".to_string()
        } else {
            name.to_uppercase()
        };
        if *genie == Genie::Cudss {
            write!(&mut buf, " {display} MA | {display} Time | {display} Error |").unwrap();
        } else {
            write!(&mut buf, " {display} Time | {display} Error |").unwrap();
        }
    }
    writeln!(&mut buf).unwrap();

    // ----- separator row with alignment -----
    write!(&mut buf, "| --- | ---: | ---: | :-: |").unwrap();
    for genie in data.genies {
        if *genie == Genie::Cudss {
            write!(&mut buf, " ---: | ---: | ---: |").unwrap();
        } else {
            write!(&mut buf, " ---: | ---: |").unwrap();
        }
    }
    writeln!(&mut buf).unwrap();

    // ----- process each matrix -----
    for (_i, matrix) in data.matrices.iter().enumerate() {
        // matrix name (with abbreviations for long names)
        let mname = if *matrix == "pres-cylin-3d-tet10-fine-nu499" {
            "pres-cylinNI"
        } else if *matrix == "pres-cylin-3d-tet10-fine" {
            "pres-cylin"
        } else if *matrix == "dielFilterV2real" {
            "dielFilterV2"
        } else {
            matrix
        };

        // ndim, nnz, sym from first genie
        let d0 = data.map.get(&(data.genies[0], matrix)).unwrap();
        let tmp_ndim = format!("{}", d0.matrix.nrow);
        let tmp_nnz = format!("{}", d0.matrix.nnz_actual);
        let str_ndim = re.replace_all(&tmp_ndim, ",");
        let str_nnz = re.replace_all(&tmp_nnz, ",");
        let str_sym = if d0.matrix.symmetric == "No" {
            "No"
        } else if d0.requests.positive_definite {
            "Yes*"
        } else {
            "Yes"
        };

        // write row start
        write!(&mut buf, "| {mname} | {str_ndim} | {str_nnz} | {str_sym} |").unwrap();

        // fill solver columns
        for genie in data.genies {
            let dat = data.map.get(&(*genie, matrix)).unwrap();

            // extra column for cuDSS
            if *genie == Genie::Cudss {
                write!(&mut buf, " {} |", dat.requests.matching).unwrap();
            }

            if dat.main.out_of_memory {
                write!(&mut buf, " oom | oom |").unwrap();
            } else if dat.time_nanoseconds.total_ifs == 0 {
                write!(&mut buf, " n/a | n/a |").unwrap();
            } else {
                // Set error string
                let str_err = if dat.verify.relative_error > BIG_REL_ERROR {
                    format!("**{:.2e}**", dat.verify.relative_error)
                } else {
                    format!("{:.2e}", dat.verify.relative_error)
                };
                // Set time string (outlier-free)
                let time = data.recalculate_total_time_without_outliers(*genie, matrix).unwrap();
                let time_str = if (time as f64) / 6e+10 > BIG_TIME_MIN {
                    format!("**{}**", fmt_nano(time, 3))
                } else {
                    fmt_nano(time, 3)
                };
                write!(&mut buf, " {time_str} | {str_err} |").unwrap();
            }
        }
        writeln!(&mut buf).unwrap();
    }

    Ok(buf)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AllResults;
    use russell_sparse::Genie;

    #[test]
    fn performance_table_md_works() {
    let data = AllResults::new(
        &[Genie::Cudss, Genie::Mumps, Genie::Umfpack],
        &["bwm2000", "rdb5000", "Goodwin_040", "fp", "helm2d03", "pre2"],
        "results/arch",
    )
        .unwrap();
        let md = performance_table_md(&data).unwrap();
        // verify header structure
        assert!(md.contains("| Matrix | Nrow | NNZ | Sym |"));
        assert!(md.contains("cuDSS MA | cuDSS Time | cuDSS Error |"));
        assert!(md.contains("MUMPS Time | MUMPS Error |"));
        assert!(md.contains("UMFPACK Time | UMFPACK Error |"));
        // verify separator row
        assert!(md.contains("| --- | ---: | ---: | :-: |"));
        // verify matrix entries appear
        for mat in &["bwm2000", "rdb5000", "Goodwin_040", "fp"] {
            assert!(md.contains(mat), "missing matrix {}", mat);
        }
        // verify ndim/nnz present (header col)
        assert!(md.contains("Nrow") && md.contains("NNZ"));
    }
}
