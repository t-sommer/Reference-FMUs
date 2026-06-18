use std::{env, path::Path};

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target = env::var("TARGET").unwrap();

    // Organize vendor by target triple to prevent cross-compilation conflicts
    let library_dir = manifest_dir.join("vendor").join(&target);

    // Check if libxml2 static library exists; if not, download and build it.
    if !library_dir.exists() {
        fetch_and_build_libxml2(&library_dir, &target);
    }

    cc::Build::new()
        .file("src/c/src/fmi_rs_xsd.c")
        .include("src/c/include")
        .include(format!("vendor/{target}/include/libxml2"))
        .compile("fmi_rs_xsd");

    // Link to libxml2 static library
    println!(
        "cargo:rustc-link-search=native={}",
        library_dir.join("lib").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        library_dir.join("lib64").display()
    );

    if target.contains("windows") {
        println!("cargo:rustc-link-lib=static=libxml2s");
        // Windows system libraries required by libxml2
        println!("cargo:rustc-link-lib=dylib=ws2_32");
        println!("cargo:rustc-link-lib=dylib=bcrypt");
        println!("cargo:rustc-link-lib=dylib=winmm");
    } else {
        println!("cargo:rustc-link-lib=static=xml2");
    }
}

/// Downloads, builds, and installs libxml2 to the specified directory.
fn fetch_and_build_libxml2(install_dir: &std::path::Path, target: &str) {
    use std::process::Command;

    let version = "2.15.3";
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let out_path = std::path::Path::new(&out_dir);

    println!(
        "cargo:warning=libxml2 not found. Downloading and building libxml2 v{}...",
        version
    );

    // Determine CMake generator based on the Rust target
    let generator = if target.contains("msvc") {
        "Visual Studio 17 2022"
    } else if target.contains("windows-gnu") {
        "MinGW Makefiles"
    } else {
        "Unix Makefiles"
    };

    // 1. Download source using curl (standard on modern Windows)
    let url = format!(
        "https://github.com/GNOME/libxml2/archive/refs/tags/v{}.tar.gz",
        version
    );
    let tar_path = out_path.join("libxml2.tar.gz");
    let status = Command::new("curl")
        .args(["-L", "-o", tar_path.to_str().unwrap(), &url])
        .status()
        .expect("Failed to execute curl. Ensure it is installed and in your PATH.");
    if !status.success() {
        panic!("Failed to download libxml2 from {}", url);
    }

    // 2. Extract source using cmake -E tar
    let status = Command::new("cmake")
        .args(["-E", "tar", "xzf", tar_path.to_str().unwrap()])
        .current_dir(out_path)
        .status()
        .expect("Failed to execute cmake -E tar.");
    if !status.success() {
        panic!("Failed to extract libxml2 source.");
    }

    let src_dir = out_path.join(format!("libxml2-{}", version));
    let build_dir = out_path.join("libxml2-build");

    // 3. Configure with CMake
    let status = Command::new("cmake")
        .args([
            "-S",
            src_dir.to_str().unwrap(),
            "-B",
            build_dir.to_str().unwrap(),
            &format!("-DCMAKE_INSTALL_PREFIX={}", install_dir.display()),
            "-DBUILD_SHARED_LIBS=OFF",
            "-DLIBXML2_WITH_PYTHON=OFF",
            "-DLIBXML2_WITH_ZLIB=OFF",
            "-DLIBXML2_WITH_LZMA=OFF",
            "-DLIBXML2_WITH_ICONV=OFF",
            "-G",
            generator,
        ])
        .output()
        .expect("Failed to execute cmake. Ensure it is installed and in your PATH.");

    if !status.status.success() {
        panic!(
            "Failed to configure libxml2 with generator {}: {}",
            generator,
            String::from_utf8_lossy(&status.stderr)
        );
    }

    // 4. Build and Install
    let status = Command::new("cmake")
        .args([
            "--build",
            build_dir.to_str().unwrap(),
            "--config",
            "Release",
            "--target",
            "install",
        ])
        .output()
        .expect("Failed to build libxml2.");

    if !status.status.success() {
        panic!(
            "Failed to install libxml2: {}",
            String::from_utf8_lossy(&status.stderr)
        );
    }
}
