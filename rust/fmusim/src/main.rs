#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use std::{path::Path};
use fmi::{SHARED_LIBRARY_EXTENSION, fmi3::{FMU3, PLATFORM_TUPLE}, types::fmiValueReference};
use clap::Parser;

#[derive(Parser)]
#[command(name = "fmusim")]
#[command(about = "FMU simulation tool")]
struct Args {
    /// Path to the FMU file or directory containing modelDescription.xml
    filename: String,
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

fn main() {

    let args = Args::parse();

    println!("Filename argument: {}", args.filename);

    let xml_path = &args.filename;
    println!("Attempting to read file: {}", xml_path);

    let text = match std::fs::read_to_string(xml_path) {
        Ok(content) => {
            println!("Successfully read XML file");
            content
        },
        Err(e) => {
            println!("ERROR: Failed to read XML file: {}", e);
            return;
        }
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
    
    println!("{model_description:#?}");

    // let doc = roxmltree::Document::parse("<rect id='rect1'/>").unwrap();

    // let elem = doc.descendants().find(|n| n.attribute("id") == Some("rect1")).unwrap();
    
    // assert!(elem.has_tag_name("rect"));

    let log_fmi_call = |status: &fmi::types::fmiStatus, message: &str| {
        println!("{message} -> {status:?}");
    };

    let log_message = |status: &fmi::types::fmiStatus, category: &str, message: &str| {
        println!("[Message][{:?}][{}] {}", status, category, message);
    };

    let unzipdir = Path::new(r"E:\WS\Reference-FMUs\rust\deploy");

    let shared_library_filename = format!("{}{}", model_description.coSimulation.unwrap().modelIdentifier, SHARED_LIBRARY_EXTENSION);

    let shared_library_path = unzipdir.join("binaries").join(PLATFORM_TUPLE).join(shared_library_filename);

    // let path = Path::new(r"E:\WS\Reference-FMUs\rust\deploy\binaries\x86_64-windows\BouncingBall.dll");

    let mut fmu = FMU3::new(
        shared_library_path.as_path(), 
        "instance1", 
        Some(Box::new(log_fmi_call)), 
        Some(Box::new(log_message))
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
