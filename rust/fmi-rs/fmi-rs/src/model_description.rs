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
