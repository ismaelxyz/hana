use std::{
    env,
    fs::File,
    io::{self, Write},
    process::Command,
};

#[cfg(target_os = "linux")]
const OS: &str = "Linux";

#[cfg(target_os = "macos")]
const OS: &str = "MacOs";

#[cfg(target_os = "windows")]
const OS: &str = "Windows";

fn main() {

    // increase the size of the stack so that it is not too short
    println!("cargo:rustc-link-arg=/stack:{}", 10 * 1024 * 1024);
    
    if env::var("CARGO_RUN_BUILD").is_ok() {
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
            release, OS, &rustc[0]
        );
        let mut file = File::create("src/consts.rs").unwrap();
        file.write_all(ver.as_bytes()).unwrap();
    }
}
