use std::env;

fn main() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let library_dir = manifest_dir.join("vendor/x86_64-windows");

    println!(
        "cargo:rustc-link-search=native={}",
        library_dir.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=sundials_core_static");
    println!("cargo:rustc-link-lib=static=sundials_cvode_static");
    println!("cargo:rustc-link-lib=static=sundials_nvecserial_static");
    println!("cargo:rustc-link-lib=static=sundials_sunlinsoldense_static");
    println!("cargo:rustc-link-lib=static=sundials_sunmatrixdense_static");
}
