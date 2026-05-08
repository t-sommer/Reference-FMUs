#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use roxmltree::Node;
use std::{collections::HashMap, error::Error, path::Path, str::FromStr};

use crate::types::fmiValueReference;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum VariableNamingConvention {
    Flat,
    Structured,
}

impl FromStr for VariableNamingConvention {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "flat" => Ok(VariableNamingConvention::Flat),
            "structured" => Ok(VariableNamingConvention::Structured),
            _ => Err(format!("Unknown variable naming convention: {}", s)),
        }
    }
}

#[derive(Debug)]
pub enum VariableType {
    Float32 {
        // fmi3Float32
        start: Option<f32>,
        // fmi3ArrayableVariable
        // dimensions: Vec<Dimension>,
        intermediateUpdate: bool,
        previous: Option<u32>,
        // fmi3TypedArrayableVariable
        declaredType: Option<String>,
        // fmi3InitializableVariable
        initial: Option<Initial>,
        // fmi3RealBaseAttributes
        quantity: Option<String>,
        unit: Option<String>,
        displayUnit: Option<String>,
        relativeQuantity: bool,
        unbounded: bool,
        // fmi3Float64Attributes
        min: Option<f32>,
        max: Option<f32>,
        nominal: Option<f32>,
        // fmi3RealVariableAttributes
        derivative: Option<fmiValueReference>,
        reinit: bool,
    },
    Float64 {
        // fmi3Float64
        start: Option<f64>,
        // fmi3ArrayableVariable
        // dimensions: Vec<Dimension>,
        intermediateUpdate: bool,
        previous: Option<u32>,
        // fmi3TypedArrayableVariable
        declaredType: Option<String>,
        // fmi3InitializableVariable
        initial: Option<Initial>,
        // fmi3RealBaseAttributes
        quantity: Option<String>,
        unit: Option<String>,
        displayUnit: Option<String>,
        relativeQuantity: bool,
        unbounded: bool,
        // fmi3Float64Attributes
        min: Option<f64>,
        max: Option<f64>,
        nominal: Option<f64>,
        // fmi3RealVariableAttributes
        derivative: Option<fmiValueReference>,
        reinit: bool,
    },
    Int8 {
        start: Option<i8>,
        initial: Option<Initial>,
        declaredType: Option<String>,
        intermediateUpdate: bool,
        previous: Option<u32>,
        quantity: Option<String>,
        min: Option<i8>,
        max: Option<i8>,
    },
    UInt8 {
        start: Option<u8>,
        initial: Option<Initial>,
        declaredType: Option<String>,
        intermediateUpdate: bool,
        previous: Option<u32>,
        quantity: Option<String>,
        min: Option<u8>,
        max: Option<u8>,
    },
    Int16 {
        start: Option<i16>,
        initial: Option<Initial>,
        declaredType: Option<String>,
        intermediateUpdate: bool,
        previous: Option<u32>,
        quantity: Option<String>,
        min: Option<i16>,
        max: Option<i16>,
    },
    UInt16 {
        start: Option<u16>,
        initial: Option<Initial>,
        declaredType: Option<String>,
        intermediateUpdate: bool,
        previous: Option<u32>,
        quantity: Option<String>,
        min: Option<u16>,
        max: Option<u16>,
    },
    Int32 {
        // fmi3Int8
        start: Option<i32>,
        // fmi3InitializableVariable
        initial: Option<Initial>,
        // fmi3TypedArrayableVariable
        declaredType: Option<String>,
        // fmi3ArrayableVariable
        // dimensions: Vec<Dimension>,
        intermediateUpdate: bool,
        previous: Option<u32>,
        // fmi3IntegerBaseAttributes
        quantity: Option<String>,
        // fmi3Int8Attributes
        min: Option<i32>,
        max: Option<i32>,
    },
    UInt32 {
        start: Option<u32>,
        initial: Option<Initial>,
        declaredType: Option<String>,
        intermediateUpdate: bool,
        previous: Option<u32>,
        quantity: Option<String>,
        min: Option<u32>,
        max: Option<u32>,
    },
    Int64 {
        start: Option<i64>,
        initial: Option<Initial>,
        declaredType: Option<String>,
        intermediateUpdate: bool,
        previous: Option<u32>,
        quantity: Option<String>,
        min: Option<i64>,
        max: Option<i64>,
    },
    UInt64 {
        start: Option<u64>,
        initial: Option<Initial>,
        declaredType: Option<String>,
        intermediateUpdate: bool,
        previous: Option<u32>,
        quantity: Option<String>,
        min: Option<u64>,
        max: Option<u64>,
    },
    Boolean {
        start: Option<bool>,
        initial: Option<Initial>,
        declaredType: Option<String>,
        intermediateUpdate: bool,
        previous: Option<u32>,
    },
    String {
        start: Option<String>,
        initial: Option<Initial>,
        declaredType: Option<String>,
        intermediateUpdate: bool,
        previous: Option<u32>,
    },
    Binary {
        start: Option<Vec<u8>>,
        initial: Option<Initial>,
        declaredType: Option<String>,
        intermediateUpdate: bool,
        previous: Option<u32>,
    },
    Clock {
        start: Option<bool>,
        initial: Option<Initial>,
        declaredType: Option<String>,
        intermediateUpdate: bool,
        previous: Option<u32>,
    },
    Enumeration {
        start: Option<i64>,
        initial: Option<Initial>,
        declaredType: String,
        intermediateUpdate: bool,
        previous: Option<u32>,
    },
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Causality {
    Parameter,
    CalculatedParameter,
    StructuralParameter,
    Input,
    Output,
    Local,
    Independent,
}

impl FromStr for Causality {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parameter" => Ok(Causality::Parameter),
            "calculatedParameter" => Ok(Causality::CalculatedParameter),
            "structuralParameter" => Ok(Causality::StructuralParameter),
            "input" => Ok(Causality::Input),
            "output" => Ok(Causality::Output),
            "local" => Ok(Causality::Local),
            "independent" => Ok(Causality::Independent),
            _ => Err(format!("Unknown causality: {}", s)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Variability {
    Constant,
    Fixed,
    Tunable,
    Discrete,
    Continuous,
}

impl FromStr for Variability {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "constant" => Ok(Variability::Constant),
            "fixed" => Ok(Variability::Fixed),
            "tunable" => Ok(Variability::Tunable),
            "discrete" => Ok(Variability::Discrete),
            "continuous" => Ok(Variability::Continuous),
            _ => Err(format!("Unknown variability: {}", s)),
        }
    }
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
pub enum Dimension {
    Fixed { start: usize },
    Variable { valueReference: fmiValueReference },
}

#[derive(Debug)]
pub struct ModelVariable {
    pub variableType: VariableType,
    pub name: String,
    pub valueReference: fmiValueReference,
    pub description: Option<String>,
    pub causality: Causality,
    pub variability: Variability,
    pub dimensions: Vec<Dimension>,
}

#[derive(Debug)]
pub struct Unknown {
    pub valueReference: fmiValueReference,
    pub dependencies: Option<Vec<fmiValueReference>>,
    pub dependenciesKind: Option<Vec<fmiValueReference>>,
}

#[derive(Debug)]
pub struct ModelDescription {
    pub fmiVersion: String,
    pub modelName: String,
    pub instantiationToken: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub copyright: Option<String>,
    pub license: Option<String>,
    pub generationTool: Option<String>,
    pub generationDateAndTime: Option<String>,
    pub variableNamingConvention: VariableNamingConvention,
    pub defaultExperiment: Option<DefaultExperiment>,
    pub modelExchange: Option<ModelExchange>,
    pub coSimulation: Option<CoSimulation>,
    pub modelVariables: Vec<ModelVariable>,
    pub outputs: Vec<Unknown>,
    pub derivatives: Vec<Unknown>,
    pub clockedStates: Vec<Unknown>,
    pub eventIndicators: Vec<Unknown>,
    pub initialUnknowns: Vec<Unknown>,
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
            relativeQuantity: node.attribute("relativeQuantity").map(|s| s == "true").unwrap_or(false),
            unbounded: node.attribute("unbounded").map(|s| s == "true").unwrap_or(false),
            min: node.optional_attribute_as("min"),
            max: node.optional_attribute_as("max"),
            nominal: node.optional_attribute_as("nominal"),
            start: node.optional_attribute_as("start"),
            derivative: node.optional_attribute_as("derivative"),
            reinit: node.attribute("reinit").map(|s| s == "true").unwrap_or(false),
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
            relativeQuantity: node.attribute("relativeQuantity").map(|s| s == "true").unwrap_or(false),
            unbounded: node.attribute("unbounded").map(|s| s == "true").unwrap_or(false),
            min: node.optional_attribute_as("min"),
            max: node.optional_attribute_as("max"),
            nominal: node.optional_attribute_as("nominal"),
            start: node.optional_attribute_as("start"),
            derivative: node.optional_attribute_as("derivative"),
            reinit: node.attribute("reinit").map(|s| s == "true").unwrap_or(false),
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

