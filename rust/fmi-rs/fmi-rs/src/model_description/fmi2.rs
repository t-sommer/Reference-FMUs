#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use roxmltree::Node;
use std::{error::Error, path::Path, str::FromStr};

use crate::types::fmiValueReference;

pub type VariableIndex = u32;


#[derive(Debug)]
pub enum VariableType {
    Real {
        declaredType: Option<String>,
        quantity: Option<String>,
        unit: Option<String>,
        displayUnit: Option<String>,
        relativeQuantity: bool,
        min: Option<String>,
        max: Option<String>,
        nominal: Option<String>,
        unbounded: bool,
        start: Option<String>,
        derivative: Option<VariableIndex>,
        reinit: bool,
    },
    Integer {
        declaredType: Option<String>,
        quantity: Option<String>,
        min: Option<String>,
        max: Option<String>,
        start: Option<String>,
    },
    Boolean {
        declaredType: Option<String>,
        start: Option<String>,
    },
    String {
        declaredType: Option<String>,
        start: Option<String>,
    },
    Enumeration {
        declaredType: String,
        quantity: Option<String>,
        min: Option<String>,
        max: Option<String>,
        start: Option<String>,
    },
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Causality {
    Parameter,
    CalculatedParameter,
    Input,
    Output,
    Local,
    Independent,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Variability {
    Constant,
    Fixed,
    Tunable,
    Discrete,
    Continuous,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Initial {
    Exact,
    Approx,
    Calculated,
}

impl FromStr for Initial {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exact" => Ok(Initial::Exact),
            "approx" => Ok(Initial::Approx),
            "calculated" => Ok(Initial::Calculated),
            _ => Err(format!("Unknown initial value: {}", s)),
        }
    }
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
    pub providesDirectionalDerivatives: bool,
    pub fixedInternalStepSize: Option<String>,
    pub canHandleVariableCommunicationStepSize: bool,
    pub canNotUseMemoryManagementFunctions: bool,
}

#[derive(Debug)]
pub struct ModelExchange {
    pub modelIdentifier: String,
    pub providesDirectionalDerivatives: bool,
    pub needsCompletedIntegratorStep: bool,
    pub canNotUseMemoryManagementFunctions: bool,
}

#[derive(Debug)]
pub struct ScalarVariable {
    pub variableType: VariableType,
    pub name: String,
    pub valueReference: fmiValueReference,
    pub description: Option<String>,
    pub causality: Causality,
    pub variability: Variability,
    pub initial: Initial,
    pub canHandleMultipleSetPerTimeInstant: bool,
}

#[derive(Debug)]
pub struct Unknown {
    pub valueReference: fmiValueReference,
    pub dependencies: Option<Vec<u32>>,
    pub dependenciesKind: Option<Vec<u32>>,
}

#[derive(Debug)]
pub struct ModelDescription {
    pub modelName: String,
    pub instantiationToken: String,
    pub defaultExperiment: Option<DefaultExperiment>,
    pub modelExchange: Option<ModelExchange>,
    pub coSimulation: Option<CoSimulation>,
    pub modelVariables: Vec<ScalarVariable>,
    pub numberOfEventIndicators: usize,
    pub outputs: Vec<Unknown>,
    pub derivatives: Vec<Unknown>,
    pub clockedStates: Vec<Unknown>,
    pub eventIndicators: Vec<Unknown>,
    pub initialUnknowns: Vec<Unknown>,
}

fn get_variable_type(node: &Node) -> Result<VariableType, Box<dyn Error>> {

    for child in node.children() {
        if child.has_tag_name("Real") {
            return Ok(VariableType::Real {
                declaredType: child.optional_attribute("declaredType"),
                quantity: child.optional_attribute("quantity"),
                unit: child.optional_attribute("unit"),
                displayUnit: child.optional_attribute("displayUnit"),
                relativeQuantity: child.attribute("relativeQuantity").map(|s| s == "true").unwrap_or(false),
                min: child.optional_attribute("min"),
                max: child.optional_attribute("max"),
                nominal: child.optional_attribute("nominal"),
                unbounded: child.attribute("unbounded").map(|s| s == "true").unwrap_or(false),
                start: child.optional_attribute("start"),
                derivative: child.optional_attribute("derivative").map(|s| s.parse().unwrap()),
                reinit: child.attribute("reinit").map(|s| s == "true").unwrap_or(false),
            });
        } else if child.has_tag_name("Integer") {
            return Ok(VariableType::Integer {
                declaredType: child.optional_attribute("declaredType"),
                quantity: child.optional_attribute("quantity"),
                min: child.optional_attribute("min"),
                max: child.optional_attribute("max"),
                start: child.optional_attribute("start"),
            });
        } else if child.has_tag_name("Boolean") {
            return Ok(VariableType::Boolean {
                declaredType: child.optional_attribute("declaredType"),
                start: child.optional_attribute("start"),
            });
        } else if child.has_tag_name("String") {
            return Ok(VariableType::String {
                declaredType: child.optional_attribute("declaredType"),
                start: child.optional_attribute("start"),
            });
        } else if child.has_tag_name("Enumeration") {
            return Ok(VariableType::Enumeration {
                declaredType: child.required_attribute("declaredType")?,
                quantity: child.optional_attribute("quantity"),
                min: child.optional_attribute("min"),
                max: child.optional_attribute("max"),
                start: child.optional_attribute("start"),
            });
        }
    }

    Err("Missing variable type element".into())
}

fn get_fmi2_unkonwns(root: &Node, name: &str) -> Result<Vec<Unknown>, Box<dyn Error>> {
    let modelStructure = root
        .descendants()
        .find(|n| n.has_tag_name("ModelStructure"))
        .ok_or("Missing ModelStructure element.")?;

    let container = modelStructure
        .descendants()
        .find(|n| n.has_tag_name(name))
        .ok_or(format!("Missing container element '{name}'."))?;

    let mut unkonwns = vec![];

    for child in container
        .descendants()
        .filter(|n| n.has_tag_name("Unknown"))
    {
        let index = child
            .attribute("index")
            .ok_or(format!("Missing index attribute in {}", name))?
            .parse()
            .unwrap();

        unkonwns.push(Unknown {
            valueReference: index,
            dependencies: None,
            dependenciesKind: None,
        });
    }

    Ok(unkonwns)
}

pub fn read_model_description(path: &Path) -> Result<ModelDescription, Box<dyn Error>> {
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

    read_fmi2_model_description(&root)
}

fn read_fmi2_model_description(root: &Node) -> Result<ModelDescription, Box<dyn Error>> {
    let ModelVariables = root
        .descendants()
        .find(|n| n.has_tag_name("ModelVariables"))
        .unwrap();

    let mut modelVariables = vec![];

    for (_i, child) in ModelVariables
        .children()
        .filter(|n| n.has_tag_name("ScalarVariable"))
        .enumerate()
    {
        let name = child.required_attribute("name")?;
        let valueReference = child.required_attribute("valueReference")?.parse().unwrap();
        let description = child.optional_attribute("description");
        let canHandleMultipleSetPerTimeInstant = child
            .bool_attribute("canHandleMultipleSetPerTimeInstant", false);

        let variableType = get_variable_type(&child)?;

        let causality = match child.attribute("causality") {
            Some("parameter") => Causality::Parameter,
            Some("calculatedParameter") => Causality::CalculatedParameter,
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
            _ => {
                if matches!(variableType, VariableType::Real {..})
                    && !matches!(
                        causality,
                        Causality::Parameter
                            | Causality::CalculatedParameter
                    )
                {
                    Variability::Continuous
                } else {
                    Variability::Discrete
                }
            }
        };

        let initial = match child.attribute("initial") {
            Some("exact") => Initial::Exact,
            Some("approx") => Initial::Approx,
            Some("calculated") => Initial::Calculated,
            _ => Initial::Exact,
        };

        let variable = ScalarVariable {
            variableType,
            name,
            valueReference,
            description,
            causality,
            variability,
            initial,
            canHandleMultipleSetPerTimeInstant,
        };

        modelVariables.push(variable);
    }

    let defaultExperiment = root
        .descendants()
        .find(|n| n.has_tag_name("DefaultExperiment"))
        .map(|e| DefaultExperiment {
            startTime: e.optional_attribute("startTime"),
            stopTime: e.optional_attribute("stopTime"),
            tolerance: e.optional_attribute("tolerance"),
            stepSize: e.optional_attribute("stepSize"),
        });

    let coSimulation = if let Some(cs) = root.descendants().find(|n| n.has_tag_name("CoSimulation"))
    {
        Some(CoSimulation {
            modelIdentifier: cs.required_attribute("modelIdentifier")?,
            providesDirectionalDerivatives: cs
                .bool_attribute("providesDirectionalDerivative", false),
            fixedInternalStepSize: cs.optional_attribute("fixedInternalStepSize"),
            canHandleVariableCommunicationStepSize: cs
                .bool_attribute("canHandleVariableCommunicationStepSize", false),
            canNotUseMemoryManagementFunctions: cs
                .bool_attribute("canNotUseMemoryManagementFunctions", false),
        })
    } else {
        None
    };

    let modelExchange =
        if let Some(me) = root.descendants().find(|n| n.has_tag_name("ModelExchange")) {
            Some(ModelExchange {
                modelIdentifier: me.required_attribute("modelIdentifier")?,
                providesDirectionalDerivatives: me
                    .bool_attribute("providesDirectionalDerivative", false),
                needsCompletedIntegratorStep: !me
                    .bool_attribute("completedIntegratorStepNotNeeded", false),
                canNotUseMemoryManagementFunctions: me
                    .bool_attribute("canNotUseMemoryManagementFunctions", false),
            })
        } else {
            None
        };

    let numberOfEventIndicators = if let Some(n) = root.attribute("numberOfEventIndicators") {
        n.parse().unwrap_or(0)
    } else {
        0
    };

    let outputs = get_fmi2_unkonwns(root, "Outputs")?;
    let derivatives = get_fmi2_unkonwns(root, "Derivatives").unwrap_or_default();
    let initialUnknowns = get_fmi2_unkonwns(root, "InitialUnknowns")?;

    let model_description = ModelDescription {
        modelName: root.required_attribute("modelName")?,
        instantiationToken: root.required_attribute("guid")?,
        defaultExperiment,
        coSimulation,
        modelExchange,
        modelVariables,
        numberOfEventIndicators,
        outputs,
        derivatives,
        clockedStates: vec![],
        eventIndicators: vec![],
        initialUnknowns,
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
