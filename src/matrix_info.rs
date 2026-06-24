use crate::StrError;
use serde::Deserialize;
use std::collections::HashMap;

const PATH: &str = "data/matrices.json";

/// Holds metadata about a matrix from the SuiteSparse Matrix Collection
///
/// This info is obtained by running the `get-matrices-info.bash` script,
/// which scrapes `https://sparse.tamu.edu` and saves the results to
/// `data/matrices.json`.
#[derive(Clone, Debug, Deserialize)]
pub struct MatrixInfo {
    /// Matrix name (e.g. `"bwm2000"`)
    #[serde(rename = "Name")]
    pub name: String,

    /// Group / collection name (e.g. `"Bai"`)
    #[serde(rename = "Group")]
    pub group: String,

    /// Number of rows
    #[serde(rename = "Num Rows")]
    pub num_rows: u64,

    /// Number of columns
    #[serde(rename = "Num Cols")]
    pub num_cols: u64,

    /// Number of numerically nonzero entries
    #[serde(rename = "Nonzeros")]
    pub nonzeros: u64,

    /// Number of pattern entries (includes explicit zeros)
    #[serde(rename = "Pattern Entries")]
    pub pattern_entries: u64,

    /// Problem kind / domain (e.g. `"Computational Fluid Dynamics Problem"`)
    #[serde(rename = "Kind")]
    pub kind: String,

    /// Whether the matrix is symmetric (`"Yes"` or `"No"`)
    #[serde(rename = "Symmetric")]
    pub symmetric: String,

    /// Whether the matrix is positive-definite (`"yes"`, `"no"`, or `null`)
    ///
    /// **Note:** This field may be absent for historical matrices.
    #[serde(rename = "Positive Definite", default)]
    pub positive_definite: Option<String>,

    /// Element type (`"real"`, `"complex"`, `"integer"`, or `"binary"`)
    #[serde(rename = "Type")]
    pub data_type: String,

    /// Human-readable description of the matrix (may be `null`)
    ///
    /// **Note:** This field may be absent for some matrices.
    #[serde(rename = "Description", default)]
    pub description: Option<String>,
}

pub struct MatricesInfo {
    /// Holds the map name to matrix information
    pub all: HashMap<String, MatrixInfo>,
}

impl MatricesInfo {
    /// Reads information about the SuiteSparse Matrices recorded in `data/matrices.json`
    pub fn new() -> Result<Self, StrError> {
        let content = std::fs::read_to_string(PATH).map_err(|_| "cannot read data/matrices.json")?;
        let vec: Vec<MatrixInfo> = serde_json::from_str(&content).map_err(|_| "cannot parse data/matrices.json")?;
        let mut all = HashMap::new();
        for mat in vec {
            all.insert(mat.name.clone(), mat);
        }
        Ok(MatricesInfo { all })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrices_info_works() {
        let info = MatricesInfo::new().unwrap();
        assert!(!info.all.is_empty(), "should have at least one entry");

        // check first entry (bwm2000)
        let bwm = info.all.get("bwm2000").expect("bwm2000 not found");
        assert_eq!(bwm.group, "Bai");
        assert_eq!(bwm.num_rows, 2000);
        assert_eq!(bwm.num_cols, 2000);
        assert_eq!(bwm.nonzeros, 7996);
        assert_eq!(bwm.pattern_entries, 7996);
        assert_eq!(bwm.kind, "Chemical Process Simulation Problem");
        assert_eq!(bwm.symmetric, "No");
        assert_eq!(bwm.positive_definite.as_deref(), Some("no"));
        assert_eq!(bwm.data_type, "real");
        assert_eq!(
            bwm.description.as_deref(),
            Some("Brusselator wave model in transport interaction of chemical solutions (1992)")
        );

        // check last entry (Flan_1565)
        let flan = info.all.get("Flan_1565").expect("Flan_1565 not found");
        assert_eq!(flan.group, "Janna");
        assert_eq!(flan.num_rows, 1564794);
        assert_eq!(flan.symmetric, "Yes");
        assert_eq!(flan.positive_definite.as_deref(), Some("yes"));
    }
}
