#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
pub mod file;
pub mod validation;

use std::{ops::Range, str::FromStr};

use crate::{model_description::{TextPos, Unit}, types::fmiValueReference};

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
pub struct Item {
    pub name: String,
    pub value: i64,
    pub description: Option<String>,
    range: Range<usize>
,
}

#[derive(Debug)]
pub enum IntervalVariability {
    Constant,
    Fixed,
    Tunable,
    Changing,
    Countdown,
    Triggered,
}

impl FromStr for IntervalVariability {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "constant" => Ok(IntervalVariability::Constant),
            "fixed" => Ok(IntervalVariability::Fixed),
            "tunable" => Ok(IntervalVariability::Tunable),
            "changing" => Ok(IntervalVariability::Changing),
            "countdown" => Ok(IntervalVariability::Countdown),
            "triggered" => Ok(IntervalVariability::Triggered),
            _ => Err(format!("Unknown interval variability: {}", s)),
        }
    }
}

#[derive(Debug)]
pub enum TypeDefinition {
    Float32 {
        // fmi3TypeDefinitionBase
        name: String,
        description: Option<String>,
        // fmi3RealBaseAttributes
        quantity: Option<String>,
        unit: Option<String>,
        displayUnit: Option<String>,
        relativeQuantity: bool,
        unbounded: bool,
        // fmi3Float32Attributes
        min: Option<f32>,
        max: Option<f32>,
        nominal: Option<f32>,
        range: Range<usize>
,
    },
    Float64 {
        // fmi3TypeDefinitionBase
        name: String,
        description: Option<String>,
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
        range: Range<usize>
,
    },
    Int8 {
        name: String,
        description: Option<String>,
        quantity: Option<String>,
        min: Option<i8>,
        max: Option<i8>,
        range: Range<usize>
,
    },
    UInt8 {
        name: String,
        description: Option<String>,
        quantity: Option<String>,
        min: Option<u8>,
        max: Option<u8>,
        range: Range<usize>,
    },
    Int16 {
        name: String,
        description: Option<String>,
        quantity: Option<String>,
        min: Option<i16>,
        max: Option<i16>,
        range: Range<usize>,
    },
    UInt16 {
        name: String,
        description: Option<String>,
        quantity: Option<String>,
        min: Option<u16>,
        max: Option<u16>,
        range: Range<usize>,
    },
    Int32 {
        name: String,
        description: Option<String>,
        quantity: Option<String>,
        min: Option<i32>,
        max: Option<i32>,
        range: Range<usize>,
    },
    UInt32 {
        name: String,
        description: Option<String>,
        quantity: Option<String>,
        min: Option<u32>,
        max: Option<u32>,
        range: Range<usize>,
    },
    Int64 {
        name: String,
        description: Option<String>,
        quantity: Option<String>,
        min: Option<i64>,
        max: Option<i64>,
        range: Range<usize>,
    },
    UInt64 {
        name: String,
        description: Option<String>,
        quantity: Option<String>,
        min: Option<u64>,
        max: Option<u64>,
        range: Range<usize>,
    },
    Boolean {
        name: String,
        description: Option<String>,
        range: Range<usize>,
    },
    String {
        name: String,
        description: Option<String>,
        range: Range<usize>,
    },
    Binary {
        name: String,
        description: Option<String>,
        mimeType: String,
        maxSize: Option<u64>,
        range: Range<usize>
,
    },
    Enumeration {
        name: String,
        description: Option<String>,
        items: Vec<Item>,
        quantity: Option<String>,
        min: Option<i64>,
        max: Option<i64>,
        range: Range<usize>,
    },
    Clock {
        name: String,
        description: Option<String>,
        canBeDeactivated: bool,
        priority: Option<u32>,
        intervalVariability: IntervalVariability,
        intervalDecimal: Option<f64>,
        shiftDecimal: f64,
        supportsFraction: bool,
        resolution: Option<u64>,
        intervalCounter: Option<u64>,
        shiftCounter: u64,
        range: Range<usize>,
    },
}

impl TypeDefinition {
    pub fn name(&self) -> &str {
        match self {
            TypeDefinition::Float32 { name, .. }
            | TypeDefinition::Float64 { name, .. }
            | TypeDefinition::Int8 { name, .. }
            | TypeDefinition::UInt8 { name, .. }
            | TypeDefinition::Int16 { name, .. }
            | TypeDefinition::UInt16 { name, .. }
            | TypeDefinition::Int32 { name, .. }
            | TypeDefinition::UInt32 { name, .. }
            | TypeDefinition::Int64 { name, .. }
            | TypeDefinition::UInt64 { name, .. }
            | TypeDefinition::Boolean { name, .. }
            | TypeDefinition::String { name, .. }
            | TypeDefinition::Binary { name, .. }
            | TypeDefinition::Enumeration { name, .. }
            | TypeDefinition::Clock { name, .. } => name,
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
        canBeDeactivated: bool,
        priority: Option<u32>,
        intervalVariability: IntervalVariability,
        intervalDecimal: Option<f64>,
        shiftDecimal: f64,
        supportsFraction: bool,
        resolution: Option<u64>,
        intervalCounter: Option<u64>,
        shiftCounter: u64,
    },
    Enumeration {
        start: Option<i64>,
        initial: Option<Initial>,
        declaredType: String,
        intermediateUpdate: bool,
        previous: Option<u32>,
    },
}

impl VariableType {
    /// Returns the name of the variable type regardless of the variant.
    pub fn name(&self) -> &'static str {
        match self {
            VariableType::Float32 { .. } => "Float32",
            VariableType::Float64 { .. } => "Float64",
            VariableType::Int8 { .. } => "Int8",
            VariableType::UInt8 { .. } => "UInt8",
            VariableType::Int16 { .. } => "Int16",
            VariableType::UInt16 { .. } => "UInt16",
            VariableType::Int32 { .. } => "Int32",
            VariableType::UInt32 { .. } => "UInt32",
            VariableType::Int64 { .. } => "Int64",
            VariableType::UInt64 { .. } => "UInt64",
            VariableType::Boolean { .. } => "Boolean",
            VariableType::String { .. } => "String",
            VariableType::Binary { .. } => "Binary",
            VariableType::Clock { .. } => "Clock",
            VariableType::Enumeration { .. } => "Enumeration",
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DependencyKind {
    Dependent,
    Constant,
    Fixed,
    Tunable,
    Discrete,
}

impl FromStr for DependencyKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dependent" => Ok(DependencyKind::Dependent),
            "constant" => Ok(DependencyKind::Constant),
            "fixed" => Ok(DependencyKind::Fixed),
            "tunable" => Ok(DependencyKind::Tunable),
            "discrete" => Ok(DependencyKind::Discrete),
            _ => Err(format!("Unknown dependency kind: {}", s)),
        }
    }
}

