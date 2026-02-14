use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor_dir = manifest_dir.join("vendor").join("c-toxcore");

    // Build c-toxcore with cmake
    let dst = cmake::Config::new(&vendor_dir)
        .define("BOOTSTRAP_DAEMON", "OFF")
        .define("BUILD_TOXAV", "ON")
        .define("MUST_BUILD_TOXAV", "ON")
        .define("DHT_BOOTSTRAP", "OFF")
        .define("BUILD_FUN_UTILS", "OFF")
        .define("AUTOTEST", "OFF")
        .define("UNITTEST", "OFF")
        .build();

    // Link the built library
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib64").display()
    );
    // c-toxcore v0.2.22 builds everything into a single libtoxcore
    println!("cargo:rustc-link-lib=static=toxcore");

    // Link system dependencies
    pkg_config::probe_library("libsodium").expect("libsodium not found");
    pkg_config::probe_library("opus").expect("opus not found");
    pkg_config::probe_library("vpx").expect("vpx not found");

    // pthreads
    println!("cargo:rustc-link-lib=pthread");

    // Generate bindings
    let include_path = dst.join("include");
    let bindings = bindgen::Builder::default()
        .header(manifest_dir.join("wrapper.h").to_str().unwrap())
        .clang_arg(format!("-I{}", include_path.display()))
        .allowlist_function("tox_.*")
        .allowlist_function("toxav_.*")
        .allowlist_function("tox_pass_.*")
        .allowlist_type("Tox.*")
        .allowlist_type("TOX.*")
        .allowlist_var("TOX_.*")
        .derive_debug(true)
        .derive_default(true)
        .generate()
        .expect("Failed to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Failed to write bindings");
}
