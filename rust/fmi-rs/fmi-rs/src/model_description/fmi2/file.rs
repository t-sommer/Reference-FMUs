use crate::model_description::file::NodeExt;
use crate::model_description::{Unit, fmi2::SimpleType};
use roxmltree::Node;
use std::str::FromStr;
use std::vec;
use std::{error::Error, path::Path};

use crate::model_description::fmi2::{
    Causality, CoSimulation, DefaultExperiment, DependencyKind, Initial, Item, ModelDescription,
    ModelExchange, ScalarVariable, Unknown, Variability, VariableType,
};

impl ModelDescription {
    pub fn read(path: &Path) -> Result<ModelDescription, Box<dyn Error>> {
        let text = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => return Err(format!("Failed to read XML file: {}", e).into()),
        };

        let opt = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..roxmltree::ParsingOptions::default()
        };

        let doc = roxmltree::Document::parse_with_options(&text, opt)?;

        let root = &doc.root_element();

        let mut modelVariables = vec![];

        for child in root.get_required_child("ModelVariables")?.get_children("ScalarVariable")
        {
            let name = child.required_attribute("name")?;

            let valueReference = child.required_attribute("valueReference")?.parse()?;

            let description = child.attribute_as("description")?;

            let canHandleMultipleSetPerTimeInstant =
                child.attribute_as("canHandleMultipleSetPerTimeInstant")?.unwrap_or_default();

            let variableType = Self::get_variable_type(&child)?;

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
                    if matches!(variableType, VariableType::Real { .. })
                        && !matches!(
                            causality,
                            Causality::Parameter | Causality::CalculatedParameter
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
            .get_child("DefaultExperiment")
            .map(|e| DefaultExperiment {
                startTime: e.attribute_as("startTime").unwrap(),
                stopTime: e.attribute_as("stopTime").unwrap(),
                tolerance: e.attribute_as("tolerance").unwrap(),
                stepSize: e.attribute_as("stepSize").unwrap(),
            });

        let coSimulation =
            if let Some(cs) = root.get_child("CoSimulation") {
                Some(CoSimulation {
                    modelIdentifier: cs.required_attribute("modelIdentifier")?,
                    providesDirectionalDerivatives: cs
                        .attribute_as("providesDirectionalDerivative")?.unwrap_or_default(),
                    fixedInternalStepSize: cs.attribute_as("fixedInternalStepSize")?,
                    canHandleVariableCommunicationStepSize: cs
                        .attribute_as("canHandleVariableCommunicationStepSize")?.unwrap_or_default(),
                    canNotUseMemoryManagementFunctions: cs
                        .attribute_as("canNotUseMemoryManagementFunctions")?.unwrap_or_default(),
                })
            } else {
                None
            };

        let modelExchange =
            if let Some(me) = root.get_child("ModelExchange") {
                Some(ModelExchange {
                    modelIdentifier: me.required_attribute("modelIdentifier")?,
                    providesDirectionalDerivatives: me
                        .attribute_as("providesDirectionalDerivative")?.unwrap_or_default(),
                    needsCompletedIntegratorStep: !me
                        .attribute_as::<bool>("completedIntegratorStepNotNeeded")?.unwrap_or_default(),
                    canNotUseMemoryManagementFunctions: me
                        .attribute_as("canNotUseMemoryManagementFunctions")?.unwrap_or_default(),
                })
            } else {
                None
            };

        let unitDefintions = root
            .get_child("UnitDefintions")
            .map(|n| n.get_children("Unit"))
            .into_iter()
            .flatten()
            .map(|n| Unit::from_node(&n))
            .collect::<Result<Vec<_>, _>>()?;

        let typeDefinitions = root
            .get_child("TypeDefinitions")
            .map(|n| n.get_child("SimpleType"))
            .into_iter()
            .flatten()
            .map(|n| SimpleType::from_node(&n))
            .collect::<Result<Vec<_>, _>>()?;

        let numberOfEventIndicators = root.attribute_as("numberOfEventIndicators")?.unwrap_or_default();
        let outputs = Self::get_unkonwns(root, "Outputs")?;
        let derivatives = Self::get_unkonwns(root, "Derivatives")?;
        let initialUnknowns = Self::get_unkonwns(root, "InitialUnknowns")?;
        let model_description = ModelDescription {
            modelName: root.required_attribute("modelName")?,
            guid: root.required_attribute("guid")?,
            description: root.attribute_as("description")?,
            author: root.attribute_as("author")?,
            version: root.attribute_as("version")?,
            copyright: root.attribute_as("copyright")?,
            license: root.attribute_as("license")?,
            generationTool: root.attribute_as("generationTool")?,
            generationDateAndTime: root.attribute_as("generationDateAndTime")?,
            variableNamingConvention: root
                .attribute_as("variableNamingConvention")?
                .unwrap_or("flat".to_string())
                .parse()?,
            defaultExperiment,
            coSimulation,
            modelExchange,
            unitDefintions,
            typeDefinitions,
            modelVariables,
            numberOfEventIndicators,
            outputs,
            derivatives,
            initialUnknowns,
        };

        Ok(model_description)
    }

    fn get_variable_type(node: &Node) -> Result<VariableType, Box<dyn Error>> {
        for child in node.children() {
            if child.has_tag_name("Real") {
                return Ok(VariableType::Real {
                    declaredType: child.attribute_as("declaredType")?,
                    quantity: child.attribute_as("quantity")?,
                    unit: child.attribute_as("unit")?,
                    displayUnit: child.attribute_as("displayUnit")?,
                    relativeQuantity: child
                        .attribute("relativeQuantity")
                        .map(|s| s == "true")
                        .unwrap_or(false),
                    min: child.attribute_as("min")?,
                    max: child.attribute_as("max")?,
                    nominal: child.attribute_as("nominal")?,
                    unbounded: child
                        .attribute("unbounded")
                        .map(|s| s == "true")
                        .unwrap_or(false),
                    start: child.attribute_as("start")?,
                    derivative: child.attribute_as("derivative")?,
                    reinit: child.attribute_as("reinit")?.unwrap_or_default(),
                });
            } else if child.has_tag_name("Integer") {
                return Ok(VariableType::Integer {
                    declaredType: child.attribute_as("declaredType")?,
                    quantity: child.attribute_as("quantity")?,
                    min: child.attribute_as("min")?,
                    max: child.attribute_as("max")?,
                    start: child.attribute_as("start")?,
                });
            } else if child.has_tag_name("Boolean") {
                return Ok(VariableType::Boolean {
                    declaredType: child.attribute_as("declaredType")?,
                    start: child.attribute_as("start")?,
                });
            } else if child.has_tag_name("String") {
                return Ok(VariableType::String {
                    declaredType: child.attribute_as("declaredType")?,
                    start: child.attribute_as("start")?,
                });
            } else if child.has_tag_name("Enumeration") {
                return Ok(VariableType::Enumeration {
                    declaredType: child.required_attribute("declaredType")?,
                    quantity: child.attribute_as("quantity")?,
                    min: child.attribute_as("min")?,
                    max: child.attribute_as("max")?,
                    start: child.attribute_as("start")?,
                });
            }
        }

        Err("Missing variable type element".into())
    }

    fn get_unkonwns(root: &Node, name: &str) -> Result<Vec<Unknown>, Box<dyn Error>> {
        let modelStructure = if let Some(modelStructure) = root.get_child("ModelStructure") {
            modelStructure
        } else {
            return Ok(Vec::new());
        };

        let container = if let Some(container) = modelStructure.get_child(name) {
            container
        } else {
            return Ok(Vec::new());
        };

        let mut unkonwns = vec![];

        for child in container.get_children("Unknown") {
            let index = child
                .required_attribute("index")?
                .parse()?;

            let dependencies: Option<Vec<u32>> = match child.attribute("dependencies") {
                Some(dependencies) => {
                    Some(dependencies.split_whitespace().map(|s| s.parse::<u32>()).collect::<Result<Vec<_>, _>>()?)
                }
                None => None,
            };

            let dependenciesKind: Option<Vec<DependencyKind>> =
                match child.attribute("dependenciesKind") {
                    Some(dependenciesKind) => {
                        Some(dependenciesKind.split_whitespace().map(DependencyKind::from_str).collect::<Result<Vec<_>, _>>()?)
                    }
                    None => None,
                };

            unkonwns.push(Unknown {
                index,
                dependencies,
                dependenciesKind,
            });
        }

        Ok(unkonwns)
    }
}

