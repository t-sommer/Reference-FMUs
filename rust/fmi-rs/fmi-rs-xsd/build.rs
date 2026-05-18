fn main() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let library_dir = manifest_dir.join("vendor/x86_64-windows");

    // Check if libxml2 static library exists; if not, download and build it.
    let lib_path = library_dir.join("lib/libxml2s.lib");
    if !lib_path.exists() {
        fetch_and_build_libxml2(&library_dir);
    }

    cc::Build::new()
        .file("src/c/src/fmi_rs_xsd.c")
        .include("src/c/include")
        .include("vendor/x86_64-windows/include/libxml2")
        .compile("fmi_rs_xsd");

    // Link to libxml2 static library
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

/// Downloads, builds, and installs libxml2 to the specified directory.
fn fetch_and_build_libxml2(install_dir: &std::path::Path) {
    use std::process::Command;

    let version = "2.15.3"; 
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let out_path = std::path::Path::new(&out_dir);

    println!("cargo:warning=libxml2s.lib not found. Downloading and building libxml2 v{}...", version);

    // 1. Download source using curl (standard on modern Windows)
    let url = format!("https://github.com/GNOME/libxml2/archive/refs/tags/v{}.tar.gz", version);
    let tar_path = out_path.join("libxml2.tar.gz");
    let status = Command::new("curl")
        .args(["-L", "-o", tar_path.to_str().unwrap(), &url])
        .status()
        .expect("Failed to execute curl. Ensure it is installed and in your PATH.");
    if !status.success() { panic!("Failed to download libxml2 from {}", url); }

    // 2. Extract source using cmake -E tar
    let status = Command::new("cmake")
        .args(["-E", "tar", "xzf", tar_path.to_str().unwrap()])
        .current_dir(out_path)
        .status()
        .expect("Failed to execute cmake -E tar.");
    if !status.success() { panic!("Failed to extract libxml2 source."); }

    let src_dir = out_path.join(format!("libxml2-{}", version));
    let build_dir = out_path.join("libxml2-build");

    // 3. Configure with CMake
    let status = Command::new("cmake")
        .args([
            "-S", src_dir.to_str().unwrap(),
            "-B", build_dir.to_str().unwrap(),
            &format!("-DCMAKE_INSTALL_PREFIX={}", install_dir.display()),
            "-DBUILD_SHARED_LIBS=OFF",
            "-DLIBXML2_WITH_PYTHON=OFF",
            "-DLIBXML2_WITH_ZLIB=OFF",
            "-DLIBXML2_WITH_LZMA=OFF",
            "-DLIBXML2_WITH_ICONV=OFF",
        ])
        .status()
        .expect("Failed to execute cmake. Ensure it is installed and in your PATH.");
    if !status.success() { panic!("Failed to configure libxml2."); }

    // 4. Build and Install
    let status = Command::new("cmake")
        .args(["--build", build_dir.to_str().unwrap(), "--config", "Release", "--target", "install"])
        .status()
        .expect("Failed to build libxml2.");
    if !status.success() { panic!("Failed to install libxml2."); }

    // 5. Ensure the static library is named libxml2s.lib (the expected name for static libxml2 on MSVC)
    let lib_file = install_dir.join("lib/libxml2.lib");
    let target_lib = install_dir.join("lib/libxml2s.lib");
    if lib_file.exists() && !target_lib.exists() {
        let _ = std::fs::rename(lib_file, target_lib);
    }
}
