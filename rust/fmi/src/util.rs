use std::fs::File;
use tempfile::TempDir;
use zip::ZipArchive;

use crate::types::*;

#[derive(Debug, PartialEq)]
pub enum VariableValue {
    Float32(Vec<fmiFloat32>),
    Float64(Vec<fmiFloat64>),
    Int8(Vec<fmiInt8>),
    UInt8(Vec<fmiUInt8>),
    Int16(Vec<fmiInt16>),
    UInt16(Vec<fmiUInt16>),
    Int32(Vec<fmiInt32>),
    UInt32(Vec<fmiUInt32>),
    Int64(Vec<fmiInt64>),
    UInt64(Vec<fmiUInt64>),
    Boolean(Vec<fmiBoolean>),
    String(Vec<String>),
    Binary(Vec<Vec<fmiByte>>),
    // Clock(fmiClock),
}

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
