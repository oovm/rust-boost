use std::{env, process::Command};

fn main() {
    if is_nightly_toolchain() {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }
}

fn is_nightly_toolchain() -> bool {
    if let Ok(toolchain) = env::var("RUSTUP_TOOLCHAIN") {
        if toolchain.starts_with("nightly") {
            return true;
        }
        if toolchain.starts_with("stable") {
            return false;
        }
    }

    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let output = Command::new(rustc).arg("--version").output().expect("failed to run `rustc --version`");
    String::from_utf8_lossy(&output.stdout).contains("nightly")
}
