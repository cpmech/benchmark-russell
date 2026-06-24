/// Defines the error output as a static string
pub type StrError = &'static str;

pub mod all_results;
pub mod constants;
pub mod latex;
pub mod markdown;
pub mod matrix_info;
pub mod util;

pub use all_results::*;
pub use constants::*;
pub use latex::*;
pub use markdown::*;
pub use matrix_info::*;
pub use util::*;
