use clap::{
    App,
    Arg,
    crate_name,
    crate_version,
    crate_authors,
    crate_description,
};

pub fn build_cli() -> App<'static, 'static> {
    #[cfg(not(target_os = "windows"))]
    let bc_path = "bc";
    #[cfg(target_os = "windows")]
    let bc_path = "bc.exe";

    App::new(crate_name!())
        .version(crate_version!())
        .version_short("v")
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("scale")
            .help("Number of truncated after the decimal point")
            .short("s")
            .long("scale")
            .value_name("NUMBER")
            .takes_value(true)
        )
        .arg(Arg::with_name("white")
            .help("Set White color in a output")
            .short("w")
            .long("white")
        )
        .arg(Arg::with_name("quiet")
            .help("No print information message")
            .short("q")
            .long("quiet")
        )
        .arg(Arg::with_name("show-limits")
            .help("Print the local limits enforced by the local version of bc, and quit")
            .long("show-limits")
        )
        .arg(Arg::with_name("bc-path")
            .help("Use a specific bc command path")
            .long("bc-path")
            .value_name("PATH")
            .takes_value(true)
            .default_value(bc_path)
        )
        .arg(Arg::with_name("inject")
            .help("First injection statement to the dntk")
            .short("i")
            .long("inject")
            .value_name("STRING")
            .takes_value(true)
        )
}