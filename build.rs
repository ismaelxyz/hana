//use std::env;
use std::process::Command;
use std::fs::File;
use std::io::{self, Write};

// mod platform { }
#[cfg(target_os = "linux")]
const OS: &str = "Linux";

#[cfg(target_os = "macos")]
const OS: &str = "MacOs"; 
//mod platform { }

#[cfg(target_os = "windows")]
const OS: &str = "Windows";
//mod platform { }

fn main() {
    //let is_release = env::var("PROFILE").unwrap() == "release";
    //println!("cargo:rerun-if-changed=./src/consts.rs");

    // Compile Parser
    //peg::cargo_build("src/parser.peg");
    let output = Command::new("rustc")
        .arg("-Vv")
        .output()
        .expect("Rustc command failed to start");
    
    // println!("status: {}", output.status);
    //io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
    assert!(output.status.success());

    let source = String::from_utf8(output.stdout).unwrap();
    let rustc = &source.split('\n').collect::<Vec<&str>>();
    let release = &rustc[rustc.len()-3][9..];
    //let rustc = &rustc[0]; // [..rustc[0].len()-2]

    let ver = format!(
"pub const VERSION: &str = \"Haru {}\\n[Rustc {}] on {}\";
pub const RUSTC: &str = \"R{}\";", env!("CARGO_PKG_VERSION"), release, OS, &rustc[0][1..]);
    let mut file = File::create("src/consts.rs").unwrap();
    file.write_all(ver.as_bytes()).unwrap();
}