use clap::{
    App,
    Arg,
    crate_name,
    crate_version,
    crate_authors,
    crate_description,
};

pub fn build_cli() -> App<'static, 'static> {
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
            .help("No print warn message")
            .short("q")
            .long("quiet")
        )
}