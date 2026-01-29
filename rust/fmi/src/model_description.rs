#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use std::{collections::HashMap, path::Path};
use crate::types::fmiValueReference;


#[derive(Debug)]
pub enum VariableType {
    Float32,
    Float64,
    UInt64,
}   

#[derive(Debug, PartialEq, Eq, Hash)]
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
pub enum Dimension {
    Fixed(usize),
    Variable(fmiValueReference),
}

#[derive(Debug)]
pub struct ModelVariable {
    pub variableType: VariableType,
    pub name: String,
    pub valueReference: fmiValueReference,
    pub causality: Causality,
    pub dimensions: Vec<Dimension>,
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
    
    let mut variables_for_vr = HashMap::new();
    let mut modelVariables = vec![];

    for (i, child) in ModelVariables.children().filter(|n| n.is_element()).enumerate() {

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

        let variableType = match child.tag_name().name() {
            "Float64" => VariableType::Float64,
            "Float32" => VariableType::Float32,
            "UInt64" => VariableType::UInt64,
            _ => todo!(),
        };

        let mut dimensions = vec![];

        for grand_child in child.children().filter(|e| e.has_tag_name("Dimension")) {
            let dimension = if let Some(value) = grand_child.attribute("start") {
                Dimension::Fixed(value.parse().unwrap())
            } else {
                Dimension::Variable(grand_child.attribute("valueReference").unwrap().parse().unwrap())
            };
            dimensions.push(dimension);
        }

        let variable = ModelVariable {
            variableType,
            name: name.to_string(),
            valueReference: valueReference,
            causality,
            dimensions
        };

        variables_for_vr.insert(valueReference, i);
        modelVariables.push(variable);

    }

    let defaultExperiment = root.descendants().find(|n| n.has_tag_name("DefaultExperiment")).map(|e| 
        DefaultExperiment {
            startTime: e.attribute("startTime").map(|s| s.to_string()),
            stopTime: e.attribute("stopTime").map(|s| s.to_string()),
            tolerance: e.attribute("tolerance").map(|s| s.to_string()),
            stepSize: e.attribute("stepSize").map(|s| s.to_string()),
        }
    );

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
