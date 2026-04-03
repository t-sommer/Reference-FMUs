fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir).parent().unwrap();
    let md_valid_lib = workspace_root.join("md-valid/install");
    let libxml2_lib = workspace_root.join("libxml2/install/lib");

    println!("cargo:rustc-link-search=native={}", md_valid_lib.display());
    println!("cargo:rustc-link-search=native={}", libxml2_lib.display());

    println!("cargo:rustc-link-lib=static=model_description_validator");

    // Use /WHOLEARCHIVE to force all symbols from libxml2 to be included
    println!("cargo:rustc-link-arg=/WHOLEARCHIVE:libxml2s.lib");

    // Link system libraries required by libxml2
    println!("cargo:rustc-link-lib=dylib=ws2_32");
    println!("cargo:rustc-link-lib=dylib=bcrypt");
    println!("cargo:rustc-link-lib=dylib=winmm");
}