#[derive(Debug)]
pub struct DefaultExperiment {
    pub startTime: Option<String>,
    pub stopTime: Option<String>,
    pub tolerance: Option<String>,
    pub stepSize: Option<String>,
    pub range: Range<usize>,
}

#[derive(Debug)]
pub struct ModelExchange {
    pub modelIdentifier: String,
    pub needsExecutionTool: bool,
    pub canBeInstantiatedOnlyOncePerProcess: bool,
    pub canGetAndSetFMUState: bool,
    pub canSerializeFMUState: bool,
    pub providesDirectionalDerivatives: bool,
    pub providesAdjointDerivatives: bool,
    pub providesPerElementDependencies: bool,
    pub needsCompletedIntegratorStep: bool,
    pub providesEvaluateDiscreteStates: bool,
    pub range: Range<usize>,
}

#[derive(Debug)]
pub struct CoSimulation {
    pub modelIdentifier: String,
    pub needsExecutionTool: bool,
    pub canBeInstantiatedOnlyOncePerProcess: bool,
    pub canGetAndSetFMUState: bool,
    pub canSerializeFMUState: bool,
    pub providesDirectionalDerivatives: bool,
    pub providesAdjointDerivatives: bool,
    pub providesPerElementDependencies: bool,
    pub canHandleVariableCommunicationStepSize: bool,
    pub fixedInternalStepSize: Option<String>,
    pub maxOutputDerivativeOrder: u32,
    pub recommendedIntermediateInputSmoothness: i32,
    pub providesIntermediateUpdate: bool,
    pub mightReturnEarlyFromDoStep: bool,
    pub canReturnEarlyAfterIntermediateUpdate: bool,
    pub hasEventMode: bool,
    pub providesEvaluateDiscreteStates: bool,
    pub range: Range<usize>,
}

#[derive(Debug)]
pub struct ScheduledExecution {
    pub modelIdentifier: String,
    pub needsExecutionTool: bool,
    pub canBeInstantiatedOnlyOncePerProcess: bool,
    pub canGetAndSetFMUState: bool,
    pub canSerializeFMUState: bool,
    pub providesDirectionalDerivatives: bool,
    pub providesAdjointDerivatives: bool,
    pub providesPerElementDependencies: bool,
    pub range: Range<usize>,
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
    pub range: Range<usize>,
}

#[derive(Debug)]
pub struct Unknown {
    pub valueReference: fmiValueReference,
    pub dependencies: Option<Vec<fmiValueReference>>,
    pub dependenciesKind: Option<Vec<DependencyKind>>,
    pub range: Range<usize>,
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
    pub scheduledExecution: Option<ScheduledExecution>,
    pub unitDefinitions: Vec<Unit>,
    pub typeDefinitions: Vec<TypeDefinition>,
    pub modelVariables: Vec<ModelVariable>,
    pub outputs: Vec<Unknown>,
    pub derivatives: Vec<Unknown>,
    pub clockedStates: Vec<Unknown>,
    pub eventIndicators: Vec<Unknown>,
    pub initialUnknowns: Vec<Unknown>,
}

impl ModelDescription {
    /// Returns the variable with the given value reference.
    pub fn get_variable(&self, vr: fmiValueReference) -> Option<&ModelVariable> {
        self.modelVariables.iter().find(|v| v.valueReference == vr)
    }

    pub fn get_unit<'a>(&'a self, variable: &'a ModelVariable) -> Option<&'a str> {
        if let VariableType::Float32 {
            unit, declaredType, ..
        }
        | VariableType::Float64 {
            unit, declaredType, ..
        } = &variable.variableType
        {
            if let Some(unit) = unit {
                return Some(unit);
            } else if let Some(declaredType) = declaredType {
                for type_definition in &self.typeDefinitions {
                    if let TypeDefinition::Float32 { name, unit, .. }
                    | TypeDefinition::Float64 { name, unit, .. } = type_definition
                        && name == declaredType
                    {
                        return unit.as_deref();
                    }
                }
            }
        }
        None
    }

    pub fn get_type_definition(&self, name: &str) -> Option<&TypeDefinition> {
        self.typeDefinitions.iter().find(|t| t.name() == name)
    }
}
