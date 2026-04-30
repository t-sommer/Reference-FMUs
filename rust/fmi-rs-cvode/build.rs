use std::env;

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir).parent().unwrap();
    let cvode_lib = workspace_root.join("cvode-7.7.0/install/lib");

    println!("cargo:rustc-link-search=native={}", cvode_lib.display());
    println!("cargo:rustc-link-lib=static=sundials_core_static");
    println!("cargo:rustc-link-lib=static=sundials_cvode_static");
    println!("cargo:rustc-link-lib=static=sundials_nvecserial_static");
    println!("cargo:rustc-link-lib=static=sundials_sunlinsoldense_static");
    println!("cargo:rustc-link-lib=static=sundials_sunmatrixdense_static");
}
