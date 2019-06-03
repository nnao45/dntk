use structopt::StructOpt;
use std::path::PathBuf;

use clap::{
    crate_name,
    crate_version,
    crate_authors,
    crate_description,
};

lazy_static! {
    pub static ref DNTK_OPT: Opts = Opts::from_args();
}

#[derive(StructOpt, Debug)]
#[structopt(
    raw(name = "crate_name!()"),
    raw(version = "crate_version!()"),
    raw(author = "crate_authors!()"),
    raw(about = "crate_description!()"),
)]
pub struct Opts {
    // Number of truncated after the decimal point
    #[structopt(short = "s", long = "scale", default_value = "20", help = "Number of truncated after the decimal point")]
    pub scale: usize,

    // Set White color in a output
    #[structopt(short = "w", long = "white", help = "Set White color in a output")]
    pub white: bool,

    // No print information message
    #[structopt(short = "q", long = "quiet", help = "No print information message")]
    pub quiet: bool,

    // Print the local limits enforced by the local version of bc, and quit
    #[structopt(long = "show-limits", help = "Print the local limits enforced by the local version of bc, and quit")]
    pub show_limits: bool,

    // Use a specific bc command path
    #[cfg(not(target_os = "windows"))]
    #[structopt(short = "b", long = "bc-path", parse(from_os_str), default_value = "bc", help = "Use a specific bc command path")]
    pub bc_path: PathBuf,

    // Use a specific bc command path
    #[cfg(target_os = "windows")]
    #[structopt(short = "b", long = "bc-path", parse(from_os_str), default_value = "bc.exe", help = "Use a specific bc command path")]
    pub bc_path: PathBuf,

    // Pre-run inject statement to the dntk
    #[structopt(short = "i", long = "inject", default_value = "", help = "Pre-run inject statement to the dntk")]
    pub inject: String,

    // Run at only once
    #[structopt(long = "once", help = "Run at only once")]
    pub once: bool,
}