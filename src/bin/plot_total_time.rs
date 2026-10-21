use benchmark_russell::{AllResults, GENIES, MatricesInfo, StrError, genie_display_name};
use plotpy::{Curve, Legend, Plot};
use russell_sparse::Genie;

const RESULTS_DIR: &str = "results/arch";

/// Matrices shown in the figure.
const SELECTED: &[&str] = &[
    "mhd1280b",
    "bwm2000",
    "rdb5000",
    "Goodwin_040",
    "fp",
    "xenon1",
    "mplate",
];

fn main() -> Result<(), StrError> {
    // Sort by NNZ so the rows read from the smallest to the largest matrix.
    let info = MatricesInfo::new()?;
    let mut matrices: Vec<&str> = SELECTED.to_vec();
    matrices.sort_by_key(|m| info.all.get(*m).map(|x| x.pattern_entries).unwrap_or(0));

    let data = AllResults::new(GENIES, &matrices, RESULTS_DIR)?;
    let n = matrices.len();

    // Times per matrix (row) and solver (column), in seconds.
    let mut rows: Vec<Vec<Option<f64>>> = Vec::with_capacity(n);
    for matrix in &matrices {
        let mut times = Vec::with_capacity(GENIES.len());
        for genie in GENIES {
            let t = data
                .get(*genie, *matrix)
                .filter(|d| !d.main.out_of_memory && d.time_nanoseconds.total_ifs > 0)
                .map(|d| d.time_nanoseconds.total_ifs as f64 / 1e9);
            times.push(t);
        }
        rows.push(times);
    }

    let mut plot = Plot::new();

    // One light dumbbell per matrix, spanning the fastest to the slowest solver.
    for (i, times) in rows.iter().enumerate() {
        let mut values: Vec<f64> = times.iter().flatten().copied().collect();
        if values.len() < 2 {
            continue;
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (min, max) = (values[0], values[values.len() - 1]);
        let mut line = Curve::new();
        line.set_line_color("#d9d9d9").set_line_width(2.0).set_line_style("-");
        line.draw(&[min, max], &[i as f64, i as f64]);
        plot.add(&line);
    }

    // One marker series per solver.
    for (gi, genie) in GENIES.iter().enumerate() {
        let mut x = Vec::new();
        let mut y = Vec::new();
        for (i, times) in rows.iter().enumerate() {
            if let Some(t) = times[gi] {
                x.push(t);
                y.push(i as f64);
            }
        }
        let (marker, color) = match genie {
            Genie::Native => ("*", "#111111"),
            Genie::Umfpack => ("o", "#1f77b4"),
            Genie::Mumps => ("+", "#d62728"),
        };
        let mut curve = Curve::new();
        curve
            .set_label(genie_display_name(*genie))
            .set_line_color(color)
            .set_marker_style(marker)
            .set_marker_color(color)
            .set_marker_size(9.0)
            .set_line_style("None");
        curve.draw(&x, &y);
        plot.add(&curve);
    }

    // Categorical y axis: matrix names, with the complex ones marked.
    let ticks: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let labels: Vec<String> = matrices
        .iter()
        .map(|m| {
            let complex = info
                .all
                .get(*m)
                .map(|x| x.data_type.starts_with("complex"))
                .unwrap_or(false);
            if complex { format!("{m} (c)") } else { m.to_string() }
        })
        .collect();
    plot.set_ticks_y_labels(&ticks, &labels);

    // Finish the figure.
    std::fs::create_dir_all("results").map_err(|_| "cannot create results")?;
    let mut legend = Legend::new();
    legend.set_location("lower right").draw();
    plot.add(&legend)
        .extra("plt.xscale('log')\n")
        .extra("plt.grid(True, which='major', axis='x', linestyle=':', alpha=0.5)\n")
        .set_labels("Total time (seconds)", "")
        .set_figure_size_points(1.618 * 380.0, 340.0)
        .save("results/plot_total_time.svg")?;
    plot.save("results/plot_total_time.png")?;

    Ok(())
}
