extern crate assert_cmd;
use assert_cmd::prelude::*;

use std::process::Command;

#[test]
fn test_cmd_with_once() {
    std::env::set_var("ENV", "TEST");
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd
        .arg("--once")
        .arg("--inject")
        .arg("1/3");
    #[cfg(not(target_os = "windows"))]
    cmd
        .assert()
        .success()
        .stdout("\u{1b}[36m\r(dntk): 1/3 = .33333333333333333333\u{1b}[0m\u{1b}[24D\n");
    #[cfg(target_os = "windows")]
    cmd
        .assert()
        .success()
        .stdout("\u{1b}[36m\r(dntk): 1/3 = .33333333333333333333\u{1b}[0m\n");
}

#[test]
fn test_cmd_with_once_white() {
    std::env::set_var("ENV", "TEST");
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd
        .arg("--once")
        .arg("--white")
        .arg("--inject")
        .arg("1+2");
    #[cfg(not(target_os = "windows"))]
    cmd
        .assert()
        .success()
        .stdout("\r(dntk): 1+2 = 3\u{1b}[4D\n");
    #[cfg(target_os = "windows")]
    cmd
        .assert()
        .success()
        .stdout("\r(dntk): 1+2 = 3\n");
}

#[test]
fn test_cmd_with_once_scale() {
    std::env::set_var("ENV", "TEST");
    let mut cmd1 = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd1
        .arg("--once")
        .arg("--scale")
        .arg("10")
        .arg("--inject")
        .arg("3/7");
    let mut cmd2 = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd2
        .arg("--once")
        .arg("--scale")
        .arg("1")
        .arg("--inject")
        .arg("3/7");
    #[cfg(not(target_os = "windows"))]
    {
        cmd1
            .assert()
            .success()
            .stdout("\u{1b}[36m\r(dntk): 3/7 = .4285714285\u{1b}[0m\u{1b}[14D\n");
        cmd2
            .assert()
            .success()
            .stdout("\u{1b}[36m\r(dntk): 3/7 = .4\u{1b}[0m\u{1b}[5D\n");
    }
    #[cfg(target_os = "windows")]
    {
        cmd1
            .assert()
            .success()
            .stdout("\u{1b}[36m\r(dntk): 3/7 = .4285714285\u{1b}[0m\n");
        cmd2
            .assert()
            .success()
            .stdout("\u{1b}[36m\r(dntk): 3/7 = .4\u{1b}[0m\n");
    }
}

#[test]
fn test_cmd_with_version() {
    std::env::set_var("ENV", "TEST");
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd
        .arg("--version")
        .assert()
        .success();
}

#[test]
fn test_cmd_with_help() {
    std::env::set_var("ENV", "TEST");
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_cmd_with_show_limits() {
    std::env::set_var("ENV", "TEST");
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd
        .arg("--show-limits")
        .assert()
        .success();
}