use std::env;

fn main() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let library_dir = manifest_dir.join("vendor/x86_64-windows");

    // Check if sundials_core_static exists; if not, download and build it.
    let lib_path = library_dir.join("lib/sundials_core_static.lib");
    if !lib_path.exists() {
        fetch_and_build_cvode(&library_dir);
    }

    println!(
        "cargo:rustc-link-search=native={}",
        library_dir.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=sundials_core_static");
    println!("cargo:rustc-link-lib=static=sundials_cvode_static");
    println!("cargo:rustc-link-lib=static=sundials_nvecserial_static");
    println!("cargo:rustc-link-lib=static=sundials_sunlinsoldense_static");
    println!("cargo:rustc-link-lib=static=sundials_sunmatrixdense_static");
}

/// Downloads, builds, and installs cvode to the specified directory.
fn fetch_and_build_cvode(install_dir: &std::path::Path) {
    use std::process::Command;

    let version = "7.7.0";
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let out_path = std::path::Path::new(&out_dir);

    println!(
        "cargo:warning=sundials_core_static.lib not found. Downloading and building cvode v{version}..."
    );

    // 1. Download source using curl (standard on modern Windows)
    let url =
        format!("https://github.com/llnl/sundials/releases/download/v7.7.0/cvode-{version}.tar.gz");
    let tar_path = out_path.join("cvode.tar.gz");
    let status = Command::new("curl")
        .args(["-L", "-o", tar_path.to_str().unwrap(), &url])
        .status()
        .expect("Failed to execute curl. Ensure it is installed and in your PATH.");
    if !status.success() {
        panic!("Failed to download cvode from {}", url);
    }

    // 2. Extract source using cmake -E tar
    let status = Command::new("cmake")
        .args(["-E", "tar", "xzf", tar_path.to_str().unwrap()])
        .current_dir(out_path)
        .status()
        .expect("Failed to execute cmake -E tar.");
    if !status.success() {
        panic!("Failed to extract cvode source.");
    }

    let src_dir = out_path.join(format!("cvode-{version}"));
    let build_dir = out_path.join("cvode-build");

    // 3. Configure with CMake
    let status = Command::new("cmake")
        .args([
            "-S",
            src_dir.to_str().unwrap(),
            "-B",
            build_dir.to_str().unwrap(),
            &format!("-DCMAKE_INSTALL_PREFIX={}", install_dir.display()),
            "-DBUILD_SHARED_LIBS=OFF",
        ])
        .status()
        .expect("Failed to execute cmake. Ensure it is installed and in your PATH.");
    if !status.success() {
        panic!("Failed to configure cvode.");
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
        .status()
        .expect("Failed to build cvode.");
    if !status.success() {
        panic!("Failed to install cvode.");
    }
}
