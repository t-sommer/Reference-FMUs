fn main() {
    static_vcruntime::metabuild();

    // Link to the static model_description_validator library  
    println!("cargo:rustc-link-search=native=E:/WS/Reference-FMUs/rust/md-valid/build/Release");
    println!("cargo:rustc-link-lib=static=model_description_validator");

    // Link to libxml2 static library - use WHOLE ARCHIVE to ensure all symbols are included
    println!("cargo:rustc-link-search=native=E:/WS/Reference-FMUs/rust/libxml2/install/lib");
    println!("cargo:rustc-link-arg=/WHOLEARCHIVE:E:/WS/Reference-FMUs/rust/libxml2/install/lib/libxml2s.lib");
    
    // Windows system libraries required by libxml2
    println!("cargo:rustc-link-lib=dylib=ws2_32");
    println!("cargo:rustc-link-lib=dylib=bcrypt");
    println!("cargo:rustc-link-lib=dylib=winmm");
}
