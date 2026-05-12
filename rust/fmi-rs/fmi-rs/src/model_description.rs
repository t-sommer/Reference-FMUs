pub mod fmi2;
pub mod fmi3;

use std::{error::Error, path::Path};

pub fn peak_fmi_version(path: &Path) -> Result<String, Box<dyn Error>> {
    let text = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => return Err(format!("Failed to read XML file: {}", e).into()),
    };

    let opt = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..roxmltree::ParsingOptions::default()
    };

    let doc = roxmltree::Document::parse_with_options(&text, opt).unwrap();

    let root = doc.root_element();

    if let Some(fmi_version) = root.attribute("fmiVersion") {
        return Ok(fmi_version.to_string());
    } else {
        Err("Attribute fmiVersion is missing.".into())
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum FMIMajorVersion {
    V2 = 2,
    V3 = 3,
}

pub fn peak_fmi_major_version(path: &Path) -> Result<FMIMajorVersion, Box<dyn Error>> {
    let fmi_version = peak_fmi_version(path)?;

    if fmi_version == "1.0" {
        Err("FMI 1.0 is not supported.".into())
    } else if fmi_version == "2.0" {
        Ok(FMIMajorVersion::V2)
    } else if fmi_version.starts_with("3.") {
        Ok(FMIMajorVersion::V3)
    } else {
        Err(format!("Unknown FMI version: {}", fmi_version).into())
    }
}
