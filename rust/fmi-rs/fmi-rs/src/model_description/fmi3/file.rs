use std::{error::Error, path::Path, str::FromStr};

use roxmltree::Node;

use crate::model_description::Unit;
use crate::model_description::file::NodeExt;

use crate::model_description::fmi3::VariableNamingConvention;
use crate::model_description::fmi3::{
    Causality, CoSimulation, DefaultExperiment, DependencyKind, Dimension, IntervalVariability,
    Item, ModelDescription, ModelExchange, ModelVariable, ScheduledExecution, TypeDefinition,
    Unknown, Variability, VariableType,
};

impl ModelDescription {
    fn get_unknowns(root: &Node, name: &str) -> Result<Vec<Unknown>, Box<dyn Error>> {
        let modelStructure = root
            .descendants()
            .find(|n| n.has_tag_name("ModelStructure"))
            .ok_or("Missing <ModelStructure> element.")?;

        let mut unknowns = vec![];

        for child in modelStructure.children().filter(|n| n.has_tag_name(name)) {
            let valueReference = child
                .attribute("valueReference")
                .ok_or_else(|| format!("Missing valueReference attribute in {}", name))?
                .parse()?;

            let dependencies: Option<Vec<u32>> = match child.attribute("dependencies") {
                Some(dependencies) => {
                    if dependencies.is_empty() {
                        Some(Vec::new())
                    } else {
                        Some(
                            dependencies
                                .split_whitespace()
                                .map(|s| s.parse::<u32>())
                                .collect::<Result<Vec<_>, _>>()?,
                        )
                    }
                }
                None => None,
            };

            let dependenciesKind: Option<Vec<DependencyKind>> =
                match child.attribute("dependenciesKind") {
                    Some(dependenciesKind) => {
                        let mut kinds = vec![];
                        for kind in dependenciesKind.split_whitespace() {
                            kinds.push(DependencyKind::from_str(kind)?);
                        }
                        Some(kinds)
                    }
                    None => None,
                };

            unknowns.push(Unknown {
                valueReference,
                dependencies,
                dependenciesKind,
                textPos: child.text_pos(),
            });
        }

        Ok(unknowns)
    }

