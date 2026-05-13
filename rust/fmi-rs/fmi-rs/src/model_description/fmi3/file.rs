use std::{collections::HashMap, error::Error, path::Path, str::FromStr};

use roxmltree::Node;

use crate::model_description::Unit;
use crate::model_description::file::StringAttribute;

use crate::model_description::fmi3::{
    Causality, CoSimulation, DefaultExperiment, DependencyKind, Dimension, IntervalVariability, Item, ModelDescription, ModelExchange, ModelVariable, TypeDefinition, Unknown, Variability, VariableType
};

impl ModelDescription {
    fn get_unknowns(root: &Node, name: &str) -> Result<Vec<Unknown>, Box<dyn Error>> {
        let modelStructure = root
            .descendants()
            .find(|n| n.has_tag_name("ModelStructure"))
            .ok_or("Missing <ModelStructure> element.")?;

        let mut unkonwns = vec![];

        for child in modelStructure.children().filter(|n| n.has_tag_name(name)) {
            let valueReference = child
                .attribute("valueReference")
                .ok_or(format!("Missing valueReference attribute in {}", name))?
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
                                    .map(|s| DependencyKind::from_str(s).unwrap())
                                    .collect(),
                            )
                        }
                    }
                    None => None,
                };

            unkonwns.push(Unknown {
                valueReference,
                dependencies,
                dependenciesKind,
            });
        }

        Ok(unkonwns)
    }

    fn get_variable_type(node: &Node) -> Result<VariableType, Box<dyn Error>> {
        if node.has_tag_name("Float32") {
            return Ok(VariableType::Float32 {
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
                declaredType: node.optional_attribute("declaredType"),
                initial: node.optional_attribute_as("initial"),
                quantity: node.optional_attribute("quantity"),
                unit: node.optional_attribute("unit"),
                displayUnit: node.optional_attribute("displayUnit"),
                relativeQuantity: node
                    .attribute("relativeQuantity")
                    .map(|s| s == "true")
                    .unwrap_or(false),
                unbounded: node
                    .attribute("unbounded")
                    .map(|s| s == "true")
                    .unwrap_or(false),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
                nominal: node.optional_attribute_as("nominal"),
                start: node.optional_attribute_as("start"),
                derivative: node.optional_attribute_as("derivative"),
                reinit: node
                    .attribute("reinit")
                    .map(|s| s == "true")
                    .unwrap_or(false),
            });
        } else if node.has_tag_name("Float64") {
            return Ok(VariableType::Float64 {
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
                declaredType: node.optional_attribute("declaredType"),
                initial: node.optional_attribute_as("initial"),
                quantity: node.optional_attribute("quantity"),
                unit: node.optional_attribute("unit"),
                displayUnit: node.optional_attribute("displayUnit"),
                relativeQuantity: node
                    .attribute("relativeQuantity")
                    .map(|s| s == "true")
                    .unwrap_or(false),
                unbounded: node
                    .attribute("unbounded")
                    .map(|s| s == "true")
                    .unwrap_or(false),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
                nominal: node.optional_attribute_as("nominal"),
                start: node.optional_attribute_as("start"),
                derivative: node.optional_attribute_as("derivative"),
                reinit: node
                    .attribute("reinit")
                    .map(|s| s == "true")
                    .unwrap_or(false),
            });
        } else if node.has_tag_name("Int8") {
            return Ok(VariableType::Int8 {
                start: node.optional_attribute_as("start"),
                initial: node.optional_attribute_as("initial"),
                declaredType: node.optional_attribute("declaredType"),
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("UInt8") {
            return Ok(VariableType::UInt8 {
                start: node.optional_attribute_as("start"),
                initial: node.optional_attribute_as("initial"),
                declaredType: node.optional_attribute("declaredType"),
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("Int16") {
            return Ok(VariableType::Int16 {
                start: node.optional_attribute_as("start"),
                initial: node.optional_attribute_as("initial"),
                declaredType: node.optional_attribute("declaredType"),
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("UInt16") {
            return Ok(VariableType::UInt16 {
                start: node.optional_attribute_as("start"),
                initial: node.optional_attribute_as("initial"),
                declaredType: node.optional_attribute("declaredType"),
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("Int32") {
            return Ok(VariableType::Int32 {
                start: node.optional_attribute_as("start"),
                initial: node.optional_attribute_as("initial"),
                declaredType: node.optional_attribute("declaredType"),
                // dimensions: get_dimensions(node)?,
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("UInt32") {
            return Ok(VariableType::UInt32 {
                start: node.optional_attribute_as("start"),
                initial: node.optional_attribute_as("initial"),
                declaredType: node.optional_attribute("declaredType"),
                // dimensions: get_dimensions(node)?,
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("Int64") {
            return Ok(VariableType::Int64 {
                start: node.optional_attribute_as("start"),
                initial: node.optional_attribute_as("initial"),
                declaredType: node.optional_attribute("declaredType"),
                // dimensions: get_dimensions(node)?,
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("UInt64") {
            return Ok(VariableType::UInt64 {
                start: node.optional_attribute_as("start"),
                initial: node.optional_attribute_as("initial"),
                declaredType: node.optional_attribute("declaredType"),
                // dimensions: get_dimensions(node)?,
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("Boolean") {
            return Ok(VariableType::Boolean {
                start: node.optional_attribute_as("start"),
                initial: node.optional_attribute_as("initial"),
                declaredType: node.optional_attribute("declaredType"),
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
            });
        } else if node.has_tag_name("String") {
            return Ok(VariableType::String {
                start: node.optional_attribute_as("start"),
                initial: node.optional_attribute_as("initial"),
                declaredType: node.optional_attribute("declaredType"),
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
            });
        } else if node.has_tag_name("Binary") {
            return Ok(VariableType::Binary {
                start: None, // TODO: node.optional_attribute_as("start"),
                initial: node.optional_attribute_as("initial"),
                declaredType: node.optional_attribute("declaredType"),
                // dimensions: get_dimensions(node)?,
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
            });
        } else if node.has_tag_name("Clock") {
            return Ok(VariableType::Clock {
                start: node.optional_attribute_as("start"),
                initial: node.optional_attribute_as("initial"),
                declaredType: node.optional_attribute("declaredType"),
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
                canBeDeactivated: node.bool_attribute("canBeDeactivated", false),
                priority: node.optional_attribute_as("priority"),
                intervalVariability: IntervalVariability::from_str(
                    node.required_attribute("intervalVariability")?.as_str(),
                )?,
                intervalDecimal: node.optional_attribute_as("intervalDecimal"),
                shiftDecimal: node.required_attribute("shiftDecimal")?.parse()?,
                supportsFraction: node.bool_attribute("supportsFraction", false),
                resolution: node.optional_attribute_as("resolution"),
                intervalCounter: node.optional_attribute_as("intervalCounter"),
                shiftCounter: node.required_attribute("shiftDecimal")?.parse()?,
            });
        } else if node.has_tag_name("Enumeration") {
            return Ok(VariableType::Enumeration {
                start: node.optional_attribute_as("start"),
                initial: node.optional_attribute_as("initial"),
                declaredType: node.required_attribute("declaredType")?,
                intermediateUpdate: node.bool_attribute("intermediateUpdate", false),
                previous: node.optional_attribute_as("previous"),
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

        let doc = roxmltree::Document::parse_with_options(&text, opt).unwrap();

        let root = &doc.root_element();

        let model_variables_node = root
            .descendants()
            .find(|n| n.has_tag_name("ModelVariables"))
            .unwrap();

        let modelVariables: Vec<ModelVariable> = model_variables_node
            .children()
            .filter(|n| n.is_element())
            .map(|child| {
                let variable_type = Self::get_variable_type(&child)?;
                let causality = child
                    .optional_attribute_as("causality")
                    .unwrap_or(Causality::Local);
                let variability = child
                    .optional_attribute_as("variability")
                    .unwrap_or_else(|| {
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
                    description: child.optional_attribute("description"),
                    causality,
                    variability,
                    dimensions: Self::get_dimensions(&child)?,
                })
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

        let defaultExperiment = root
            .descendants()
            .find(|n| n.has_tag_name("DefaultExperiment"))
            .map(|e| DefaultExperiment {
                startTime: e.attribute("startTime").map(|s| s.to_string()),
                stopTime: e.attribute("stopTime").map(|s| s.to_string()),
                tolerance: e.attribute("tolerance").map(|s| s.to_string()),
                stepSize: e.attribute("stepSize").map(|s| s.to_string()),
            });

        let coSimulation = if let Some(cs) =
            root.descendants().find(|n| n.has_tag_name("CoSimulation"))
        {
            Some(CoSimulation {
                modelIdentifier: cs.attribute("modelIdentifier").unwrap().to_string(),
                providesDirectionalDerivatives: cs
                    .bool_attribute("providesDirectionalDerivatives", false),
                fixedInternalStepSize: cs.attribute("fixedInternalStepSize").map(|s| s.to_string()),
                canHandleVariableCommunicationStepSize: cs
                    .bool_attribute("canHandleVariableCommunicationStepSize", false),
                canNotUseMemoryManagementFunctions: true,
            })
        } else {
            None
        };

        let modelExchange =
            if let Some(me) = root.descendants().find(|n| n.has_tag_name("ModelExchange")) {
                Some(ModelExchange {
                    modelIdentifier: me.attribute("modelIdentifier").unwrap().to_string(),
                    providesDirectionalDerivatives: me
                        .bool_attribute("providesDirectionalDerivatives", false),
                    needsCompletedIntegratorStep: me
                        .bool_attribute("needsCompletedIntegratorStep", false),
                    canNotUseMemoryManagementFunctions: true,
                })
            } else {
                None
            };

        let unitDefintions = root
            .descendants()
            .find(|n| n.has_tag_name("UnitDefinitions"))
            .map(|n| n.descendants())
            .into_iter()
            .flatten()
            .filter(|n| n.has_tag_name("Unit"))
            .into_iter()
            .map(|n| Unit::from_node(&n).unwrap())
            .collect();

        let typeDefinitions = root
            .descendants()
            .find(|n| n.has_tag_name("TypeDefinitions"))
            .map(|n| n.descendants())
            .into_iter()
            .flatten()
            .filter(|n| n.tag_name().name().ends_with("Type"))
            .into_iter()
            .map(|n| TypeDefinition::from_node(&n).unwrap())
            .collect();

        let outputs = Self::get_unknowns(root, "Output")?;
        let derivatives = Self::get_unknowns(root, "ContinuousStateDerivative")?;
        let clockedStates = Self::get_unknowns(root, "ClockedState")?;
        let initialUnknowns = Self::get_unknowns(root, "InitialUnknown")?;
        let eventIndicators = Self::get_unknowns(root, "EventIndicator")?;

        let model_description = ModelDescription {
            fmiVersion: root.required_attribute("fmiVersion")?,
            modelName: root.required_attribute("modelName")?,
            instantiationToken: root.required_attribute("instantiationToken")?,
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
            modelExchange,
            coSimulation,
            unitDefintions,
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
        let description = node.optional_attribute("description");

        if node.has_tag_name("Float32Type") {
            return Ok(TypeDefinition::Float32 {
                name,
                description,
                quantity: node.optional_attribute("quantity"),
                unit: node.optional_attribute("unit"),
                displayUnit: node.optional_attribute("displayUnit"),
                relativeQuantity: node.bool_attribute("relativeQuantity", false),
                unbounded: node.bool_attribute("unbounded", false),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
                nominal: node.optional_attribute_as("nominal"),
            });
        } else if node.has_tag_name("Float64Type") {
            return Ok(TypeDefinition::Float64 {
                name,
                description,
                quantity: node.optional_attribute("quantity"),
                unit: node.optional_attribute("unit"),
                displayUnit: node.optional_attribute("displayUnit"),
                relativeQuantity: node.bool_attribute("relativeQuantity", false),
                unbounded: node.bool_attribute("unbounded", false),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
                nominal: node.optional_attribute_as("nominal"),
            });
        } else if node.has_tag_name("Int8Type") {
            return Ok(TypeDefinition::Int8 {
                name,
                description,
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("UInt8Type") {
            return Ok(TypeDefinition::UInt8 {
                name,
                description,
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("Int16Type") {
            return Ok(TypeDefinition::Int16 {
                name,
                description,
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("UInt16Type") {
            return Ok(TypeDefinition::UInt16 {
                name,
                description,
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("Int32Type") {
            return Ok(TypeDefinition::Int32 {
                name,
                description,
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("UInt32Type") {
            return Ok(TypeDefinition::UInt32 {
                name,
                description,
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("Int64Type") {
            return Ok(TypeDefinition::Int64 {
                name,
                description,
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("UInt64Type") {
            return Ok(TypeDefinition::UInt64 {
                name,
                description,
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else if node.has_tag_name("BooleanType") {
            return Ok(TypeDefinition::Boolean {
                name,
                description,
            });
        } else if node.has_tag_name("StringType") {
            return Ok(TypeDefinition::String {
                name,
                description,
            });
        } else if node.has_tag_name("BinaryType") {
            return Ok(TypeDefinition::Binary {
                name,
                description,
                mimeType: node.attribute("mimeType").unwrap_or("application/octet-stream").to_string(),
                maxSize: node.optional_attribute_as("maxSize"),
            });
        } else if node.has_tag_name("EnumerationType") {

            let mut items = vec![];
                for child in node.children() {
                    if child.has_tag_name("Item") {
                        items.push(Item::from_node(&child)?);
                    }
                }

            return Ok(TypeDefinition::Enumeration {
                name,
                description,
                items,
                quantity: node.optional_attribute("quantity"),
                min: node.optional_attribute_as("min"),
                max: node.optional_attribute_as("max"),
            });
        } else {
            return Err(format!("Unknown type definition: {}", node.tag_name().name()).into());
        }
    }
}

impl Item {
    pub(crate) fn from_node(node: &roxmltree::Node) -> Result<Self, Box<dyn Error>> {
        Ok(Item {
            name: node.required_attribute("name")?,
            description: node.optional_attribute("description"),
            value: node.required_attribute("value")?.parse()?,
        })
    }
}