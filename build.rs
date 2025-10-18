fn main() {
    let target = std::env::var("TARGET").unwrap();

    // Linux環境の場合、libcの種類に応じて適切なファイルを指定
    if target.contains("linux") {
        let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_else(|_| "gnu".to_string());

        let linux_term_lib = match target_env.as_str() {
            "musl" => "linux_musl.rs",
            "gnu" => "linux_gnu.rs",
            // デフォルトはgnu (ほとんどのLinuxディストリビューションで使用)
            _ => "linux_gnu.rs",
        };

        println!("cargo:rustc-env=LINUX_TERM_LIB={}", linux_term_lib);
    }
}
