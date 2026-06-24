use benchmark_russell::StrError;
use russell_sparse::{get_library_versions, get_system_info_linux};

fn main() -> Result<(), StrError> {
    // write information about the system
    let mut text = String::new();
    text += "\nSystem information:\n\n```\n";
    text += get_system_info_linux().as_str();
    text += "\n```\n";

    // write information about the libraries
    text += "\nLibraries information:\n\n```\n";
    text += get_library_versions().as_str();
    text += "\n```\n";
    println!("{}", text);
    Ok(())
}
