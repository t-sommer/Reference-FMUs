#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use std::path::Path;
use crate::types::fmiValueReference;


#[derive(Debug)]
pub enum VariableType {
    Float32,
    Float64,
}   

#[derive(Debug, PartialEq)]
pub enum Causality {
    Parameter,
    CalculatedParameter,
    StructuralParameter,
    Input,
    Output,
    Local,
    Independent
}

#[derive(Debug)]
pub struct DefaultExperiment {
    pub startTime: Option<String>,
    pub stopTime: Option<String>,
    pub tolerance: Option<String>,
    pub stepSize: Option<String>,
}

#[derive(Debug)]
pub struct CoSimulation {
    pub modelIdentifier: String,
}

#[derive(Debug)]
pub struct ModelVariable {
    pub variableType: VariableType,
    pub name: String,
    pub valueReference: fmiValueReference,
    pub causality: Causality,
}

#[derive(Debug)]
pub struct ModelDescription {
    pub fmiVersion: String,
    pub modelName: String,
    pub instantiationToken: String,
    pub defaultExperiment: Option<DefaultExperiment>,
    pub coSimulation: Option<CoSimulation>,
    pub modelVariables: Vec<ModelVariable>,
}

pub fn read_model_description(path: &Path) -> Result<ModelDescription, String> {

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

    let defaultExperiment = root.descendants().find(|n| n.has_tag_name("DefaultExperiment")).map(|e| 
        DefaultExperiment {
            startTime: e.attribute("startTime").map(|s| s.to_string()),
            stopTime: e.attribute("stopTime").map(|s| s.to_string()),
            tolerance: e.attribute("tolerance").map(|s| s.to_string()),
            stepSize: e.attribute("stepSize").map(|s| s.to_string()),
        }
    );

    // let defaultExperiment = if let Some(e) = root.descendants().find(|n| n.has_tag_name("DefaultExperiment")) {
    //     Some(
    //         DefaultExperiment {
    //             startTime: e.attribute("startTime").map(|s| s.to_string()),
    //             stopTime: e.attribute("stopTime").map(|s| s.to_string()),
    //             tolerance: e.attribute("tolerance").map(|s| s.to_string()),
    //             stepSize: e.attribute("stepSize").map(|s| s.to_string()),
    //         }
    //     )
    // } else {
    //     None
    // };

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
        defaultExperiment,
        coSimulation,
        modelVariables,
    };

    Ok(model_description)
}
