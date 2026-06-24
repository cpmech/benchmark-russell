use crate::{MatricesInfo, StrError};
use std::fmt::Write;
use std::process::Command;

pub fn get_key_for_matching_algorithm(matching: &str) -> String {
    // No matching algorithm (default)
    //
    // CUDSS_MATCHING_ALG_NONE
    if matching == "None" {
        return "a".to_string();
    }

    // Automatic selection
    //
    // Same as CUDSS_MATCHING_ALG_MAX_DIAG_PRODUCT (the most robust option).
    // Matching with scaling; requires matrix values during analysis.
    //
    // CUDSS_MATCHING_ALG_AUTO
    if matching == "Auto" {
        return "b".to_string();
    }

    // Column permutation to maximize the number of diagonal entries (values arbitrary).
    // MC64 JOB=1. Does not use matrix values; not recommended unless justified.
    //
    // CUDSS_MATCHING_ALG_MAX_DIAG_COUNT
    if matching == "MaxDiagCount" {
        return "c".to_string();
    }

    // Column permutation to maximize the smallest value on the diagonal. MC64 JOB=2.
    //
    // CUDSS_MATCHING_ALG_MAX_MIN_DIAG
    if matching == "MaxMinDiag" {
        return "d".to_string();
    }

    // Alternate algorithm to maximize the smallest value on the diagonal. MC64 JOB=3.
    // May differ in performance from CUDSS_MATCHING_ALG_MAX_MIN_DIAG.
    //
    // CUDSS_MATCHING_ALG_MAX_MIN_DIAG_ALT
    if matching == "MaxMinDiagAlt" {
        return "e".to_string();
    }

    // Column permutation to maximize the sum of diagonal entries. MC64 JOB=4.
    //
    // CUDSS_MATCHING_ALG_MAX_DIAG_SUM
    if matching == "MaxDiagSum" {
        return "f".to_string();
    }

    // Column permutation to maximize the product of diagonal entries;
    // also computes row/column scaling so that nonzero diagonal entries are 1 in
    // absolute value and off-diagonal entries are ≤ 1. MC64 JOB=5. Most impactful for accuracy;
    // requires matrix values during analysis.
    //
    // CUDSS_MATCHING_ALG_MAX_DIAG_PRODUCT
    if matching == "MaxDiagProduct" {
        return "g".to_string();
    }

    return "h".to_string();
}

pub fn get_description_for_matching_algorithm(matching: &str) -> String {
    // No matching algorithm (default).
    if matching == "None" {
        return "No matching algorithm (default).".to_string();
    }

    // Automatic selection.
    if matching == "Auto" {
        return "Automatic selection.".to_string();
    }

    // Column permutation to maximize the number of diagonal entries.
    if matching == "MaxDiagCount" {
        return "Column permutation to maximize the number of diagonal entries.".to_string();
    }

    // Column permutation to maximize the smallest value on the diagonal.
    if matching == "MaxMinDiag" {
        return "Column permutation to maximize the smallest value on the diagonal.".to_string();
    }

    // Alternate algorithm to maximize the smallest value on the diagonal.
    if matching == "MaxMinDiagAlt" {
        return "Alternate algorithm to maximize the smallest value on the diagonal.".to_string();
    }

    // Column permutation to maximize the sum of diagonal entries.
    if matching == "MaxDiagSum" {
        return "Column permutation to maximize the sum of diagonal entries.".to_string();
    }

    // Column permutation to maximize the product of diagonal entries.
    if matching == "MaxDiagProduct" {
        return "Column permutation to maximize the product of diagonal entries.".to_string();
    }

    return "Unknown".to_string();
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

/// Discovers the NVIDIA driver version by running `nvidia-smi`
pub fn get_nvidia_driver_version() -> Result<String, StrError> {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=driver_version", "--format=csv,noheader"])
        .output()
        .map_err(|_| "failed to execute nvidia-smi")?;

    if !output.status.success() {
        return Err("nvidia-smi command failed");
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if version.is_empty() {
        return Err("nvidia driver version not found");
    }

    Ok(version)
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
    use super::get_nvidia_driver_version;

    fn has_nvidia_smi() -> bool {
        std::process::Command::new("nvidia-smi")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[test]
    fn get_nvidia_driver_version_works() {
        if !has_nvidia_smi() {
            return;
        }
        let version = get_nvidia_driver_version().unwrap();
        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }
}