    fn get_variable_type(node: &Node) -> Result<VariableType, Box<dyn Error>> {
        if node.has_tag_name("Float32") {
            return Ok(VariableType::Float32 {
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
                declaredType: node.attribute_as("declaredType")?,
                initial: node.attribute_as("initial")?,
                quantity: node.attribute_as("quantity")?,
                unit: node.attribute_as("unit")?,
                displayUnit: node.attribute_as("displayUnit")?,
                relativeQuantity: node
                    .attribute("relativeQuantity")
                    .map(|s| s == "true")
                    .unwrap_or(false),
                unbounded: node
                    .attribute("unbounded")
                    .map(|s| s == "true")
                    .unwrap_or(false),
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
                nominal: node.attribute_as("nominal")?,
                start: node.attribute_as("start")?,
                derivative: node.attribute_as("derivative")?,
                reinit: node
                    .attribute("reinit")
                    .map(|s| s == "true")
                    .unwrap_or(false),
            });
        } else if node.has_tag_name("Float64") {
            return Ok(VariableType::Float64 {
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
                declaredType: node.attribute_as("declaredType")?,
                initial: node.attribute_as("initial")?,
                quantity: node.attribute_as("quantity")?,
                unit: node.attribute_as("unit")?,
                displayUnit: node.attribute_as("displayUnit")?,
                relativeQuantity: node
                    .attribute("relativeQuantity")
                    .map(|s| s == "true")
                    .unwrap_or(false),
                unbounded: node
                    .attribute("unbounded")
                    .map(|s| s == "true")
                    .unwrap_or(false),
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
                nominal: node.attribute_as("nominal")?,
                start: node.attribute_as("start")?,
                derivative: node.attribute_as("derivative")?,
                reinit: node
                    .attribute("reinit")
                    .map(|s| s == "true")
                    .unwrap_or(false),
            });
        } else if node.has_tag_name("Int8") {
            return Ok(VariableType::Int8 {
                start: node.attribute_as("start")?,
                initial: node.attribute_as("initial")?,
                declaredType: node.attribute_as("declaredType")?,
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
            });
        } else if node.has_tag_name("UInt8") {
            return Ok(VariableType::UInt8 {
                start: node.attribute_as("start")?,
                initial: node.attribute_as("initial")?,
                declaredType: node.attribute_as("declaredType")?,
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
            });
        } else if node.has_tag_name("Int16") {
            return Ok(VariableType::Int16 {
                start: node.attribute_as("start")?,
                initial: node.attribute_as("initial")?,
                declaredType: node.attribute_as("declaredType")?,
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
            });
        } else if node.has_tag_name("UInt16") {
            return Ok(VariableType::UInt16 {
                start: node.attribute_as("start")?,
                initial: node.attribute_as("initial")?,
                declaredType: node.attribute_as("declaredType")?,
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
            });
        } else if node.has_tag_name("Int32") {
            return Ok(VariableType::Int32 {
                start: node.attribute_as("start")?,
                initial: node.attribute_as("initial")?,
                declaredType: node.attribute_as("declaredType")?,
                // dimensions: get_dimensions(node)?,
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
            });
        } else if node.has_tag_name("UInt32") {
            return Ok(VariableType::UInt32 {
                start: node.attribute_as("start")?,
                initial: node.attribute_as("initial")?,
                declaredType: node.attribute_as("declaredType")?,
                // dimensions: get_dimensions(node)?,
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
            });
        } else if node.has_tag_name("Int64") {
            return Ok(VariableType::Int64 {
                start: node.attribute_as("start")?,
                initial: node.attribute_as("initial")?,
                declaredType: node.attribute_as("declaredType")?,
                // dimensions: get_dimensions(node)?,
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
            });
        } else if node.has_tag_name("UInt64") {
            return Ok(VariableType::UInt64 {
                start: node.attribute_as("start")?,
                initial: node.attribute_as("initial")?,
                declaredType: node.attribute_as("declaredType")?,
                // dimensions: get_dimensions(node)?,
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
            });
        } else if node.has_tag_name("Boolean") {
            return Ok(VariableType::Boolean {
                start: node.attribute_as("start")?,
                initial: node.attribute_as("initial")?,
                declaredType: node.attribute_as("declaredType")?,
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
            });
        } else if node.has_tag_name("String") {
            return Ok(VariableType::String {
                start: node.attribute_as("start")?,
                initial: node.attribute_as("initial")?,
                declaredType: node.attribute_as("declaredType")?,
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
            });
        } else if node.has_tag_name("Binary") {
            return Ok(VariableType::Binary {
                start: None, // TODO: node.optional_attribute_as("start"),
                initial: node.attribute_as("initial")?,
                declaredType: node.attribute_as("declaredType")?,
                // dimensions: get_dimensions(node)?,
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
            });
        } else if node.has_tag_name("Clock") {
            return Ok(VariableType::Clock {
                start: node.attribute_as("start")?,
                initial: node.attribute_as("initial")?,
                declaredType: node.attribute_as("declaredType")?,
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
                canBeDeactivated: node.attribute_as("canBeDeactivated")?.unwrap_or_default(),
                priority: node.attribute_as("priority")?,
                intervalVariability: IntervalVariability::from_str(
                    node.required_attribute("intervalVariability")?.as_str(),
                )?,
                intervalDecimal: node.attribute_as("intervalDecimal")?,
                shiftDecimal: node.required_attribute("shiftDecimal")?.parse()?,
                supportsFraction: node.attribute_as("supportsFraction")?.unwrap_or_default(),
                resolution: node.attribute_as("resolution")?,
                intervalCounter: node.attribute_as("intervalCounter")?,
                shiftCounter: node.required_attribute("shiftDecimal")?.parse()?,
            });
        } else if node.has_tag_name("Enumeration") {
            return Ok(VariableType::Enumeration {
                start: node.attribute_as("start")?,
                initial: node.attribute_as("initial")?,
                declaredType: node.required_attribute("declaredType")?,
                intermediateUpdate: node.attribute_as("intermediateUpdate")?.unwrap_or_default(),
                previous: node.attribute_as("previous")?,
            });
        }

        Err("Missing variable type element".into())
    }

    fn get_dimensions(node: &Node) -> Result<Vec<Dimension>, Box<dyn Error>> {
        let mut dimensions = vec![];

        for child in node.children().filter(|n| n.is_element()) {
            if child.has_tag_name("Dimension") {
                if let Some(size_str) = child.attribute("start") {
                    let size = size_str.parse()?;
                    dimensions.push(Dimension::Fixed { start: size });
                } else if let Some(vr_str) = child.attribute("valueReference") {
                    let vr = vr_str.parse()?;
                    dimensions.push(Dimension::Variable { valueReference: vr });
                } else {
                    return Err(
                        "Dimension must have either size or valueReference attribute".into(),
                    );
                }
            }
        }

        Ok(dimensions)
    }

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

