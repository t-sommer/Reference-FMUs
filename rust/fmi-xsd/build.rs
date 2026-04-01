fn main() {
    // static_vcruntime::metabuild();

    // Link to the static model_description_validator library  
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir).parent().unwrap();
    let md_valid_lib = workspace_root.join("md-valid/install");
    
    println!("cargo:rustc-link-search=native={}", md_valid_lib.display());
    println!("cargo:rustc-link-lib=static=model_description_validator");
    
    // Link to libxml2 static library - use WHOLE ARCHIVE to ensure all symbols are included
    let libxml2_lib = workspace_root.join("libxml2/install/lib");
    let libxml2_archive = libxml2_lib.join("libxml2s.lib");
    
    println!("cargo:rustc-link-search=native={}", libxml2_lib.display());
    println!("cargo:rustc-link-arg=/WHOLEARCHIVE:{}", libxml2_archive.display());
    
    // Windows system libraries required by libxml2
    println!("cargo:rustc-link-lib=dylib=ws2_32");
    println!("cargo:rustc-link-lib=dylib=bcrypt");
    println!("cargo:rustc-link-lib=dylib=winmm");
}
