fn main() {
    // Link to the static model_description_validator library  
    println!("cargo:rustc-link-search=native=E:/WS/Reference-FMUs/rust/md-valid/build/Debug");
    println!("cargo:rustc-link-lib=static=model_description_validator");

    // Link to libxml2 DLL (libxml2.dll must be in PATH or same directory as executable)
    println!("cargo:rustc-link-search=native=C:/Users/tsr2/Downloads/libxml2-v2.15.2/install/lib");
    println!("cargo:rustc-link-lib=dylib=libxml2");
    
    // Windows system libraries required by libxml2
    println!("cargo:rustc-link-lib=dylib=ws2_32");
    println!("cargo:rustc-link-lib=dylib=bcrypt");
    println!("cargo:rustc-link-lib=dylib=winmm");
}