        let modelVariables: Vec<ModelVariable> = root
            .get_required_child("ModelVariables")?
            .children()
            .filter(|n| n.is_element())
            .map(|child| {
                let variable_type = Self::get_variable_type(&child)?;
                let causality = child.attribute_as("causality")?.unwrap_or(Causality::Local);
                let variability = child.attribute_as("variability")?.unwrap_or({
                    if matches!(
                        variable_type,
                        VariableType::Float32 { .. } | VariableType::Float64 { .. }
                    ) && matches!(
                        causality,
                        Causality::Input
                            | Causality::Output
                            | Causality::Independent
                            | Causality::Local
                    ) {
                        Variability::Continuous
                    } else {
                        Variability::Discrete
                    }
                });

                Ok(ModelVariable {
                    variableType: variable_type,
                    name: child.required_attribute("name")?,
                    valueReference: child.required_attribute("valueReference")?.parse()?,
                    description: child.attribute_as("description")?,
                    causality,
                    variability,
                    dimensions: Self::get_dimensions(&child)?,
                    textPos: child.text_pos(),
                })
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

        let defaultExperiment = root
            .get_child("DefaultExperiment")
            .map(|n| DefaultExperiment::from_node(&n))
            .transpose()?;

        let modelExchange = root
            .get_child("ModelExchange")
            .map(|n| ModelExchange::from_node(&n))
            .transpose()?;

        let coSimulation = root
            .get_child("CoSimulation")
            .map(|n| CoSimulation::from_node(&n))
            .transpose()?;

        let scheduledExecution = root
            .get_child("ScheduledExecution")
            .map(|n| ScheduledExecution::from_node(&n))
            .transpose()?;

        let unitDefinitions = root
            .get_child("UnitDefinitions")
            .map(|n| n.get_children("Unit"))
            .into_iter()
            .flatten()
            .map(|n| Unit::from_node(&n))
            .collect::<Result<Vec<_>, _>>()?;

        let typeDefinitions = root
            .get_child("TypeDefinitions")
            .map(|n| {
                n.children()
                    .filter(|c| c.tag_name().name().ends_with("Type"))
            })
            .into_iter()
            .flatten()
            .map(|n| TypeDefinition::from_node(&n))
            .collect::<Result<Vec<_>, _>>()?;

        let outputs = Self::get_unknowns(root, "Output")?;
        let derivatives = Self::get_unknowns(root, "ContinuousStateDerivative")?;
        let clockedStates = Self::get_unknowns(root, "ClockedState")?;
        let initialUnknowns = Self::get_unknowns(root, "InitialUnknown")?;
        let eventIndicators = Self::get_unknowns(root, "EventIndicator")?;

        let model_description = ModelDescription {
            fmiVersion: root.required_attribute("fmiVersion")?,
            modelName: root.required_attribute("modelName")?,
            instantiationToken: root.required_attribute("instantiationToken")?,
            description: root.attribute_as("description")?,
            author: root.attribute_as("author")?,
            version: root.attribute_as("version")?,
            copyright: root.attribute_as("copyright")?,
            license: root.attribute_as("license")?,
            generationTool: root.attribute_as("generationTool")?,
            generationDateAndTime: root.attribute_as("generationDateAndTime")?,
            variableNamingConvention: root
                .attribute_as("variableNamingConvention")?
                .unwrap_or(VariableNamingConvention::Flat),
            defaultExperiment,
            modelExchange,
            coSimulation,
            scheduledExecution,
            unitDefinitions,
            typeDefinitions,
            modelVariables,
            outputs,
            derivatives,
            clockedStates,
            eventIndicators,
            initialUnknowns,
        };

        Ok(model_description)
    }
}

impl TypeDefinition {
    fn from_node(node: &Node) -> Result<Self, Box<dyn Error>> {
        let name = node.required_attribute("name")?;
        let description = node.attribute_as("description")?;

        if node.has_tag_name("Float32Type") {
            Ok(TypeDefinition::Float32 {
                name,
                description,
                quantity: node.attribute_as("quantity")?,
                unit: node.attribute_as("unit")?,
                displayUnit: node.attribute_as("displayUnit")?,
                relativeQuantity: node.attribute_as("relativeQuantity")?.unwrap_or_default(),
                unbounded: node.attribute_as("unbounded")?.unwrap_or_default(),
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
                nominal: node.attribute_as("nominal")?,
                textPos: node.text_pos(),
            })
        } else if node.has_tag_name("Float64Type") {
            Ok(TypeDefinition::Float64 {
                name,
                description,
                quantity: node.attribute_as("quantity")?,
                unit: node.attribute_as("unit")?,
                displayUnit: node.attribute_as("displayUnit")?,
                relativeQuantity: node.attribute_as("relativeQuantity")?.unwrap_or_default(),
                unbounded: node.attribute_as("unbounded")?.unwrap_or_default(),
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
                nominal: node.attribute_as("nominal")?,
                textPos: node.text_pos(),
            })
        } else if node.has_tag_name("Int8Type") {
            Ok(TypeDefinition::Int8 {
                name,
                description,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
                textPos: node.text_pos(),
            })
        } else if node.has_tag_name("UInt8Type") {
            Ok(TypeDefinition::UInt8 {
                name,
                description,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
                textPos: node.text_pos(),
            })
        } else if node.has_tag_name("Int16Type") {
            Ok(TypeDefinition::Int16 {
                name,
                description,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
                textPos: node.text_pos(),
            })
        } else if node.has_tag_name("UInt16Type") {
            Ok(TypeDefinition::UInt16 {
                name,
                description,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
                textPos: node.text_pos(),
            })
        } else if node.has_tag_name("Int32Type") {
            Ok(TypeDefinition::Int32 {
                name,
                description,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
                textPos: node.text_pos(),
            })
        } else if node.has_tag_name("UInt32Type") {
            Ok(TypeDefinition::UInt32 {
                name,
                description,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
                textPos: node.text_pos(),
            })
        } else if node.has_tag_name("Int64Type") {
            Ok(TypeDefinition::Int64 {
                name,
                description,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
                textPos: node.text_pos(),
            })
        } else if node.has_tag_name("UInt64Type") {
            Ok(TypeDefinition::UInt64 {
                name,
                description,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
                textPos: node.text_pos(),
            })
        } else if node.has_tag_name("BooleanType") {
            Ok(TypeDefinition::Boolean { name, description, textPos: node.text_pos() })
        } else if node.has_tag_name("StringType") {
            Ok(TypeDefinition::String { name, description, textPos: node.text_pos() })
        } else if node.has_tag_name("BinaryType") {
            Ok(TypeDefinition::Binary {
                name,
                description,
                mimeType: node
                    .attribute("mimeType")
                    .unwrap_or("application/octet-stream")
                    .to_string(),
                maxSize: node.attribute_as("maxSize")?,
                textPos: node.text_pos(),
            })
        } else if node.has_tag_name("EnumerationType") {
            let mut items = vec![];
            for child in node.children() {
                if child.has_tag_name("Item") {
                    items.push(Item::from_node(&child)?);
                }
            }
            Ok(TypeDefinition::Enumeration {
                name,
                description,
                items,
                quantity: node.attribute_as("quantity")?,
                min: node.attribute_as("min")?,
                max: node.attribute_as("max")?,
                textPos: node.text_pos(),
            })
        } else {
            Err(format!("Unknown type definition: {}", node.tag_name().name()).into())
        }
    }
}

impl DefaultExperiment {
    fn from_node(node: &roxmltree::Node) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(DefaultExperiment {
            startTime: node.attribute_as("startTime")?,
            stopTime: node.attribute_as("stopTime")?,
            tolerance: node.attribute_as("tolerance")?,
            stepSize: node.attribute_as("stepSize")?,
            textPos: node.text_pos(),
        })
    }
}