    read_fmi3_model_description(&root)
}

fn read_fmi3_model_description(root: &Node) -> Result<ModelDescription, Box<dyn Error>> {
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

        let variableType = get_variable_type(&child)?;

        let causality = child.attribute("causality")
            .map(|s| Causality::from_str(s).unwrap())
            .unwrap_or(Causality::Local);

        let variability = child.attribute("variability")
            .map(|s| Variability::from_str(s).unwrap())
            .unwrap_or_else(|| {
                if matches!(variableType, VariableType::Float32 { .. } | VariableType::Float64 { .. })
                    && matches!(
                        causality,
                        Causality::Input | Causality::Output | Causality::Independent | Causality::Local
                    )
                {
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
            dimensions: get_dimensions(&child)?,
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

    let outputs = get_fmi3_unkonwns(root, "Output")?;
    let derivatives = get_fmi3_unkonwns(root, "ContinuousStateDerivative")?;
    let clockedStates = get_fmi3_unkonwns(root, "ClockedState")?;
    let initialUnknowns = get_fmi3_unkonwns(root, "InitialUnknown")?;
    let eventIndicators = get_fmi3_unkonwns(root, "EventIndicator")?;

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
        variableNamingConvention: root.optional_attribute("variableNamingConvention").unwrap_or("flat".to_string()).parse()?,
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
