use benchmark_russell::{AllResults, GENIES, MATRICES, MATRICES_COMPLEX, MatricesInfo, StrError, genie_display_name};
use plotpy::{Curve, Legend, Plot, Text};
use russell_sparse::Genie;

const RESULTS_DIR: &str = "results/arch";

/// Matrices shown in the figures.
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

    std::fs::create_dir_all("results").map_err(|_| "cannot create results")?;

    // ---- figure 1: absolute total time ----
    let mut plot_time = Plot::new();
    for (i, times) in rows.iter().enumerate() {
        let mut values: Vec<f64> = times.iter().flatten().copied().collect();
        if values.len() < 2 {
            continue;
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut line = Curve::new();
        line.set_line_color("#d9d9d9").set_line_width(2.0).set_line_style("-");
        line.draw(&[values[0], values[values.len() - 1]], &[i as f64, i as f64]);
        plot_time.add(&line);
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
            Genie::Direct => ("*", "#111111"),
            Genie::Iterative(_) => ("s", "#9467bd"),
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
        plot_time.add(&curve);
    }
    plot_time.set_ticks_y_labels(&ticks, &labels);
    let mut legend1 = Legend::new();
    legend1.set_location("lower right").draw();
    plot_time
        .add(&legend1)
        .extra("plt.xscale('log')\n")
        .extra("plt.grid(True, which='major', axis='x', linestyle=':', alpha=0.5)\n")
        .set_labels("Total time (seconds)", "")
        .set_title("Total time")
        .set_figure_size_points(1.618 * 380.0, 340.0)
        .extra("plt.tight_layout()\n")
        .save("results/plot_time.svg")?;
    plot_time.save("results/plot_time.png")?;

    // ---- figure 2: speed-up over Direct ----
    let mut plot_speed = Plot::new();

    // Parity line: 1x means the same time as Direct.
    let mut parity = Curve::new();
    parity
        .set_label("1×")
        .set_line_color("#888888")
        .set_line_style("--")
        .set_line_width(1.0);
    parity.draw(&[1.0, 1.0], &[-0.5, n as f64 - 0.5]);
    plot_speed.add(&parity);

    // Light span from the smallest to the largest speed-up per matrix.
    for (i, times) in rows.iter().enumerate() {
        let direct = times[0];
        let mut factors: Vec<f64> = [times[1], times[2]]
            .iter()
            .flatten()
            .filter_map(|t| direct.map(|nt| nt / t))
            .collect();
        if factors.len() < 2 {
            continue;
        }
        factors.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut line = Curve::new();
        line.set_line_color("#d9d9d9").set_line_width(2.0).set_line_style("-");
        line.draw(&[factors[0], factors[factors.len() - 1]], &[i as f64, i as f64]);
        plot_speed.add(&line);
    }

    // One marker series per external solver.
    for (gi, marker, color) in [(1usize, "o", "#1f77b4"), (2, "+", "#d62728")] {
        let genie = GENIES[gi];
        let mut x = Vec::new();
        let mut y = Vec::new();
        for (i, times) in rows.iter().enumerate() {
            if let (Some(direct), Some(t)) = (times[0], times[gi]) {
                x.push(direct / t);
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
        plot_speed.add(&curve);
    }
    plot_speed.set_ticks_y_labels(&ticks, &labels);
    let mut legend2 = Legend::new();
    legend2.set_location("lower right").draw();
    plot_speed
        .add(&legend2)
        .extra("plt.xscale('log')\n")
        .extra("plt.grid(True, which='major', axis='x', linestyle=':', alpha=0.5)\n")
        .set_labels("Speed-up over Direct (×)", "")
        .set_title("Speed-up over Direct")
        .set_figure_size_points(1.618 * 380.0, 340.0)
        .extra("plt.tight_layout()\n")
        .save("results/plot_speedup.svg")?;
    plot_speed.save("results/plot_speedup.png")?;

    // ---- figure 3: speed-up over Direct, UMFPACK only ----
    let mut plot_umfpack = Plot::new();
    let mut parity = Curve::new();
    parity
        .set_label("1×")
        .set_line_color("#888888")
        .set_line_style("--")
        .set_line_width(1.0);
    parity.draw(&[1.0, 1.0], &[-0.5, n as f64 - 0.5]);
    plot_umfpack.add(&parity);

    let mut x = Vec::new();
    let mut y = Vec::new();
    for (i, times) in rows.iter().enumerate() {
        if let (Some(direct), Some(t)) = (times[0], times[1]) {
            x.push(direct / t);
            y.push(i as f64);
        }
    }
    let mut curve = Curve::new();
    curve
        .set_label(genie_display_name(Genie::Umfpack))
        .set_line_color("#1f77b4")
        .set_marker_style("o")
        .set_marker_color("#1f77b4")
        .set_marker_size(9.0)
        .set_line_style("None");
    curve.draw(&x, &y);
    plot_umfpack.add(&curve);

    plot_umfpack.set_ticks_y_labels(&ticks, &labels);
    let mut legend3 = Legend::new();
    legend3.set_location("lower right").draw();
    plot_umfpack
        .add(&legend3)
        .extra("plt.xscale('log')\n")
        .extra("plt.grid(True, which='major', axis='x', linestyle=':', alpha=0.5)\n")
        .set_labels("Speed-up over Direct (×)", "")
        .set_title("Speed-up over Direct")
        .set_figure_size_points(1.618 * 380.0, 340.0)
        .extra("plt.tight_layout()\n")
        .save("results/plot_speedup_umfpack.svg")?;
    plot_umfpack.save("results/plot_speedup_umfpack.png")?;

    // ---- figure 4: UMFPACK / MUMPS speed-up for every matrix with both results ----
    let all: Vec<&str> = MATRICES.iter().chain(MATRICES_COMPLEX.iter()).copied().collect();
    let data_all = AllResults::new(&[Genie::Umfpack, Genie::Mumps], &all, RESULTS_DIR)?;
    let mut real_x = Vec::new();
    let mut real_y = Vec::new();
    let mut complex_x = Vec::new();
    let mut complex_y = Vec::new();
    let mut labels_to_add: Vec<(&str, f64, f64)> = Vec::new();
    let mut xmin = f64::INFINITY;
    let mut xmax = 0.0f64;
    for name in &all {
        let (Some(umfpack), Some(mumps)) = (data_all.get(Genie::Umfpack, name), data_all.get(Genie::Mumps, name))
        else {
            continue;
        };
        if umfpack.main.out_of_memory || mumps.main.out_of_memory {
            continue;
        }
        let t_umfpack = umfpack.time_nanoseconds.total_ifs as f64;
        let t_mumps = mumps.time_nanoseconds.total_ifs as f64;
        if t_umfpack == 0.0 || t_mumps == 0.0 {
            continue;
        }
        let Some(meta) = info.all.get(*name) else { continue };
        let x = meta.pattern_entries as f64;
        let y = t_mumps / t_umfpack;
        xmin = xmin.min(x);
        xmax = xmax.max(x);
        if umfpack.matrix.complex {
            complex_x.push(x);
            complex_y.push(y);
        } else {
            real_x.push(x);
            real_y.push(y);
        }
        if matches!(*name, "boyd2" | "ASIC_680k" | "bwm2000" | "mplate") {
            labels_to_add.push((*name, x, y));
        }
    }

    let mut plot_all = Plot::new();
    let mut parity = Curve::new();
    parity
        .set_label("1×")
        .set_line_color("#888888")
        .set_line_style("--")
        .set_line_width(1.0);
    parity.draw(&[xmin * 0.7, xmax * 1.4], &[1.0, 1.0]);
    plot_all.add(&parity);

    for (x, y, label, color, marker) in [
        (&real_x, &real_y, "real", "#1f77b4", "o"),
        (&complex_x, &complex_y, "complex", "#ff7f0e", "s"),
    ] {
        let mut curve = Curve::new();
        curve
            .set_label(label)
            .set_line_color(color)
            .set_marker_style(marker)
            .set_marker_color(color)
            .set_marker_size(8.0)
            .set_line_style("None");
        curve.draw(x, y);
        plot_all.add(&curve);
    }

    for (name, x, y) in labels_to_add {
        let mut text = Text::new();
        text.set_fontsize(8.0)
            .set_color("#333333")
            .set_align_horizontal("left")
            .set_align_vertical("center");
        let ty = if y >= 1.0 { y * 1.15 } else { y * 0.85 };
        text.draw(x * 1.06, ty, name);
        plot_all.add(&text);
    }

    let mut legend4 = Legend::new();
    legend4.set_location("lower left").draw();
    plot_all
        .add(&legend4)
        .extra("plt.xscale('log')\n")
        .extra("plt.yscale('log')\n")
        .extra("plt.grid(True, which='major', linestyle=':', alpha=0.5)\n")
        .set_labels("NNZ", "UMFPACK / MUMPS (×, >1 = UMFPACK faster)")
        .set_title("UMFPACK / MUMPS")
        .set_figure_size_points(1.618 * 420.0, 360.0)
        .extra("plt.tight_layout()\n")
        .save("results/plot_speedup_umfpack_mumps.svg")?;
    plot_all.save("results/plot_speedup_umfpack_mumps.png")?;

    Ok(())
}