impl SimpleType {
    fn from_node(node: &Node) -> Result<Self, Box<dyn Error>> {
        let name = node.required_attribute("name")?;
        let description = node.attribute_as("description")?;

        for child in node.children() {
            if child.has_tag_name("Real") {
                return Ok(SimpleType::Real {
                    name,
                    description,
                    quantity: child.attribute_as("quantity")?,
                    unit: child.attribute_as("unit")?,
                    displayUnit: child.attribute_as("displayUnit")?,
                    relativeQuantity: child.attribute_as("relativeQuantity")?.unwrap_or_default(),
                    unbounded: child.attribute_as("unbounded")?.unwrap_or_default(),
                    min: child.attribute_as("min")?,
                    max: child.attribute_as("max")?,
                    nominal: child.attribute_as("nominal")?,
                });
            } else if child.has_tag_name("Integer") {
                return Ok(SimpleType::Integer {
                    name,
                    description,
                    quantity: child.attribute_as("quantity")?,
                    min: child.attribute_as("min")?,
                    max: child.attribute_as("max")?,
                });
            } else if child.has_tag_name("Boolean") {
                return Ok(SimpleType::Boolean { name, description });
            } else if child.has_tag_name("String") {
                return Ok(SimpleType::String { name, description });
            } else if child.has_tag_name("Enumeration") {
                let mut items = vec![];
                for grand_child in child.children() {
                    if grand_child.has_tag_name("Item") {
                        items.push(Item::from_node(&grand_child)?);
                    }
                }
                return Ok(SimpleType::Enumeration {
                    name,
                    description,
                    items,
                    quantity: child.attribute_as("quantity")?,
                });
            }
        }

        Err("Missing variable type element".into())
    }
}

impl Item {
    pub(crate) fn from_node(node: &roxmltree::Node) -> Result<Self, Box<dyn Error>> {
        Ok(Item {
            name: node.required_attribute("name")?,
            description: node.attribute_as("description")?,
            value: node.required_attribute_as("value")?,
        })
    }
}
