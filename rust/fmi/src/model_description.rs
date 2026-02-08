// #![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use std::{collections::HashMap, error::Error, path::Path, str::FromStr};
use roxmltree::Node;

use crate::types::fmiValueReference;

#[derive(Debug, PartialEq)]
pub enum MajorVersion {
    V2,
    V3,
}

#[derive(Debug)]
pub enum VariableType {
    Float32,
    Float64,
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Boolean,
    String,
    Binary,
    Clock,
    Enumeration,
}

impl FromStr for VariableType {

    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Float32" => Ok(VariableType::Float32),
            "Float64" => Ok(VariableType::Float64),
            "Int8" => Ok(VariableType::Int8),
            "UInt8" => Ok(VariableType::UInt8),
            "Int16" => Ok(VariableType::Int16),
            "UInt16" => Ok(VariableType::UInt16),
            "Int32" => Ok(VariableType::Int32),
            "UInt32" => Ok(VariableType::UInt32),
            "Int64" => Ok(VariableType::Int64),
            "UInt64" => Ok(VariableType::UInt64),
            "Boolean" => Ok(VariableType::Boolean),
            "String" => Ok(VariableType::String),
            "Binary" => Ok(VariableType::Binary),
            "Clock" => Ok(VariableType::Clock),
            "Enumeration" => Ok(VariableType::Enumeration),
            _ => Err(format!("Unknown variable type: {}", s)),
        }
    }
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Variability {
    Constant, 
    Fixed, 
    Tunable, 
    Discrete, 
    Continuous
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
    pub fixedInternalStepSize: Option<String>,
    pub canHandleVariableCommunicationStepSize: bool,
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
    pub variability: Variability,
    pub dimensions: Vec<Dimension>,
}

#[derive(Debug)]
pub struct ModelDescription {
    pub majorVersion: MajorVersion,
    pub modelName: String,
    pub instantiationToken: String,
    pub defaultExperiment: Option<DefaultExperiment>,
    pub coSimulation: Option<CoSimulation>,
    pub modelVariables: Vec<ModelVariable>,
}

pub fn read_model_description(path: &Path) -> Result<ModelDescription, Box<dyn Error>> {

    let text = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => return Err(format!("Failed to read XML file: {}", e).into())
    };

    let opt = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..roxmltree::ParsingOptions::default()
    };

    let doc = roxmltree::Document::parse_with_options(&text, opt).unwrap();

    let root = doc.root_element();

    if let Some(fmi_version) = root.attribute("fmiVersion") {
        match fmi_version {
            "2.0" => read_fmi2_model_description(&root),
            "3.0" => read_fmi3_model_description(&root),
            _ => Err(format!("Unsupported FMI version: {fmi_version}").into()),
        }
    } else {
        Err("Attribute fmiVersion is missing.".into())
    }
}


fn get_variable_type(node: &Node) -> Result<VariableType, Box<dyn Error>> {

    for child in node.children() {
        if child.has_tag_name("Real") {
            return Ok(VariableType::Float64);
        } else if child.has_tag_name("Integer") {
            return Ok(VariableType::Int32);
        } else if child.has_tag_name("Boolean") {
            return Ok(VariableType::Boolean);
        } else if child.has_tag_name("String") {
            return Ok(VariableType::String);
        } else if child.has_tag_name("Enumeration") {
            return Ok(VariableType::Enumeration);
        }
    }

    Err("Missing variable type element".into())
}

