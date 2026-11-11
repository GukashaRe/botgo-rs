use std::env;
use std::process::Command;

fn main() {
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    let version = Command::new(rustc)
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|output| output.split_whitespace().nth(1).map(str::to_owned))
        .unwrap_or_else(|| {
            env::var("CARGO_PKG_RUST_VERSION").unwrap_or_else(|_| "unknown".to_owned())
        });

    println!("cargo:rustc-env=BOTGO_RUST_VERSION={version}");
    println!("cargo:rerun-if-changed=build.rs");
}
