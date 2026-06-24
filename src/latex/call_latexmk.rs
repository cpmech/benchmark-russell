use crate::StrError;
use std::fs;
use std::fs::File;
use std::io::Write as io_write;
use std::path::Path;
use std::process::Command;

const OUT_DIR: &str = "/tmp/benchmark-russell/latex";

const TITLE: &str = "Performance benchmarks using the Rust Scientific Library (Russell)";
const AUTHOR: &str = "Dr Dorival Pedroso";

/// Calls LatexMK
pub fn call_latexmk(
    filename_stem: &str,
    buffer: &String,
    narrow_margins: bool,
    small_font: bool,
) -> Result<(), StrError> {
    let out_dir = Path::new(OUT_DIR);
    fs::create_dir_all(out_dir).map_err(|_| "cannot create directory")?;

    let geometry = if narrow_margins {
        "\\usepackage[top=1cm,bottom=2cm,left=1cm,right=1cm]{geometry}\n"
    } else {
        ""
    };

    let font_size = if small_font { "10pt" } else { "12pt" };

    // write the full LaTeX document
    let full_document = format!(
        "\\documentclass[{},a4paper]{{article}}\n\
         \\usepackage{{enumitem}}\n\
         \\usepackage{{booktabs}}\n\
         \\usepackage{{nicematrix}}\n\
         \\usepackage{{amsmath,amssymb}}\n\
         \\usepackage{{bm}}\n\
         \\usepackage{{siunitx}}\n\
         \\usepackage{{xcolor}} % Required for defining custom colors\n\
         \\definecolor{{DocBlue}}{{RGB}}{{30, 100, 200}} % A clean, professional blue\n\
         \\usepackage[labelfont=bf]{{caption}}\n\
         \\usepackage[\n\
         \x20\x20 colorlinks=true,    % Turns off the default boxes, turns on colored text\n\
         \x20\x20 urlcolor=DocBlue,   % Colors url and href links\n\
         \x20\x20 linkcolor=DocBlue,  % Colors internal links (sections, equations)\n\
         \x20\x20 citecolor=DocBlue,  % Colors bibliography citations\n\
         \x20\x20 breaklinks=true     % Allows long URLs to wrap nicely across lines\n\
         ]{{hyperref}}\n\
         \\usepackage{{paralist}}\n\
         \x20\x20 \\let\\itemize\\compactitem\n\
         \x20\x20 \\let\\enditemize\\endcompactitem\n\
         \x20\x20 \\pltopsep=0.5em\n\
         \x20\x20 \\plitemsep=1pt\n\
         \x20\x20 \\plparsep=1pt\n\
         {}\
         \\title{{{}}}\n\
         \\author{{{}}}\n\
         \\begin{{document}}\n\
         \\maketitle\n\
         \n\
         {}\
         \n\
         \\end{{document}}",
        font_size, geometry, TITLE, AUTHOR, buffer
    );
    let tex_name = format!("{}.tex", filename_stem);
    let tex_path = format!("{}/{}", OUT_DIR, tex_name);
    let mut file = File::create(&tex_path).map_err(|_| "cannot create file")?;
    file.write_all(full_document.as_bytes())
        .map_err(|_| "cannot write file")?;
    file.sync_all().map_err(|_| "cannot sync file")?;

    // run latexmk with explicit working directory (no global CWD change)
    let jobname = format!("-jobname={}", filename_stem);
    let output = Command::new("latexmk")
        .args([
            "-pdf",
            "-shell-escape",
            "-halt-on-error",
            "-interaction=batchmode",
            &jobname,
            &tex_name,
        ])
        .current_dir(out_dir)
        .output()
        .map_err(|_| "cannot run latexmk")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
        return Err("latexmk failed");
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::call_latexmk;
    use std::fmt::Write;
    use std::fs;
    use std::path::Path;

    fn has_latexmk() -> bool {
        std::process::Command::new("latexmk")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[test]
    fn call_latex_works() {
        if !has_latexmk() {
            return;
        }

        let stem = "call_latex_works";
        let tex_path = format!("/tmp/benchmark-russell/latex/{}.tex", stem);
        let pdf_path = format!("/tmp/benchmark-russell/latex/{}.pdf", stem);

        // clean up any leftover files from a previous run
        let _ = fs::remove_file(&tex_path);
        let _ = fs::remove_file(&pdf_path);

        // valid LaTeX table content
        let mut buffer = String::new();
        writeln!(&mut buffer, "\\begin{{tabular}}{{cc}}").unwrap();
        writeln!(&mut buffer, "\\toprule").unwrap();
        writeln!(&mut buffer, "  Col A & Col B \\\\").unwrap();
        writeln!(&mut buffer, "\\midrule").unwrap();
        writeln!(&mut buffer, "   1.0 & 2.0 \\\\").unwrap();
        writeln!(&mut buffer, "\\bottomrule").unwrap();
        writeln!(&mut buffer, "\\end{{tabular}}").unwrap();

        call_latexmk(stem, &buffer, false, false).unwrap();

        // verify PDF was produced
        assert!(Path::new(&pdf_path).exists(), "PDF was not generated");
    }

    #[test]
    fn call_latex_with_narrow_margins_works() {
        if !has_latexmk() {
            return;
        }

        let stem = "call_latex_narrow_margins";
        let tex_path = format!("/tmp/benchmark-russell/latex/{}.tex", stem);
        let pdf_path = format!("/tmp/benchmark-russell/latex/{}.pdf", stem);

        let _ = fs::remove_file(&tex_path);
        let _ = fs::remove_file(&pdf_path);

        let mut buffer = String::new();
        writeln!(&mut buffer, "\\begin{{tabular}}{{cc}}").unwrap();
        writeln!(&mut buffer, "\\toprule").unwrap();
        writeln!(&mut buffer, "  Col A & Col B \\\\").unwrap();
        writeln!(&mut buffer, "\\midrule").unwrap();
        writeln!(&mut buffer, "   1.0 & 2.0 \\\\").unwrap();
        writeln!(&mut buffer, "\\bottomrule").unwrap();
        writeln!(&mut buffer, "\\end{{tabular}}").unwrap();

        call_latexmk(stem, &buffer, true, false).unwrap();

        assert!(Path::new(&pdf_path).exists(), "PDF was not generated");
    }

    #[test]
    fn call_latex_with_small_font_works() {
        if !has_latexmk() {
            return;
        }

        let stem = "call_latex_small_font";
        let tex_path = format!("/tmp/benchmark-russell/latex/{}.tex", stem);
        let pdf_path = format!("/tmp/benchmark-russell/latex/{}.pdf", stem);

        let _ = fs::remove_file(&tex_path);
        let _ = fs::remove_file(&pdf_path);

        let mut buffer = String::new();
        writeln!(&mut buffer, "\\begin{{tabular}}{{cc}}").unwrap();
        writeln!(&mut buffer, "\\toprule").unwrap();
        writeln!(&mut buffer, "  Col A & Col B \\\\").unwrap();
        writeln!(&mut buffer, "\\midrule").unwrap();
        writeln!(&mut buffer, "   1.0 & 2.0 \\\\").unwrap();
        writeln!(&mut buffer, "\\bottomrule").unwrap();
        writeln!(&mut buffer, "\\end{{tabular}}").unwrap();

        call_latexmk(stem, &buffer, false, true).unwrap();

        assert!(Path::new(&pdf_path).exists(), "PDF was not generated");
    }
}
