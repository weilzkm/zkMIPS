use std::path::Path;
use std::process::Command;

fn main() {
    let go_src = Path::new("../go-ethereum");
    let status = Command::new("go")
        .arg("build")
        .arg("-tags")
        .arg("ziren")
        .arg("./cmd/keeper")
        .current_dir(go_src)
        .env("GOOS", "linux")
        .env("GOARCH", "mipsle")
        .env("GOMIPS", "softfloat")
        .status()
        .expect("failed to build simple go guest");

    if !status.success() {
        panic!("go build failed");
    }

    // 5. 告诉 Cargo：只要 go/ 目录有变动就重新跑 build.rs
    println!("cargo:rerun-if-changed=../go-ethereum");
}
