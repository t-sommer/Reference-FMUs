use std::{error::Error, fmt::Display, fs::File, io::{self, Write}};
use zip::ZipArchive;
use tempfile::TempDir;

use crate::{fmi3::FMU3, model_description::{Dimension, ModelVariable, VariableType}};


pub fn extract_fmu(fmu_path: &str) -> Result<TempDir, Box<dyn std::error::Error>> {

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
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(temp_dir)
}
