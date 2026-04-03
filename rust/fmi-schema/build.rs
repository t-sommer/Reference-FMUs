fn main() {
    // static_vcruntime::metabuild();

    // Link to the static model_description_validator library
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let library_dir = manifest_dir.join("vendor/x86_64-windows");

    println!("cargo:rustc-link-search=native={}", library_dir.display());
    println!("cargo:rustc-link-lib=static=model_description_validator");

    // Link to libxml2 static library - use WHOLE ARCHIVE to ensure all symbols are included
    println!(
        "cargo:rustc-link-search=native={}",
        library_dir.join("lib").display()
    );
    println!(
        "cargo:rustc-link-arg=/WHOLEARCHIVE:{}",
        library_dir.join("lib/libxml2s.lib").display()
    );

    // Windows system libraries required by libxml2
    println!("cargo:rustc-link-lib=dylib=ws2_32");
    println!("cargo:rustc-link-lib=dylib=bcrypt");
    println!("cargo:rustc-link-lib=dylib=winmm");
}
