use std::env;
use std::path::Path;

fn main() {
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let platform = format!("{}-{}", arch, os);

    println!("cargo:rustc-env=ARCH={}", arch);
    println!("cargo:rustc-env=OS={}", os);
    println!("cargo:rustc-env=PLATFORM={}", platform);

    let platform_dir = format!("src/script/luas/{}", platform);

    if Path::new(&platform_dir).exists() {
        println!("cargo:rustc-env=SCRIPT_PLATFORM={}", platform);
    } else {
        println!("cargo:rustc-env=SCRIPT_PLATFORM=unknown");
    }

    println!("cargo:rerun-if-changed=src/script/luas");
}
