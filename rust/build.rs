use std::{env, path::PathBuf};

fn main() {
    for key in ["LIBBINDER_NDK", "OPLUS_PLATFORM_LIB_DIR"] {
        println!("cargo:rerun-if-env-changed={key}");
    }
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("android") {
        return;
    }
    let library = env::var_os("LIBBINDER_NDK").map(PathBuf::from)
        .or_else(|| env::var_os("OPLUS_PLATFORM_LIB_DIR").map(|p| PathBuf::from(p).join("libbinder_ndk.so")))
        .expect("Android builds require LIBBINDER_NDK or OPLUS_PLATFORM_LIB_DIR pointing to the real platform library");
    assert!(library.is_file(), "platform libbinder_ndk.so is missing");
    println!(
        "cargo:rustc-link-search=native={}",
        library.parent().unwrap().display()
    );
    // Explicit input prevents accidentally selecting the NDK's public stub.
    println!(
        "cargo:rustc-link-arg-bin=oplus_project_service={}",
        library.display()
    );
    println!("cargo:rustc-link-lib=dylib=log");
}
