use crate::model_description::Unit;
use crate::model_description::file::StringAttribute;
use roxmltree::Node;
use std::{error::Error, path::Path};

use crate::model_description::fmi2::{
    Causality, CoSimulation, DefaultExperiment, DependencyKind, Initial, ModelDescription,
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

        let doc = roxmltree::Document::parse_with_options(&text, opt).unwrap();

        let root = &doc.root_element();

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

            let canHandleMultipleSetPerTimeInstant =
                child.bool_attribute("canHandleMultipleSetPerTimeInstant", false);

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
            .descendants()
            .find(|n| n.has_tag_name("DefaultExperiment"))
            .map(|e| DefaultExperiment {
                startTime: e.optional_attribute("startTime"),
                stopTime: e.optional_attribute("stopTime"),
                tolerance: e.optional_attribute("tolerance"),
                stepSize: e.optional_attribute("stepSize"),
            });

        let coSimulation =
            if let Some(cs) = root.descendants().find(|n| n.has_tag_name("CoSimulation")) {
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

        let unitDefintions = root
            .descendants()
            .find(|n| n.has_tag_name("UnitDefinitions"))
            .map(|u| u.descendants())
            .into_iter()
            .flatten()
            .filter(|n| n.has_tag_name("Unit"))
            .into_iter()
            .map(|u| Unit::from_node(&u))
            .collect();

        let numberOfEventIndicators = if let Some(n) = root.attribute("numberOfEventIndicators") {
            n.parse().unwrap_or(0)
        } else {
            0
        };
        let outputs = Self::get_unkonwns(root, "Outputs")?;
        let derivatives = Self::get_unkonwns(root, "Derivatives").unwrap_or_default();
        let initialUnknowns = Self::get_unkonwns(root, "InitialUnknowns")?;
        let model_description = ModelDescription {
            modelName: root.required_attribute("modelName")?,
            guid: root.required_attribute("guid")?,
            description: root.optional_attribute("description"),
            author: root.optional_attribute("author"),
            version: root.optional_attribute("version"),
            copyright: root.optional_attribute("copyright"),
            license: root.optional_attribute("license"),
            generationTool: root.optional_attribute("generationTool"),
            generationDateAndTime: root.optional_attribute("generationDateAndTime"),
            variableNamingConvention: root
                .optional_attribute("variableNamingConvention")
                .unwrap_or("flat".to_string())
                .parse()?,
            defaultExperiment,
            coSimulation,
            modelExchange,
            unitDefintions,
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
                    declaredType: child.optional_attribute("declaredType"),
                    quantity: child.optional_attribute("quantity"),
                    unit: child.optional_attribute("unit"),
                    displayUnit: child.optional_attribute("displayUnit"),
                    relativeQuantity: child
                        .attribute("relativeQuantity")
                        .map(|s| s == "true")
                        .unwrap_or(false),
                    min: child.optional_attribute("min"),
                    max: child.optional_attribute("max"),
                    nominal: child.optional_attribute("nominal"),
                    unbounded: child
                        .attribute("unbounded")
                        .map(|s| s == "true")
                        .unwrap_or(false),
                    start: child.optional_attribute("start"),
                    derivative: child
                        .optional_attribute("derivative")
                        .map(|s| s.parse().unwrap()),
                    reinit: child
                        .attribute("reinit")
                        .map(|s| s == "true")
                        .unwrap_or(false),
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

    fn get_unkonwns(root: &Node, name: &str) -> Result<Vec<Unknown>, Box<dyn Error>> {
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

            let dependencies: Option<Vec<u32>> = match child.attribute("dependencies") {
                Some(dependencies) => {
                    if dependencies.is_empty() {
                        Some(Vec::new())
                    } else {
                        Some(
                            dependencies
                                .split(" ")
                                .map(|s| s.parse::<u32>().unwrap())
                                .collect(),
                        )
                    }
                }
                None => None,
            };

            let dependenciesKind: Option<Vec<DependencyKind>> =
                match child.attribute("dependenciesKind") {
                    Some(dependenciesKind) => {
                        if dependenciesKind.is_empty() {
                            Some(Vec::new())
                        } else {
                            Some(
                                dependenciesKind
                                    .split(" ")
                                    .map(|s| match s {
                                        "dependent" => DependencyKind::Dependent,
                                        "constant" => DependencyKind::Constant,
                                        "fixed" => DependencyKind::Fixed,
                                        "tunable" => DependencyKind::Tunable,
                                        "discrete" => DependencyKind::Discrete,
                                        _ => panic!("Unknown dependenciesKind: {}", s),
                                    })
                                    .collect(),
                            )
                        }
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