fn read_fmi2_model_description(root: &Node) -> Result<ModelDescription, Box<dyn Error>> {

    let ModelVariables = root.descendants().find(|n| n.has_tag_name("ModelVariables")).unwrap();
    
    let mut modelVariables = vec![];

    for (i, child) in ModelVariables.children().filter(|n| n.has_tag_name("ScalarVariable")).enumerate() {

        let name = child.required_attribute("name")?;
        let valueReference = child.required_attribute("valueReference")?.parse().unwrap();

        let variableType = get_variable_type(&child)?;

        let causality = match child.attribute("causality") {
            Some("parameter") => Causality::Parameter,
            Some("calculatedParameter") => Causality::CalculatedParameter,
            Some("structuralParameter") => Causality::StructuralParameter,
            Some("input") => Causality::Input,
            Some("output") => Causality::Output,
            Some("independent") => Causality::Independent,
            _ => Causality::Local,
        };

        let variability = match child.attribute("variability") {
            Some("constant") => Variability::Constant,
            Some("fixed") => Variability::Fixed,
            Some("tunable") => Variability::Tunable,
            Some("discrete") => Variability::Discrete,
            Some("continuous") => Variability::Continuous,
            _ => { if matches!(variableType, VariableType::Float32 | VariableType::Float64) 
                && !matches!(causality, Causality::Parameter | Causality::StructuralParameter | Causality::CalculatedParameter) {
                    Variability::Continuous
                } else {
                    Variability::Discrete
                }
            },

        };

        let dimensions = vec![];

        let variable = ModelVariable {
            variableType,
            name,
            valueReference,
            causality,
            variability,
            dimensions,
        };

        modelVariables.push(variable);

    }

    let defaultExperiment = root.descendants().find(|n| n.has_tag_name("DefaultExperiment")).map(|e| 
        DefaultExperiment {
            startTime: e.optional_attribute("startTime"),
            stopTime: e.optional_attribute("stopTime"),
            tolerance: e.optional_attribute("tolerance"),
            stepSize: e.optional_attribute("stepSize"),
        }
    );

    let coSimulation = if let Some(cs) = root.descendants().find(|n| n.has_tag_name("CoSimulation")) {
        Some(
            CoSimulation {
                modelIdentifier: cs.required_attribute("modelIdentifier")?,
                fixedInternalStepSize: cs.optional_attribute("fixedInternalStepSize"),
                canHandleVariableCommunicationStepSize: cs.bool_attribute("canHandleVariableCommunicationStepSize", false),
            }
        )
    } else {
        None
    };

    let model_description = ModelDescription {
        majorVersion: MajorVersion::V2,
        modelName: root.required_attribute("modelName")?,
        instantiationToken: root.required_attribute("guid")?,
        defaultExperiment,
        coSimulation,
        modelVariables,
    };

    Ok(model_description)
}

trait StringAttribute {
    fn optional_attribute(&self, name: &str) -> Option<String>;
    fn required_attribute(&self, name: &str) -> Result<String, Box<dyn Error>>;
    fn bool_attribute(&self, name: &str, default: bool) -> bool;
}

impl<'a, 'input> StringAttribute for Node<'a, 'input> {

    fn required_attribute(&self, name: &str) -> Result<String, Box<dyn Error>> {
        self.attribute(name)
            .ok_or_else(|| format!("Missing required attribute '{}'", name).into())
            .map(|s| s.to_string())
    }
    
    fn optional_attribute(&self, name: &str) -> Option<String> {
        self.attribute(name).map(|v| v.to_string())
    }

    fn bool_attribute(&self, name: &str, default: bool) -> bool {
        if let Some(value) = self.attribute(name) {
            value.parse().unwrap_or(default)
        } else {
            default
        }
    }
}

fn read_fmi3_model_description(root: &Node) -> Result<ModelDescription, Box<dyn Error>> {

    let ModelVariables = root.descendants().find(|n| n.has_tag_name("ModelVariables")).unwrap();
    
    let mut variables_for_vr = HashMap::new();
    let mut modelVariables = vec![];

    for (i, child) in ModelVariables.children().filter(|n| n.is_element()).enumerate() {

        let name = child.attribute("name").unwrap();
        let valueReference = child.attribute("valueReference").unwrap().parse().unwrap();

        let variableType = VariableType::from_str(child.tag_name().name())?;

        let causality = match child.attribute("causality") {
            Some("parameter") => Causality::Parameter,
            Some("calculatedParameter") => Causality::CalculatedParameter,
            Some("structuralParameter") => Causality::StructuralParameter,
            Some("input") => Causality::Input,
            Some("output") => Causality::Output,
            Some("independent") => Causality::Independent,
            _ => Causality::Local,
        };

        let variability = match child.attribute("variability") {
            Some("constant") => Variability::Constant,
            Some("fixed") => Variability::Fixed,
            Some("tunable") => Variability::Tunable,
            Some("discrete") => Variability::Discrete,
            Some("continuous") => Variability::Continuous,
            _ => { if matches!(variableType, VariableType::Float32 | VariableType::Float64) 
                && !matches!(causality, Causality::Parameter | Causality::StructuralParameter | Causality::CalculatedParameter) {
                    Variability::Continuous
                } else {
                    Variability::Discrete
                }
            },

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
            variability,
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
                modelIdentifier: cs.attribute("modelIdentifier").unwrap().to_string(),
                fixedInternalStepSize: cs.attribute("fixedInternalStepSize").map(|s| s.to_string()),
                canHandleVariableCommunicationStepSize: cs.bool_attribute("canHandleVariableCommunicationStepSize", false),
            }
        )
    } else {
        None
    };

    let model_description = ModelDescription {
        majorVersion: MajorVersion::V3,
        modelName: root.attribute("modelName").unwrap().to_string(),
        instantiationToken: root.attribute("instantiationToken").unwrap().to_string(),
        defaultExperiment,
        coSimulation,
        modelVariables,
    };

    Ok(model_description)
}