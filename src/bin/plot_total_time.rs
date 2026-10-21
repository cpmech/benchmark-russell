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

    let mut plot = Plot::new();

    // ---- panel 1: absolute total time ----
    plot.set_subplot(1, 2, 1);
    for (i, times) in rows.iter().enumerate() {
        let mut values: Vec<f64> = times.iter().flatten().copied().collect();
        if values.len() < 2 {
            continue;
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut line = Curve::new();
        line.set_line_color("#d9d9d9").set_line_width(2.0).set_line_style("-");
        line.draw(&[values[0], values[values.len() - 1]], &[i as f64, i as f64]);
        plot.add(&line);
    }
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
    plot.set_ticks_y_labels(&ticks, &labels);
    let mut legend1 = Legend::new();
    legend1.set_location("lower right").draw();
    plot.add(&legend1)
        .extra("plt.xscale('log')\n")
        .extra("plt.grid(True, which='major', axis='x', linestyle=':', alpha=0.5)\n")
        .set_labels("Total time (seconds)", "")
        .set_title("Total time");

    // ---- panel 2: speed-up over Native ----
    plot.set_subplot(1, 2, 2);

    // Parity line: 1x means the same time as Native.
    let mut parity = Curve::new();
    parity
        .set_label("1×")
        .set_line_color("#888888")
        .set_line_style("--")
        .set_line_width(1.0);
    parity.draw(&[1.0, 1.0], &[-0.5, n as f64 - 0.5]);
    plot.add(&parity);

    // Light span from the smallest to the largest speed-up per matrix.
    for (i, times) in rows.iter().enumerate() {
        let native = times[0];
        let mut factors: Vec<f64> = [times[1], times[2]]
            .iter()
            .flatten()
            .filter_map(|t| native.map(|nt| nt / t))
            .collect();
        if factors.len() < 2 {
            continue;
        }
        factors.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut line = Curve::new();
        line.set_line_color("#d9d9d9").set_line_width(2.0).set_line_style("-");
        line.draw(&[factors[0], factors[factors.len() - 1]], &[i as f64, i as f64]);
        plot.add(&line);
    }

    // One marker series per external solver.
    for (gi, marker, color) in [(1usize, "o", "#1f77b4"), (2, "+", "#d62728")] {
        let genie = GENIES[gi];
        let mut x = Vec::new();
        let mut y = Vec::new();
        for (i, times) in rows.iter().enumerate() {
            if let (Some(native), Some(t)) = (times[0], times[gi]) {
                x.push(native / t);
                y.push(i as f64);
            }
        }
        let mut curve = Curve::new();
        curve
            .set_label(genie_display_name(genie))
            .set_line_color(color)
            .set_marker_style(marker)
            .set_marker_color(color)
            .set_marker_size(9.0)
            .set_line_style("None");
        curve.draw(&x, &y);
        plot.add(&curve);
    }
    plot.set_ticks_y_labels(&ticks, &labels);
    let mut legend2 = Legend::new();
    legend2.set_location("lower right").draw();
    plot.add(&legend2)
        .extra("plt.xscale('log')\n")
        .extra("plt.grid(True, which='major', axis='x', linestyle=':', alpha=0.5)\n")
        .set_labels("Speed-up over Native (×)", "")
        .set_title("Speed-up over Native");

    // ---- finish ----
    std::fs::create_dir_all("results").map_err(|_| "cannot create results")?;
    plot.set_figure_size_points(1.618 * 720.0, 360.0)
        .extra("plt.tight_layout()\n")
        .save("results/plot_total_time.svg")?;
    plot.save("results/plot_total_time.png")?;

    Ok(())
}
