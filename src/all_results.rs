use crate::StrError;
use russell_lab::format_nanoseconds;
use russell_sparse::{Genie, StatsLinSol};
use russell_stat::{Statistics, outliers, quartiles};
use std::collections::HashMap;
use std::path::Path;

/// Holds all results organized by Genie and Matrix
///
/// A matrix/solver pair without a result file (or with a file that cannot be
/// parsed, such as the 2.8 JSONs kept under `results/legacy-2.8`) is stored as
/// `None` and rendered as `n/a` by the table generators.
pub struct AllResults<'a> {
    /// Holds the Genies in the order passed down to `new()` so that we can loop over sorted entries
    pub genies: &'a [Genie],

    /// Holds the matrix names in the order passed down to `new()` so that we can loop over sorted entries
    pub matrices: &'a [&'a str],

    /// Holds all results; `None` means the run is missing or not readable
    pub map: HashMap<(Genie, String), Option<StatsLinSol>>,
}

impl<'a> AllResults<'a> {
    /// Reads all results, tolerating missing or unreadable files
    pub fn new(genies: &'a [Genie], matrices: &'a [&'a str], results_dir: &str) -> Result<Self, StrError> {
        let mut all = HashMap::new();
        for genie in genies {
            for matrix in matrices {
                let genie_str = genie.to_string();
                let name = format!("{}/{}/{}-{}.json", results_dir, genie_str, genie_str, matrix);
                let dat = if Path::new(&name).exists() {
                    match StatsLinSol::read_json(&name) {
                        Ok(dat) => Some(dat),
                        Err(_) => {
                            println!("WARNING: cannot parse {name}; treating it as n/a");
                            None
                        }
                    }
                } else {
                    None
                };
                all.insert((*genie, matrix.to_string()), dat);
            }
        }
        Ok(AllResults {
            genies,
            matrices,
            map: all,
        })
    }

    /// Returns the result for a given (genie, matrix), if available
    pub fn get(&self, genie: Genie, matrix: &str) -> Option<&StatsLinSol> {
        self.map.get(&(genie, matrix.to_string())).and_then(|dat| dat.as_ref())
    }

    /// Returns true if the result holds at least one measured run
    fn has_runs(dat: &StatsLinSol) -> bool {
        !dat.main.out_of_memory && !dat.time_nanoseconds.total_ifs_array.is_empty()
    }

    /// Returns one row of a Markdown table with total-time statistics for a given (genie, matrix)
    pub fn get_statistics(&self, genie: Genie, matrix: &str) -> Result<String, StrError> {
        let dat = match self.get(genie, matrix) {
            Some(dat) => dat,
            None => return Ok(format!("| {matrix} | n/a | n/a | n/a | n/a |")),
        };

        // handle out-of-memory case
        if dat.main.out_of_memory {
            return Ok(format!("| {matrix} | oom | oom | oom | oom |"));
        }

        // handle no-runs case
        if !Self::has_runs(dat) {
            return Ok(format!("| {matrix} | n/a | n/a | n/a | n/a |"));
        }

        // total times (initialization + factorization + solve)
        let mut times: Vec<_> = dat.time_nanoseconds.total_ifs_array.iter().map(|n| *n as f64).collect();

        // basic statistics
        let stat = Statistics::new(&times);
        let min = format_nanoseconds(stat.min as u128);
        let max = format_nanoseconds(stat.max as u128);

        // quartiles (q2 = median)
        let (_, q2, _) = quartiles(&mut times);
        let median = format_nanoseconds(q2 as u128);

        // outliers (first one or "n/a")
        let out = outliers(&times);
        let outlier = if let Some((_, val)) = out.first() {
            format_nanoseconds(*val as u128)
        } else {
            "n/a".to_string()
        };

        Ok(format!("| {matrix} | {min} | {median} | {max} | {outlier} |"))
    }

    /// Recalculates the total time without outliers
    ///
    /// Returns `None` when the run is missing, out of memory, or has no samples.
    pub fn recalculate_total_time_without_outliers(&self, genie: Genie, matrix: &str) -> Option<u128> {
        let dat = self.get(genie, matrix)?;
        if !Self::has_runs(dat) {
            return None;
        }

        // total times (initialization + factorization + solve)
        let times: Vec<_> = dat.time_nanoseconds.total_ifs_array.iter().map(|n| *n as f64).collect();

        // outliers
        let out = outliers(&times);
        if out.is_empty() {
            Some(dat.time_nanoseconds.total_ifs)
        } else {
            // indices of outliers
            let outlier_indices: Vec<usize> = out.iter().map(|(i, _)| *i).collect();
            // filter out outliers
            let clean: Vec<f64> = times
                .iter()
                .enumerate()
                .filter(|(i, _)| !outlier_indices.contains(i))
                .map(|(_, v)| *v)
                .collect();
            // average of clean times
            let avg = clean.iter().sum::<f64>() / clean.len() as f64;
            Some(avg as u128)
        }
    }

    /// Prints a short summary of the number of runs recorded per solver
    pub fn print_run_counts(&self) {
        for genie in self.genies {
            let mut counts: Vec<usize> = Vec::new();
            let mut measured = 0usize;
            for matrix in self.matrices.iter() {
                if let Some(dat) = self.get(*genie, *matrix) {
                    if Self::has_runs(dat) {
                        measured += 1;
                        counts.push(dat.time_nanoseconds.total_ifs_array.len());
                    }
                }
            }
            counts.sort_unstable();
            counts.dedup();
            eprintln!("... {genie}: {measured} measured matrices, run counts {counts:?}");
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::AllResults;
    use russell_sparse::Genie;

    const RESULTS_DIR: &str = "results/arch";

    #[test]
    fn new_tolerates_missing_results() {
        let data = AllResults::new(&[Genie::Direct], &["this-matrix-does-not-exist"], RESULTS_DIR).unwrap();
        assert!(
            data.map
                .contains_key(&(Genie::Direct, "this-matrix-does-not-exist".to_string()))
        );
        assert!(data.get(Genie::Direct, "this-matrix-does-not-exist").is_none());
        let row = data
            .get_statistics(Genie::Direct, "this-matrix-does-not-exist")
            .unwrap();
        assert_eq!(row, "| this-matrix-does-not-exist | n/a | n/a | n/a | n/a |");
    }

    #[test]
    fn recalculate_total_time_works_when_data_exists() {
        let data = AllResults::new(&[Genie::Umfpack], &["bwm2000"], RESULTS_DIR).unwrap();
        let Some(dat) = data.get(Genie::Umfpack, "bwm2000") else {
            println!("skipping: no 4.0 result for bwm2000/umfpack yet");
            return;
        };
        let time = data
            .recalculate_total_time_without_outliers(Genie::Umfpack, "bwm2000")
            .unwrap();
        println!("time = {}", time);
        assert!(!dat.time_nanoseconds.total_ifs_array.is_empty());
        assert!(time <= dat.time_nanoseconds.total_ifs);
    }

    #[test]
    fn new_works_when_data_exists() {
        let data = AllResults::new(&[Genie::Umfpack], &["boyd2"], RESULTS_DIR).unwrap();
        let Some(res) = data.get(Genie::Umfpack, "boyd2") else {
            println!("skipping: no 4.0 result for boyd2/umfpack yet");
            return;
        };
        assert_eq!(res.main.solver, "UMFPACK");
        assert_eq!(res.matrix.name, "boyd2");
        assert_eq!(res.matrix.nnz, 1500397);
        assert_eq!(res.matrix.nnz_actual, 1500397);
    }

    #[test]
    fn print_total_time_stats_works_when_data_exists() {
        let data = AllResults::new(&[Genie::Umfpack], &["boyd2"], RESULTS_DIR).unwrap();
        if data.get(Genie::Umfpack, "boyd2").is_none() {
            println!("skipping: no 4.0 result for boyd2/umfpack yet");
            return;
        }
        let row = data.get_statistics(Genie::Umfpack, "boyd2").unwrap();
        assert!(row.starts_with("| boyd2 |"));
    }
}
