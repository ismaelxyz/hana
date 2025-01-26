use std::fs::File;
use std::io::{self, Write};
use std::process::Command;

#[cfg(target_os = "linux")]
const OS: &str = "Linux";

#[cfg(target_os = "macos")]
const OS: &str = "MacOs";

#[cfg(target_os = "windows")]
const OS: &str = "Windows";

fn main() {
    let output = Command::new("rustc")
        .arg("-Vv")
        .output()
        .expect("Rustc command failed to start");

    io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());

    let source = String::from_utf8(output.stdout).unwrap();
    let rustc = &source.split('\n').collect::<Vec<&str>>();
    let release = &rustc[rustc.len() - 3][9..];

    let ver = format!(
        "pub const VERSION: &str = \"Haru {}\\n({}) on {}\";",
        release,
        OS,
        &rustc[0]
    );
    let mut file = File::create("src/consts.rs").unwrap();
    file.write_all(ver.as_bytes()).unwrap();
}
