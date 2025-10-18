use clap::Parser;

lazy_static! {
    pub static ref DNTK_OPT: Opts = Opts::parse();
}

#[derive(Parser, Debug)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub struct Opts {
    // Number of truncated after the decimal point
    #[arg(short = 's', long = "scale", default_value_t = 20, help = "Number of truncated after the decimal point")]
    pub scale: usize,

    // Set White color in a output
    #[arg(short = 'w', long = "white", help = "Set White color in a output")]
    pub white: bool,

    // No print information message
    #[arg(short = 'q', long = "quiet", help = "No print information message")]
    pub quiet: bool,

    // Print the local limits enforced by the local version of bc, and quit
    #[arg(long = "show-limits", help = "Print the local limits enforced by the local version of bc, and quit")]
    pub show_limits: bool,

    // bc_path is no longer needed since we're using fasteval internally
    // // Use a specific bc command path
    // #[cfg(not(target_os = "windows"))]
    // #[arg(short = 'b', long = "bc-path", value_parser, default_value = "bc", help = "Use a specific bc command path")]
    // pub bc_path: PathBuf,
    //
    // // Use a specific bc command path
    // #[cfg(target_os = "windows")]
    // #[arg(short = 'b', long = "bc-path", value_parser, default_value = "bc.exe", help = "Use a specific bc command path")]
    // pub bc_path: PathBuf,

    // Pre-run inject statement to the dntk
    #[arg(short = 'i', long = "inject", default_value = "", help = "Pre-run inject statement to the dntk")]
    pub inject: String,

    // Run at only once
    #[arg(long = "once", help = "Run at only once")]
    pub once: bool,
}
