use crate::{MatricesInfo, StrError};
use fancy_regex::Regex;
use std::fmt::Write;

/// Generates a Markdown table with matrix metadata: Name, Nrow, NNZ, Sym
///
/// Reads info from `data/matrices.json` (see `get-matrices-info.bash`).
pub fn matrix_table_md(matrices: &[&str]) -> Result<String, StrError> {
    let info = MatricesInfo::new()?;
    let re = Regex::new(r"\B(?=(\d{3})+(?!\d))").unwrap();

    let mut buf = String::new();

    // header
    writeln!(&mut buf, "| Matrix | Nrow | NNZ | Sym |").unwrap();
    writeln!(&mut buf, "| --- | ---: | ---: | :-: |").unwrap();

    for matrix in matrices {
        let mname = if *matrix == "pres-cylin-3d-tet10-fine-nu499" {
            "pres-cylinNI"
        } else if *matrix == "pres-cylin-3d-tet10-fine" {
            "pres-cylin"
        } else if *matrix == "dielFilterV2real" {
            "dielFilterV2"
        } else {
            matrix
        };

        if let Some(m) = info.all.get(*matrix) {
            let tmp_nrow = format!("{}", m.num_rows);
            let tmp_nnz = format!("{}", m.pattern_entries);
            let str_nrow = re.replace_all(&tmp_nrow, ",");
            let str_nnz = re.replace_all(&tmp_nnz, ",");
            let str_sym = if m.symmetric == "No" {
                "No"
            } else if m.positive_definite.as_deref() == Some("yes") {
                "Yes*"
            } else {
                "Yes"
            };
            writeln!(&mut buf, "| {mname} | {str_nrow} | {str_nnz} | {str_sym} |").unwrap();
        } else {
            // matrix not in SuiteSparse (e.g. pres-cylin)
            writeln!(&mut buf, "| {mname} | — | — | — |").unwrap();
        }
    }

    Ok(buf)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_table_md_works() {
        let md = matrix_table_md(&["bwm2000", "Goodwin_040"]).unwrap();
        // header
        assert!(md.contains("| Matrix | Nrow | NNZ | Sym |"));
        assert!(md.contains("| --- | ---: | ---: | :-: |"));
        // bwm2000
        assert!(md.contains("| bwm2000 | 2,000 | 7,996 | No |"));
        // Goodwin_040
        assert!(md.contains("| Goodwin_040 | 17,922 | 561,677 | No |"));
    }

    #[test]
    fn matrix_table_md_handles_pres_cylin() {
        let md = matrix_table_md(&["pres-cylin-3d-tet10-fine"]).unwrap();
        assert!(md.contains("| pres-cylin | 1,711,464 | 133,562,188 | Yes* |"));
    }
}