impl ModelExchange {
    fn from_node(node: &roxmltree::Node) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(ModelExchange {
            modelIdentifier: node.required_attribute_as("modelIdentifier")?,
            needsExecutionTool: node.attribute_as("needsExecutionTool")?.unwrap_or_default(),
            canBeInstantiatedOnlyOncePerProcess: node
                .attribute_as("canBeInstantiatedOnlyOncePerProcess")?
                .unwrap_or_default(),
            canGetAndSetFMUState: node
                .attribute_as("canGetAndSetFMUState")?
                .unwrap_or_default(),
            canSerializeFMUState: node
                .attribute_as("canSerializeFMUState")?
                .unwrap_or_default(),
            providesDirectionalDerivatives: node
                .attribute_as("providesDirectionalDerivatives")?
                .unwrap_or_default(),
            providesAdjointDerivatives: node
                .attribute_as("providesAdjointDerivatives")?
                .unwrap_or_default(),
            providesPerElementDependencies: node
                .attribute_as("providesPerElementDependencies")?
                .unwrap_or_default(),
            needsCompletedIntegratorStep: node
                .attribute_as("needsCompletedIntegratorStep")?
                .unwrap_or_default(),
            providesEvaluateDiscreteStates: node
                .attribute_as("providesEvaluateDiscreteStates")?
                .unwrap_or_default(),
            textPos: node.text_pos(),
        })
    }
}

