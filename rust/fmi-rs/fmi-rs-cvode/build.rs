use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target = env::var("TARGET").unwrap();

    // Organize vendor by target triple to prevent cross-compilation conflicts
    let library_dir = manifest_dir.join("vendor").join(&target);

    let lib_ext = if target.contains("windows") {
        "lib"
    } else {
        "a"
    };
    let lib_name = format!("sundials_core_static.{}", lib_ext);

    // Check if sundials_core_static exists; if not, download and build it.
    let lib_path = library_dir.join("lib").join(lib_name);

    if !lib_path.exists() {
        fetch_and_build_cvode(&library_dir, &target);
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
fn fetch_and_build_cvode(install_dir: &Path, target: &str) {
    let version = "7.7.0";
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let out_path = Path::new(&out_dir);

    println!(
        "cargo:warning=Sundials not found. Downloading and building CVODE v{version} for {target}..."
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
    let url =
        format!("https://github.com/llnl/sundials/releases/download/v7.7.0/cvode-{version}.tar.gz");
    let tar_path = out_path.join("cvode.tar.gz");
    let status = Command::new("curl")
        .args(["-L", "-o", tar_path.to_str().unwrap(), &url])
        .output()
        .expect("Failed to execute curl. Ensure it is installed and in your PATH.");

    if !status.status.success() {
        panic!(
            "Failed to download cvode from {}. Error: {}",
            url,
            String::from_utf8_lossy(&status.stderr)
        );
    }

    // 2. Extract source using cmake -E tar
    let status = Command::new("cmake")
        .args(["-E", "tar", "xzf", tar_path.to_str().unwrap()])
        .current_dir(out_path)
        .output()
        .expect("Failed to execute cmake -E tar.");
    if !status.status.success() {
        panic!(
            "Failed to extract cvode source: {}",
            String::from_utf8_lossy(&status.stderr)
        );
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
            "-G",
            generator,
        ])
        .output()
        .expect("Failed to execute cmake. Ensure it is installed and in your PATH.");

    if !status.status.success() {
        panic!(
            "Failed to configure cvode with generator {}: {}",
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
        .expect("Failed to build cvode.");

    if !status.status.success() {
        panic!(
            "Failed to install cvode: {}",
            String::from_utf8_lossy(&status.stderr)
        );
    }
}
