fn main() {
    // Link to the static libxml2 library you built
    println!("cargo:rustc-link-search=native=C:\\Users\\tsr2\\Downloads\\libxml2-v2.15.2\\install\\lib");
    println!("cargo:rustc-link-lib=static=libxml2s");
    
    // Windows system libraries
    println!("cargo:rustc-link-lib=dylib=winmm");
    println!("cargo:rustc-link-lib=dylib=ws2_32");
    println!("cargo:rustc-link-lib=dylib=bcrypt");
}
