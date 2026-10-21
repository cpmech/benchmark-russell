use benchmark_russell::{AllResults, GENIES, MATRICES, StrError, genie_display_name};
use plotpy::{Curve, Legend, Plot};
use russell_sparse::Genie;

const RESULTS_DIR: &str = "results/arch";

fn main() -> Result<(), StrError> {
    let data = AllResults::new(GENIES, MATRICES, RESULTS_DIR)?;

    // generate one curve per solver
    let mut plot = Plot::new();
    for genie in GENIES {
        let mut x = Vec::new();
        let mut y = Vec::new();
        for matrix in MATRICES.iter() {
            if let Some(dat) = data.get(*genie, *matrix) {
                if dat.main.out_of_memory || dat.time_nanoseconds.total_ifs == 0 {
                    continue;
                }
                x.push(dat.matrix.nnz_actual as f64);
                y.push(dat.time_nanoseconds.total_ifs as f64 / 1e+9);
            }
        }
        let marker = match genie {
            Genie::Native => "*",
            Genie::Umfpack => "o",
            Genie::Mumps => "+",
        };
        let mut curve = Curve::new();
        curve.set_label(genie_display_name(*genie)).set_marker_style(marker);
        curve.draw(&x, &y);
        plot.add(&curve);
    }

    // finish the figure (log scale because Native may be orders of magnitude slower)
    std::fs::create_dir_all("/tmp/benchmark-russell").map_err(|_| "cannot create /tmp/benchmark-russell")?;
    let mut legend = Legend::new();
    legend.set_location("upper left").draw();
    plot.add(&legend)
        .extra("plt.yscale('log')\n")
        .set_labels("NNZ", "Total time (seconds)")
        .set_figure_size_points(1.618 * 350.0, 350.0)
        .save("/tmp/benchmark-russell/plot_total_time.svg")?;

    Ok(())
}
