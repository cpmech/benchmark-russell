use crate::constants::{BIG_REL_ERROR, BIG_TIME_MIN};
use crate::{AllResults, StrError, genie_display_name};
use russell_lab::format_nanoseconds_with_digits as fmt_nano;
use std::fmt::Write;

/// Generates a Markdown comparison table for the linear solvers
///
/// Times and errors exceeding the configured thresholds are rendered **bold**.
/// Handles out-of-memory (`oom`) and not-available (`n/a`).
pub fn performance_table_md<'a>(data: &AllResults<'a>) -> Result<String, StrError> {
    // ----- validate input -----
    if data.map.is_empty() {
        return Err("results map must not be empty");
    }

    // ----- build header row: solver names spanning columns -----
    let mut buf = String::new();
    write!(&mut buf, "| Matrix |").unwrap();
    for genie in data.genies {
        let display = genie_display_name(*genie);
        write!(&mut buf, " {display} Time | {display} Error |").unwrap();
    }
    writeln!(&mut buf).unwrap();

    // ----- separator row with alignment -----
    write!(&mut buf, "| --- |").unwrap();
    for _ in data.genies {
        write!(&mut buf, " ---: | ---: |").unwrap();
    }
    writeln!(&mut buf).unwrap();

    // ----- process each matrix -----
    for matrix in data.matrices.iter() {
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

        // write row start
        write!(&mut buf, "| {mname} |").unwrap();

        // fill solver columns
        for genie in data.genies {
            match data.get(*genie, *matrix) {
                None => {
                    write!(&mut buf, " n/a | n/a |").unwrap();
                }
                Some(dat) if dat.main.out_of_memory => {
                    write!(&mut buf, " oom | oom |").unwrap();
                }
                Some(dat) if dat.time_nanoseconds.total_ifs_array.is_empty() => {
                    write!(&mut buf, " n/a | n/a |").unwrap();
                }
                Some(dat) => {
                    // Set error string
                    let str_err = if dat.verify.relative_error > BIG_REL_ERROR {
                        format!("**{:.2e}**", dat.verify.relative_error)
                    } else {
                        format!("{:.2e}", dat.verify.relative_error)
                    };
                    // Set time string (outlier-free)
                    match data.recalculate_total_time_without_outliers(*genie, *matrix) {
                        Some(time) => {
                            let time_str = if (time as f64) / 6e+10 > BIG_TIME_MIN {
                                format!("**{}**", fmt_nano(time, 3))
                            } else {
                                fmt_nano(time, 3)
                            };
                            write!(&mut buf, " {time_str} | {str_err} |").unwrap();
                        }
                        None => {
                            write!(&mut buf, " n/a | n/a |").unwrap();
                        }
                    }
                }
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
            &[Genie::Direct, Genie::Umfpack, Genie::Mumps],
            &["bwm2000", "rdb5000", "Goodwin_040", "fp", "helm2d03", "pre2"],
            "results/arch",
        )
        .unwrap();
        let md = performance_table_md(&data).unwrap();
        // verify header structure (no Nrow/NNZ/Sym columns)
        assert!(md.contains("| Matrix |"));
        assert!(!md.contains("| Nrow |"));
        assert!(!md.contains("| NNZ |"));
        assert!(!md.contains("| Sym |"));
        assert!(md.contains("Direct Time | Direct Error |"));
        assert!(md.contains("UMFPACK Time | UMFPACK Error |"));
        assert!(md.contains("MUMPS Time | MUMPS Error |"));
        // verify separator row
        assert!(md.contains("| --- | ---: | ---: | ---: | ---: |"));
        // verify matrix entries appear
        for mat in &["bwm2000", "rdb5000", "Goodwin_040", "fp"] {
            assert!(md.contains(mat), "missing matrix {}", mat);
        }
    }
}
