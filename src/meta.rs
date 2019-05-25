use clap::{
    App,
    Arg,
    crate_name,
    crate_version,
    crate_authors,
    crate_description,
};

pub fn build_cli() -> App<'static, 'static> {
    App::new(crate_name!())   // Cargo.tomlのnameを参照する
        .version(crate_version!())      // Cargo.tomlのversionを参照する
        .author(crate_authors!())       // Cargo.tomlのauthorsを参照する
        .about(crate_description!())    // Cargo.tomlのdescriptionを参照する
        .arg(Arg::with_name("scale")              // フラグを定義
            .help("Number of truncated after the decimal point")                // ヘルプメッセージ
            .short("s")                         // ショートコマンド
            .long("scale")                       // ロングコマンド
            .value_name("NUMBER")
            .takes_value(true)                  // 値を持つことを定義
        )
        .arg(Arg::with_name("white")              // オプションを定義
            .help("Set White color in a output")              // ヘルプメッセージ
            .short("w")                         // ショートコマンド
            .long("white")                        // ロングコマンド
    )
}