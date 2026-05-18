fn main() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let library_dir = manifest_dir.join("vendor/x86_64-windows");

    cc::Build::new()
        .file("src/c/src/fmi_rs_xsd.c")
        .include("src/c/include")
        .include("vendor/x86_64-windows/include/libxml2")
        .compile("fmi_rs_xsd");

    // Link to libxml2 static library - use WHOLE ARCHIVE to ensure all symbols are included
    println!(
        "cargo:rustc-link-search=native={}",
        library_dir.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=libxml2s");

    // Windows system libraries required by libxml2
    println!("cargo:rustc-link-lib=dylib=ws2_32");
    println!("cargo:rustc-link-lib=dylib=bcrypt");
    println!("cargo:rustc-link-lib=dylib=winmm");
}
