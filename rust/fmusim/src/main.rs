#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use std::{fs::{File, read, read_to_string}, io::Write, path::Path};
use fmi::{SHARED_LIBRARY_EXTENSION, fmi3::{FMU3, PLATFORM_TUPLE}, types::fmiValueReference};
use clap::Parser;
use zip::ZipArchive;
use tempfile::TempDir;

fn extract_fmu(fmu_path: &str) -> Result<TempDir, Box<dyn std::error::Error>> {

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

#[derive(Parser)]
#[command(name = "fmusim")]
#[command(about = "FMU simulation tool")]
struct Args {
    /// Path to the FMU file or directory containing modelDescription.xml
    filename: String,
    
    /// Enable logging of FMI function calls
    #[arg(long)]
    log_fmi_calls: bool,
}

#[derive(Debug)]
enum VariableType {
    Float64,
}   

#[derive(Debug, PartialEq)]
enum Causality {
    Parameter,
    CalculatedParameter,
    StructuralParameter,
    Input,
    Output,
    Local,
    Independent
}

#[derive(Debug)]
struct CoSimulation {
    modelIdentifier: String,
}

#[derive(Debug)]
struct ModelVariable {
    variableType: VariableType,
    name: String,
    valueReference: fmiValueReference,
    causality: Causality,
}

#[derive(Debug)]
struct ModelDescription {
    fmiVersion: String,
    modelName: String,
    instantiationToken: String,
    coSimulation: Option<CoSimulation>,
    modelVariables: Vec<ModelVariable>,
}

fn read_model_description(path: &Path) -> Result<ModelDescription, String> {


    let text = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => return Err(format!("ERROR: Failed to read XML file: {}", e))
    };

    let opt = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..roxmltree::ParsingOptions::default()
    };

    let doc = roxmltree::Document::parse_with_options(&text, opt).unwrap();

    let root = doc.root_element();
    
    let ModelVariables = root.descendants().find(|n| n.has_tag_name("ModelVariables")).unwrap();
    
    let mut modelVariables = vec![];

    for child in ModelVariables.children().filter(|n| n.is_element()) {

        let name = child.attribute("name").unwrap();
        let valueReference = child.attribute("valueReference").unwrap().parse().unwrap();

        let causality = match child.attribute("causality") {
            Some("parameter") => Causality::Parameter,
            Some("calculatedParameter") => Causality::CalculatedParameter,
            Some("structuralParameter") => Causality::StructuralParameter,
            Some("input") => Causality::Input,
            Some("output") => Causality::Output,
            Some("independent") => Causality::Independent,
            _ => Causality::Local,
        };

        match child.tag_name().name() {
            "Float64" => {
                // println!("Float64!"),
                modelVariables.push(ModelVariable {
                    variableType: VariableType::Float64,
                    name: name.to_string(),
                    valueReference: valueReference,
                    causality: causality,
                });
                    
            }
            _ => {},
        }
        // println!("{child:?}");
    }

    let coSimulation = if let Some(cs) = root.descendants().find(|n| n.has_tag_name("CoSimulation")) {
        Some(
            CoSimulation {
                modelIdentifier: cs.attribute("modelIdentifier").unwrap().to_string()
            }
        )
    } else {
        None
    };

    let model_description = ModelDescription {
        fmiVersion: root.attribute("fmiVersion").unwrap().to_string(),
        modelName: root.attribute("modelName").unwrap().to_string(),
        instantiationToken: root.attribute("instantiationToken").unwrap().to_string(),
        coSimulation: coSimulation,
        modelVariables: modelVariables,
    };

    Ok(model_description)
}

fn main() {

    let args = Args::parse();

    // Extract FMU to temporary directory
    let temp_dir = match extract_fmu(&args.filename) {
        Ok(dir) => dir,
        Err(e) => {
            println!("ERROR: Failed to extract FMU: {}", e);
            return;
        }
    };

    // Path to modelDescription.xml in the extracted directory
    let xml_path = temp_dir.path().join("modelDescription.xml");

    let model_description = read_model_description(xml_path.as_path()).unwrap();
    
    // println!("{model_description:#?}");

    // Create logging callbacks only if requested
    let log_fmi_call = if args.log_fmi_calls {
        Some(Box::new(|status: &fmi::types::fmiStatus, message: &str| {
            println!("{message} -> {status:?}");
        }) as Box<dyn Fn(&fmi::types::fmiStatus, &str) + Send + Sync>)
    } else {
        None
    };

    let log_message = if args.log_fmi_calls {
        Some(Box::new(|status: &fmi::types::fmiStatus, category: &str, message: &str| {
            println!("[Message][{:?}][{}] {}", status, category, message);
        }) as Box<dyn Fn(&fmi::types::fmiStatus, &str, &str) + Send + Sync>)
    } else {
        None
    };

    let unzipdir = temp_dir.path();

    let shared_library_filename = format!("{}{}", model_description.coSimulation.unwrap().modelIdentifier, SHARED_LIBRARY_EXTENSION);

    let shared_library_path = unzipdir.join("binaries").join(PLATFORM_TUPLE).join(shared_library_filename);

    let mut fmu = FMU3::new(
        shared_library_path.as_path(), 
        "instance1", 
        log_fmi_call, 
        log_message
    ).expect("Failed to load FMU");

    fmu.instantiateCoSimulation(
        &model_description.modelName,
        &model_description.instantiationToken,
        None, 
        false, 
        false, 
        false, 
        false, 
        &[]
    );

    fmu.enterInitializationMode(None, 0.0, Some(1.0));

    fmu.exitInitializationMode();

    let value_references: Vec<u32> = model_description.modelVariables
        .iter()
        .filter(|v| v.causality == Causality::Output)
        .map(|v| v.valueReference)
        .collect();

    let mut values = vec![0.0; value_references.len()];

    let mut eventHandlingNeeded: bool = false;
    let mut terminateSimulation: bool = false;
    let mut earlyReturn: bool = false;
    let mut lastSuccessfulTime: fmi::types::fmiFloat64 = 0.0;

    fmu.doStep(0.0, 0.1, true, &mut eventHandlingNeeded, &mut terminateSimulation, &mut earlyReturn, &mut lastSuccessfulTime); 

    fmu.getFloat64(&value_references, &mut values);

    fmu.terminate();

    fmu.freeInstance();
}
