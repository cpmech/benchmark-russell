use crate::{MatricesInfo, StrError};
use russell_sparse::Genie;
use std::fmt::Write;

/// Returns the display name of a solver
pub fn genie_display_name(genie: Genie) -> &'static str {
    match genie {
        Genie::Native => "Native",
        Genie::Umfpack => "UMFPACK",
        Genie::Mumps => "MUMPS",
    }
}

/// Generates a LaTeX itemized list of tested matrices with descriptions from matrices.json
pub fn get_matrix_list_latex(matrices: &[&str]) -> Result<String, StrError> {
    let info = MatricesInfo::new()?;
    let mut buf = String::new();

    for (i, matrix) in matrices.iter().enumerate() {
        let num = i + 1;
        let escaped = matrix.replace("_", "\\_");
        if let Some(m) = info.all.get(*matrix) {
            let group = m.group.replace("_", "\\_");
            let description = m.description.as_deref().unwrap_or("No description available.");
            let description = description
                .replace("_", "\\_")
                .replace("&", "\\&")
                .replace("%", "\\%")
                .replace("#", "\\#");
            writeln!(
                &mut buf,
                "  \\item[$\\text{{{}}}$] \\texttt{{{}}} ({}) -- {}",
                num, escaped, group, description
            )
            .unwrap();
        } else {
            // pres-cylin-3d-tet10-fine (not from SuiteSparse)
            writeln!(
                &mut buf,
                "  \\item[$\\text{{{}}}$] \\texttt{{{}}} (Pedroso) -- FEM stiffness matrix of a pressurized cylinder (Tet10 with 1,711,464 DOF). Not from the Collection. From \\href{{https://onlinelibrary.wiley.com/doi/10.1002/nme.7545}}{{Pedroso DM (2024) Caveats of three direct linear solvers for finite element analyses}}.",
                num, "pres-cylin"
            )
            .unwrap();
        }
    }

    Ok(buf)
}

/// Generates a Markdown list of tested matrices with descriptions from matrices.json
pub fn get_matrix_list_md(matrices: &[&str]) -> Result<String, StrError> {
    let info = MatricesInfo::new()?;
    let mut buf = String::new();

    for (i, matrix) in matrices.iter().enumerate() {
        let num = i + 1;
        if let Some(m) = info.all.get(*matrix) {
            let description = m.description.as_deref().unwrap_or("No description available.");
            writeln!(&mut buf, " {num}. **{matrix}** ({}) -- {description}", m.group).unwrap();
        } else {
            // pres-cylin-3d-tet10-fine (not from SuiteSparse)
            writeln!(
                &mut buf,
                " {num}. **pres-cylin** (Pedroso) -- FEM stiffness matrix of a pressurized cylinder (Tet10 with 1,711,464 DOF). Not from the Collection. From [Pedroso DM (2024) Caveats of three direct linear solvers for finite element analyses](https://onlinelibrary.wiley.com/doi/10.1002/nme.7545)."
            )
            .unwrap();
        }
    }

    Ok(buf)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{genie_display_name, get_matrix_list_md};
    use russell_sparse::Genie;

    #[test]
    fn genie_display_name_works() {
        assert_eq!(genie_display_name(Genie::Native), "Native");
        assert_eq!(genie_display_name(Genie::Umfpack), "UMFPACK");
        assert_eq!(genie_display_name(Genie::Mumps), "MUMPS");
    }

    #[test]
    fn matrix_list_md_works() {
        let list = get_matrix_list_md(&["bwm2000", "pres-cylin-3d-tet10-fine"]).unwrap();
        assert!(list.contains("**bwm2000** (Bai)"));
        assert!(list.contains("**pres-cylin-3d-tet10-fine** (Pedroso)"));
    }
}
