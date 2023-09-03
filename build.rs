use std::env;
use std::fs::create_dir_all;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rustc-env=TARGET={}", env::var("TARGET").unwrap());
    println!("cargo:rerun-if-env-changed=TYPSTFMT_VERSION");
    println!("cargo:rerun-if-env-changed=GEN_ARTIFACTS");

    if option_env!("TYPSTFMT_VERSION").is_none() {
        println!("cargo:rustc-env=TYPSTFMT_VERSION={}", typst_version());
    }

    if let Some(dir) = env::var_os("GEN_ARTIFACTS") {
        let out = &Path::new(&dir);
        create_dir_all(out).unwrap();
    }
}

fn typst_version() -> String {
    let pkg = env!("CARGO_PKG_VERSION");
    let hash = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout.get(..8)?.into()).ok())
        .unwrap_or_else(|| "unknown hash".into());

    format!("{pkg} ({hash})")
}
