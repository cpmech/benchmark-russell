use benchmark_russell::{AllResults, MATRICES};
use plotpy::{Curve, Legend, Plot, RayEndpoint, Text};
use russell_sparse::*;

const RESULTS_DIR: &str = "results/arch";

fn main() -> Result<(), StrError> {
    let genies = &[Genie::Cudss, Genie::Mumps, Genie::Umfpack];
    let data = AllResults::new(genies, MATRICES, RESULTS_DIR)?;

    let mut x_nnz_mumps = Vec::new();
    let mut x_nnz_umfpack = Vec::new();
    let mut x_nnz_cudss = Vec::new();
    let mut y_time_mumps = Vec::new();
    let mut y_time_umfpack = Vec::new();
    let mut y_time_cudss = Vec::new();

    let mut plot = Plot::new();
    let mut outlier1 = Text::new();
    let mut outlier2 = Text::new();
    let mut ray = Curve::new();
    outlier1
        .set_align_horizontal("right")
        .set_align_vertical("top")
        .set_fontsize(12.0);
    outlier2
        .set_bbox(true)
        .set_bbox_style("square,pad=0.01")
        .set_bbox_edgecolor("white")
        .set_bbox_facecolor("white")
        .set_align_horizontal("center")
        .set_align_vertical("top")
        .set_fontsize(10.0)
        .set_rotation(90.0);
    ray.set_line_color("#c7c7c7").set_line_style("--");

    for matrix in data.matrices {
        let msg = if *matrix == "dielFilterV2real" {
            "dielFilterV2".to_string()
        } else {
            matrix.to_string()
        };
        println!("{}", msg);
        for genie in data.genies {
            let dat = data.map.get(&(*genie, matrix)).unwrap();
            if dat.time_nanoseconds.total_ifs == 0 {
                continue;
            }
            let nnz = dat.matrix.nnz as f64;
            let seconds = (dat.time_nanoseconds.total_ifs as f64) / 1e+9;
            match genie {
                Genie::Mumps => {
                    x_nnz_mumps.push(nnz);
                    y_time_mumps.push(seconds);
                    if msg == "Raj1"
                        || msg == "ASIC_680k"
                        || msg == "PFlow_742"
                        || msg == "atmosmodd"
                        || msg == "rajat31"
                        || msg == "Transport"
                        || msg == "Emilia_923"
                        || msg == "dielFilterV2"
                        || msg == "Flan_1565"
                        || msg == "pres-cylin"
                    {
                        let x = if msg == "Raj1" { nnz - 5.0e5 } else { nnz };
                        outlier2.draw(x, -2.0, &format!("{}", &msg));
                        ray.draw_ray(nnz, 0.0, RayEndpoint::Vertical);
                    }
                }
                Genie::Umfpack => {
                    x_nnz_umfpack.push(nnz);
                    y_time_umfpack.push(seconds);
                    if dat.matrix.name == "PFlow_742" {
                        outlier1.draw(nnz, 149.0, &format!("{:.2}$\\uparrow$", seconds));
                    }
                }
                _ => {
                    x_nnz_cudss.push(nnz);
                    y_time_cudss.push(seconds);
                }
            }
        }
    }

    // generate figure
    let mut curve_mumps = Curve::new();
    let mut curve_umfpack = Curve::new();
    let mut curve_cudss = Curve::new();
    let mut legend = Legend::new();
    legend.set_location("upper center").draw();
    curve_mumps
        .set_label("MUMPS")
        .set_marker_style("+")
        .draw(&x_nnz_mumps, &y_time_mumps);
    curve_umfpack
        .set_label("UMFPACK")
        .set_marker_style("o")
        .draw(&x_nnz_umfpack, &y_time_umfpack);
    curve_cudss
        .set_label("cuDSS")
        .set_marker_style("*")
        .set_line_style("--")
        .draw(&x_nnz_cudss, &y_time_cudss);
    plot.add(&ray)
        .add(&curve_mumps)
        .add(&curve_umfpack)
        .add(&curve_cudss)
        .add(&outlier1)
        .add(&outlier2)
        .add(&legend)
        .set_yrange(-50.0, 150.0)
        .extra("ticks = [tick for tick in plt.gca().get_yticks() if tick >=0]\n")
        .extra("plt.gca().set_yticks(ticks)\n")
        .set_labels("NNZ", "Total time (seconds)")
        .set_figure_size_points(1.618 * 350.0, 350.0)
        .save("/tmp/benchmark-russell/plot_total_time.svg")?;

    Ok(())
}
