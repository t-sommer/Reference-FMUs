use std::{collections::HashMap, error::Error, path::Path, str::FromStr};

use roxmltree::Node;

use crate::model_description::fmi3::{Causality, CoSimulation, DefaultExperiment, Dimension, ModelDescription, ModelExchange, ModelVariable, Unknown, Variability, VariableType};

impl ModelDescription {

    fn get_fmi3_unkonwns(root: &Node, name: &str) -> Result<Vec<Unknown>, Box<dyn Error>> {
    let modelStructure = root
        .descendants()
        .find(|n| n.has_tag_name("ModelStructure"))
        .ok_or("Missing ModelStructure element.")?;

    let mut unkonwns = vec![];

    for child in modelStructure.children().filter(|n| n.has_tag_name(name)) {
        let valueReference = child
            .attribute("valueReference")
            .ok_or(format!("Missing valueReference attribute in {}", name))?
            .parse()
            .unwrap();

        unkonwns.push(Unknown {
            valueReference,
            dependencies: None,
            dependenciesKind: None,
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
                return Err("Dimension must have either size or valueReference attribute".into());
            }
        }
    }

    Ok(dimensions)
}

pub fn read_from_file(path: &Path) -> Result<ModelDescription, Box<dyn Error>> {
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

    let mut variables_for_vr = HashMap::new();
    let mut modelVariables = vec![];

    for (i, child) in ModelVariables
        .children()
        .filter(|n| n.is_element())
        .enumerate()
    {
        let name = child.attribute("name").unwrap();
        let valueReference = child.attribute("valueReference").unwrap().parse().unwrap();

        let variableType = Self::get_variable_type(&child)?;

        let causality = child
            .attribute("causality")
            .map(|s| Causality::from_str(s).unwrap())
            .unwrap_or(Causality::Local);

        let variability = child
            .attribute("variability")
            .map(|s| Variability::from_str(s).unwrap())
            .unwrap_or_else(|| {
                if matches!(
                    variableType,
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

        let variable = ModelVariable {
            variableType,
            name: name.to_string(),
            valueReference: valueReference,
            description: child.optional_attribute("description"),
            causality,
            variability,
            dimensions: Self::get_dimensions(&child)?,
        };

        variables_for_vr.insert(valueReference, i);
        modelVariables.push(variable);
    }

    let defaultExperiment = root
        .descendants()
        .find(|n| n.has_tag_name("DefaultExperiment"))
        .map(|e| DefaultExperiment {
            startTime: e.attribute("startTime").map(|s| s.to_string()),
            stopTime: e.attribute("stopTime").map(|s| s.to_string()),
            tolerance: e.attribute("tolerance").map(|s| s.to_string()),
            stepSize: e.attribute("stepSize").map(|s| s.to_string()),
        });

    let coSimulation = if let Some(cs) = root.descendants().find(|n| n.has_tag_name("CoSimulation"))
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

    let modelExchange = if let Some(me) =
        root.descendants().find(|n| n.has_tag_name("ModelExchange"))
    {
        Some(ModelExchange {
            modelIdentifier: me.attribute("modelIdentifier").unwrap().to_string(),
            providesDirectionalDerivatives: me
                .bool_attribute("providesDirectionalDerivatives", false),
            needsCompletedIntegratorStep: me.bool_attribute("needsCompletedIntegratorStep", false),
            canNotUseMemoryManagementFunctions: true,
        })
    } else {
        None
    };

    let outputs = Self::get_fmi3_unkonwns(root, "Output")?;
    let derivatives = Self::get_fmi3_unkonwns(root, "ContinuousStateDerivative")?;
    let clockedStates = Self::get_fmi3_unkonwns(root, "ClockedState")?;
    let initialUnknowns = Self::get_fmi3_unkonwns(root, "InitialUnknown")?;
    let eventIndicators = Self::get_fmi3_unkonwns(root, "EventIndicator")?;

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

trait StringAttribute {
    fn optional_attribute(&self, name: &str) -> Option<String>;
    fn required_attribute(&self, name: &str) -> Result<String, Box<dyn Error>>;
    fn bool_attribute(&self, name: &str, default: bool) -> bool;
    fn optional_attribute_as<T: FromStr>(&self, name: &str) -> Option<T>;
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

    fn optional_attribute_as<T: FromStr>(&self, name: &str) -> Option<T> {
        self.attribute(name).and_then(|v| v.parse().ok())
    }
}