use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::CompressionMethod;
use std::fs;


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

    fs::create_dir_all("deploy/binaries/x86_64-windows")?;

    fs::copy(
        "target/debug/BouncingBall.dll",
        "deploy/binaries/x86_64-windows/BouncingBall.dll",
    )?;

    let src_dir = PathBuf::from("deploy");
    let dst_file = PathBuf::from("BouncingBall.fmu");

    zip_dir(src_dir, dst_file)?;

    Ok(())
}