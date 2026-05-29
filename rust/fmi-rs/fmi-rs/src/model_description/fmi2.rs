#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
pub mod file;
pub mod validation;

use std::str::FromStr;

use crate::{
    model_description::{TextPos, Unit},
    types::fmiValueReference,
};

pub type VariableIndex = u32;

#[derive(Debug, PartialEq)]
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

impl VariableType {
    /// Returns the name of the variable type regardless of the variant.
    pub fn name(&self) -> &'static str {
        match self {
            VariableType::Real { .. } => "Real",
            VariableType::Integer { .. } => "Integer",
            VariableType::Boolean { .. } => "Boolean",
            VariableType::String { .. } => "String",
            VariableType::Enumeration { .. } => "Enumeration",
        }
    }
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

impl FromStr for Causality {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parameter" => Ok(Causality::Parameter),
            "calculatedParameter" => Ok(Causality::CalculatedParameter),
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
            _ => Err(format!("Unknown intial: {}", s)),
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
    pub value: i32,
    pub description: Option<String>,
    pub textPos: TextPos,
}

#[derive(Debug)]
pub enum SimpleType {
    Real {
        name: String,
        description: Option<String>,
        quantity: Option<String>,
        unit: Option<String>,
        displayUnit: Option<String>,
        relativeQuantity: bool,
        min: Option<f32>,
        max: Option<f32>,
        nominal: Option<f32>,
        unbounded: bool,
        textPos: TextPos,
    },
    Integer {
        name: String,
        description: Option<String>,
        quantity: Option<String>,
        min: Option<i32>,
        max: Option<i32>,
        textPos: TextPos,
    },
    Boolean {
        name: String,
        description: Option<String>,
        textPos: TextPos,
    },
    String {
        name: String,
        description: Option<String>,
        textPos: TextPos,
    },
    Enumeration {
        name: String,
        description: Option<String>,
        items: Vec<Item>,
        quantity: Option<String>,
        textPos: TextPos,
    },
}

impl SimpleType {
    /// Returns the name of the type definition regardless of the variant.
    pub fn name(&self) -> &str {
        match self {
            SimpleType::Real { name, .. }
            | SimpleType::Integer { name, .. }
            | SimpleType::Boolean { name, .. }
            | SimpleType::String { name, .. }
            | SimpleType::Enumeration { name, .. } => name,
        }
    }
}

#[derive(Debug)]
pub struct DefaultExperiment {
    pub startTime: Option<String>,
    pub stopTime: Option<String>,
    pub tolerance: Option<String>,
    pub stepSize: Option<String>,
    pub textPos: TextPos,
}

#[derive(Debug)]
pub struct CoSimulation {
    pub sourceFiles: Vec<String>,
    pub modelIdentifier: String,
    pub needsExecutionTool: bool,
    pub canHandleVariableCommunicationStepSize: bool,
    pub canInterpolateInputs: bool,
    pub maxOutputDerivativeOrder: u32,
    pub canRunAsynchronuously: bool,
    pub canBeInstantiatedOnlyOncePerProcess: bool,
    pub canNotUseMemoryManagementFunctions: bool,
    pub canGetAndSetFMUstate: bool,
    pub canSerializeFMUstate: bool,
    pub providesDirectionalDerivative: bool,
    pub textPos: TextPos,
}

#[derive(Debug)]
pub struct ModelExchange {
    pub sourceFiles: Vec<String>,
    pub modelIdentifier: String,
    pub needsExecutionTool: bool,
    pub completedIntegratorStepNotNeeded: bool,
    pub canBeInstantiatedOnlyOncePerProcess: bool,
    pub canNotUseMemoryManagementFunctions: bool,
    pub canGetAndSetFMUstate: bool,
    pub canSerializeFMUstate: bool,
    pub providesDirectionalDerivative: bool,
    pub textPos: TextPos,
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
    pub textPos: TextPos,
}

#[derive(Debug)]
pub struct Unknown {
    pub index: VariableIndex,
    pub dependencies: Option<Vec<VariableIndex>>,
    pub dependenciesKind: Option<Vec<DependencyKind>>,
    pub textPos: TextPos,
}

#[derive(Debug)]
pub struct ModelDescription {
    pub modelName: String,
    pub guid: String,
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
    pub unitDefintions: Vec<Unit>,
    pub typeDefinitions: Vec<SimpleType>,
    pub modelVariables: Vec<ScalarVariable>,
    pub numberOfEventIndicators: u32,
    pub outputs: Vec<Unknown>,
    pub derivatives: Vec<Unknown>,
    pub initialUnknowns: Vec<Unknown>,
}

impl ModelDescription {
    /// Returns the first variable found with the given value reference.
    pub fn get_variable_by_value_reference(
        &self,
        vr: fmiValueReference,
    ) -> Option<&ScalarVariable> {
        self.modelVariables.iter().find(|v| v.valueReference == vr)
    }

    pub fn get_variable_by_name(&self, name: &str) -> Option<&ScalarVariable> {
        self.modelVariables.iter().find(|v| v.name == name)
    }

    pub fn get_variable_by_index(&self, index: VariableIndex) -> Option<&ScalarVariable> {
        self.modelVariables.get((index - 1) as usize)
    }

    pub fn get_unit<'a>(&'a self, variable: &'a ScalarVariable) -> Option<&'a str> {
        if let VariableType::Real {
            unit, declaredType, ..
        } = &variable.variableType
        {
            if let Some(unit) = unit {
                return Some(unit);
            } else if let Some(declaredType) = declaredType {
                for simple_type in &self.typeDefinitions {
                    if let SimpleType::Real { name, unit, .. } = simple_type
                        && name == declaredType
                    {
                        return unit.as_deref();
                    }
                }
            }
        }
        None
    }

    pub fn get_type_definition(&self, name: &str) -> Option<&SimpleType> {
        self.typeDefinitions.iter().find(|t| t.name() == name)
    }
}
