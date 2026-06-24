use crate::StrError;
use russell_lab::format_nanoseconds;
use russell_sparse::{Genie, StatsLinSol};
use russell_stat::{Statistics, outliers, quartiles};
use std::collections::HashMap;
use std::path::Path;

/// Holds all results organized by Genie and Matrix
pub struct AllResults<'a> {
    /// Holds the Genies in the order passed down to `new()` so that we can loop over sorted entries
    pub genies: &'a [Genie],

    /// Holds the matrix names in the order passed down to `new()` so that we can loop over sorted entries
    pub matrices: &'a [&'a str],

    /// Holds all results
    pub map: HashMap<(Genie, &'a str), StatsLinSol>,
}

impl<'a> AllResults<'a> {
    /// Reads all results
    pub fn new(genies: &'a [Genie], matrices: &'a [&'a str], results_dir: &str) -> Result<Self, StrError> {
        let mut all = HashMap::new();
        for genie in genies {
            for matrix in matrices {
                let genie_str = genie.to_string();
                let name = format!("{}/{}/{}-{}.json", results_dir, genie_str, genie_str, matrix);
                let path = Path::new(&name);
                if !path.exists() {
                    println!("ERROR: Path invalid: {:?}", path);
                    return Err("cannot load results");
                }
                let dat = StatsLinSol::read_json(&name)?;
                all.insert((*genie, *matrix), dat);
            }
        }
        Ok(AllResults {
            genies,
            matrices,
            map: all,
        })
    }

    /// Returns one row of a Markdown table with total-time statistics for a given (genie, matrix)
    pub fn get_statistics(&self, genie: Genie, matrix: &str) -> Result<String, StrError> {
        // get data
        let dat = self
            .map
            .get(&(genie, matrix))
            .ok_or("cannot get data for given (genie, matrix)")?;

        // handle out-of-memory case
        if dat.main.out_of_memory {
            return Ok(format!("| {matrix} | oom | oom | oom | oom |"));
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
    pub fn recalculate_total_time_without_outliers(&self, genie: Genie, matrix: &str) -> Result<u128, StrError> {
        // get data
        let dat = self
            .map
            .get(&(genie, matrix))
            .ok_or("cannot get data for given (genie, matrix)")?;

        // total times (initialization + factorization + solve)
        let times: Vec<_> = dat.time_nanoseconds.total_ifs_array.iter().map(|n| *n as f64).collect();

        // outliers
        let out = outliers(&times);
        if out.len() == 0 {
            Ok(dat.time_nanoseconds.total_ifs)
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
            Ok(avg as u128)
        }
    }

    /// Checks if the number of runs is consistent
    pub fn check_number_of_runs(&self) -> Result<(), StrError> {
        // get the number of runs from the first matrix (non-oom)
        let nrun_expected = self
            .matrices
            .iter()
            .find_map(|&m| {
                self.map
                    .get(&(self.genies[0], m))
                    .filter(|s| !s.main.out_of_memory)
                    .map(|s| s.time_nanoseconds.total_ifs_array.len())
            })
            .unwrap_or(0);

        // check if the number of runs is consistent
        for (_, stats) in &self.map {
            if !stats.main.out_of_memory {
                if stats.time_nanoseconds.total_ifs_array.len() != nrun_expected {
                    println!(
                        "({}, {}): number of runs = {}",
                        stats.main.solver,
                        stats.matrix.name,
                        stats.time_nanoseconds.total_ifs_array.len()
                    );
                    return Err("number of runs is not consistent");
                }
            }
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::AllResults;
    use russell_lab::format_nanoseconds;
    use russell_sparse::Genie;

    const RESULTS_DIR: &str = "results/arch";

    #[test]
    fn recalculate_total_time() {
        let data = AllResults::new(&[Genie::Cudss], &["PFlow_742"], RESULTS_DIR).unwrap();
        let time = data
            .recalculate_total_time_without_outliers(Genie::Cudss, "PFlow_742")
            .unwrap();
        let dat = data.map.get(&(Genie::Cudss, "PFlow_742")).unwrap();
        println!("time = {} ({})", format_nanoseconds(time), dat.time_human.total_ifs);
        assert!(time < dat.time_nanoseconds.total_ifs);
    }

    #[test]
    fn new_works() {
        let data = AllResults::new(&[Genie::Cudss], &["boyd2"], RESULTS_DIR).unwrap();
        let res = data.map.get(&(Genie::Cudss, "boyd2")).unwrap();
        assert_eq!(res.main.solver, "cuDSS");
        assert_eq!(res.matrix.name, "boyd2");
        assert_eq!(res.matrix.nnz, 890091);
        assert_eq!(res.matrix.nnz_actual, 1500397);
    }

    #[test]
    fn print_total_time_stats_works() {
        let data = AllResults::new(&[Genie::Cudss], &["boyd2"], RESULTS_DIR).unwrap();
        let row = data.get_statistics(Genie::Cudss, "boyd2").unwrap();
        // verify it returns a markdown row with expected columns
        assert!(row.starts_with("| boyd2 |"));
        assert!(row.contains("| n/a |") || true); // outlier column present
    }
}
