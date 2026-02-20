use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::CompressionMethod;
use std::fs;
use std::process::Command;
use fmi::SHARED_LIBRARY_EXTENSION;


fn zip_dir<P: AsRef<Path>>(src_dir: P, dst_file: P) -> Result<(), Box<dyn std::error::Error>> {
    let src_dir = src_dir.as_ref();
    let dst_file = dst_file.as_ref();

    let file = File::create(dst_file)?;
    let mut zip = zip::ZipWriter::new(file);

    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    for entry in WalkDir::new(src_dir) {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(src_dir)?.to_string_lossy().replace("\\", "/");

        if path.is_file() {
            zip.start_file(name.as_str(), options)?;
            let mut f = File::open(path)?;
            io::copy(&mut f, &mut zip)?;
        } else if path.is_dir() {
            // Add directories explicitly (optional but keeps empty dirs)
            if !name.is_empty() {
                let dir_name = format!("{}/", name);
                zip.add_directory(dir_name, options)?;
            }
        }
    }

    zip.finish()?;
    Ok(())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args: Vec<String> = std::env::args().collect();
    let mut fmi_version = "3".to_string();

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--fmi-version" {
            if i + 1 < args.len() {
                fmi_version = args[i + 1].clone();
                i += 1;
            }
        }
        i += 1;
    }

    let (deploy_dir, platform, features) = match fmi_version.as_str() {
        "2" => ("fmi2", fmi::fmi2::PLATFORM, vec!["--features", "fmi2"]),
        "3" => ("fmi3", fmi::fmi3::PLATFORM_TUPLE, vec![]),
        _ => return Err(format!("Unsupported FMI version: {}", fmi_version).into()),
    };

    let dist_dir = PathBuf::from("dist").join(deploy_dir);

    fs::create_dir_all(&dist_dir)?;

    let model_names = ["BouncingBall", "Dahlquist"];

    for model_name in model_names {
        
        let mut command = Command::new("cargo");
        command.args(["build", "--package", model_name]);
        
        if !features.is_empty() {
            command.args(&features);
        }

        let status = command.status()?;

        if !status.success() {
            return Err(format!("Failed to build {}", model_name).into());
        }

        let binary_dir = PathBuf::from(model_name).join(deploy_dir).join("binaries").join(platform);
        
        fs::create_dir_all(&binary_dir)?;
        
        dbg!(&binary_dir);

        let shared_library_name = format!("{}{}", model_name, SHARED_LIBRARY_EXTENSION);
        
        let dll_src = format!("target/debug/{shared_library_name}");
        let dll_dst = binary_dir.join(shared_library_name);
        
        dbg!(&dll_src);
        dbg!(&dll_dst);
        
        fs::copy(
            dll_src,
            dll_dst,
        )?;

        let src_dir = PathBuf::from(model_name).join(deploy_dir);
        let dst_file = dist_dir.join(format!("{}.fmu", model_name));

        dbg!(&src_dir);
        dbg!(&dst_file);

        zip_dir(src_dir, dst_file)?;
    }

    Ok(())
}