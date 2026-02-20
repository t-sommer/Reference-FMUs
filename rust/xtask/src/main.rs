use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::CompressionMethod;
use std::fs;
use fmi::fmi3::PLATFORM_TUPLE;
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

    let model_names = ["BouncingBall", "Dahlquist"];

    let deploy_dir = "fmi3";

    for model_name in model_names {
        
        let binary_dir = PathBuf::from(model_name).join(deploy_dir).join("binaries").join(PLATFORM_TUPLE);
        
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
        let dst_file = PathBuf::from(format!("{}.fmu", model_name));

        dbg!(&src_dir);
        dbg!(&dst_file);

        zip_dir(src_dir, dst_file)?;
    }

    Ok(())
}