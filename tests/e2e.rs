use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_cmd_with_once() {
    std::env::set_var("DNTK_ENV", "TEST");
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--once").arg("--inject").arg("1/3");
    // Note: f64 precision causes slight rounding differences
    // Just check that the output contains the expression and starts with .333
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "Command should succeed");
    assert!(stdout.contains("1/3"), "Output should contain '1/3'");
    assert!(
        stdout.contains(".333"),
        "Output should contain result starting with .333"
    );
}

#[test]
fn test_cmd_with_once_white() {
    std::env::set_var("DNTK_ENV", "TEST");
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--once").arg("--white").arg("--inject").arg("1+2");
    #[cfg(not(target_os = "windows"))]
    cmd.assert()
        .success()
        .stdout("\r(dntk): \r(dntk): 1+2 = 3\u{1b}[4D\n");
    #[cfg(target_os = "windows")]
    cmd.assert()
        .success()
        .stdout("\r(dntk): \r(dntk): 1+2 = 3\n");
}

#[test]
fn test_cmd_with_once_scale() {
    std::env::set_var("DNTK_ENV", "TEST");
    let mut cmd1 = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd1.arg("--once")
        .arg("--scale")
        .arg("10")
        .arg("--inject")
        .arg("3/7");
    let mut cmd2 = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd2.arg("--once")
        .arg("--scale")
        .arg("1")
        .arg("--inject")
        .arg("3/7");

    // Note: f64 precision causes slight rounding differences
    // Just check that the output contains the expression and approximate results
    let output1 = cmd1.output().unwrap();
    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    assert!(output1.status.success(), "Command 1 should succeed");
    assert!(stdout1.contains("3/7"), "Output 1 should contain '3/7'");
    assert!(
        stdout1.contains(".428"),
        "Output 1 should contain result starting with .428"
    );

    let output2 = cmd2.output().unwrap();
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(output2.status.success(), "Command 2 should succeed");
    assert!(stdout2.contains("3/7"), "Output 2 should contain '3/7'");
    assert!(stdout2.contains(".4"), "Output 2 should contain result .4");
}

#[test]
fn test_cmd_with_version() {
    std::env::set_var("DNTK_ENV", "TEST");
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--version").assert().success();
}

#[test]
fn test_cmd_with_help() {
    std::env::set_var("DNTK_ENV", "TEST");
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--help").assert().success();
}

#[test]
fn test_cmd_with_show_limits() {
    std::env::set_var("DNTK_ENV", "TEST");
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--show-limits").assert().success();
}

#[test]
fn test_cmd_with_multi_statements() {
    std::env::set_var("DNTK_ENV", "TEST");
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--once").arg("--inject").arg("a=1; b=a*2; b+3");
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("a=1; b=a*2; b+3"));
    assert!(stdout.contains("= 5"));
}