impl CoSimulation {
    fn from_node(node: &roxmltree::Node) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(CoSimulation {
            modelIdentifier: node.required_attribute_as("modelIdentifier")?,
            needsExecutionTool: node.attribute_as("needsExecutionTool")?.unwrap_or_default(),
            canBeInstantiatedOnlyOncePerProcess: node
                .attribute_as("canBeInstantiatedOnlyOncePerProcess")?
                .unwrap_or_default(),
            canGetAndSetFMUState: node
                .attribute_as("canGetAndSetFMUState")?
                .unwrap_or_default(),
            canSerializeFMUState: node
                .attribute_as("canSerializeFMUState")?
                .unwrap_or_default(),
            providesDirectionalDerivatives: node
                .attribute_as("providesDirectionalDerivatives")?
                .unwrap_or_default(),
            providesAdjointDerivatives: node
                .attribute_as("providesAdjointDerivatives")?
                .unwrap_or_default(),
            providesPerElementDependencies: node
                .attribute_as("providesPerElementDependencies")?
                .unwrap_or_default(),
            canHandleVariableCommunicationStepSize: node
                .attribute_as("canHandleVariableCommunicationStepSize")?
                .unwrap_or_default(),
            fixedInternalStepSize: node.attribute_as("fixedInternalStepSize")?,
            maxOutputDerivativeOrder: node
                .attribute_as("maxOutputDerivativeOrder")?
                .unwrap_or_default(),
            recommendedIntermediateInputSmoothness: node
                .attribute_as("recommendedIntermediateInputSmoothness")?
                .unwrap_or_default(),
            providesIntermediateUpdate: node
                .attribute_as("providesIntermediateUpdate")?
                .unwrap_or_default(),
            mightReturnEarlyFromDoStep: node
                .attribute_as("mightReturnEarlyFromDoStep")?
                .unwrap_or_default(),
            canReturnEarlyAfterIntermediateUpdate: node
                .attribute_as("canReturnEarlyAfterIntermediateUpdate")?
                .unwrap_or_default(),
            hasEventMode: node.attribute_as("hasEventMode")?.unwrap_or_default(),
            providesEvaluateDiscreteStates: node
                .attribute_as("providesEvaluateDiscreteStates")?
                .unwrap_or_default(),
            textPos: node.text_pos(),
        })
    }
}

impl ScheduledExecution {
    fn from_node(node: &roxmltree::Node) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(ScheduledExecution {
            modelIdentifier: node.required_attribute_as("modelIdentifier")?,
            needsExecutionTool: node.attribute_as("needsExecutionTool")?.unwrap_or_default(),
            canBeInstantiatedOnlyOncePerProcess: node
                .attribute_as("canBeInstantiatedOnlyOncePerProcess")?
                .unwrap_or_default(),
            canGetAndSetFMUState: node
                .attribute_as("canGetAndSetFMUState")?
                .unwrap_or_default(),
            canSerializeFMUState: node
                .attribute_as("canSerializeFMUState")?
                .unwrap_or_default(),
            providesDirectionalDerivatives: node
                .attribute_as("providesDirectionalDerivatives")?
                .unwrap_or_default(),
            providesAdjointDerivatives: node
                .attribute_as("providesAdjointDerivatives")?
                .unwrap_or_default(),
            providesPerElementDependencies: node
                .attribute_as("providesPerElementDependencies")?
                .unwrap_or_default(),
            textPos: node.text_pos(),
        })
    }
}

impl Item {
    pub(crate) fn from_node(node: &roxmltree::Node) -> Result<Self, Box<dyn Error>> {
        Ok(Item {
            name: node.required_attribute("name")?,
            description: node.attribute_as("description")?,
            value: node.required_attribute("value")?.parse()?,
            textPos: node.text_pos(),
        })
    }
}
