use std::{fs::File, path::Path};
use tempfile::TempDir;
use zip::ZipArchive;

pub fn extract_fmu<P: AsRef<Path>>(fmu_path: P) -> Result<TempDir, Box<dyn std::error::Error>> {
    // Create temporary directory
    let temp_dir = TempDir::new()?;

    // Open the FMU file (which is a ZIP archive)
    let file = File::open(fmu_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Extract all files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => temp_dir.path().join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            // Directory
            std::fs::create_dir_all(&outpath)?;
        } else {
            // File
            if let Some(p) = outpath.parent()
                && !p.exists()
            {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(temp_dir)
}

/// Returns all entries of the ZIP archive
pub fn get_zip_contents(fmu_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Open the FMU file (which is a ZIP archive)
    let file = File::open(fmu_path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut entries = vec![];

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        if let Some(path) = file.enclosed_name() {
            entries.push(path.to_str().unwrap().to_string());
        }
    }

    Ok(entries)
}
