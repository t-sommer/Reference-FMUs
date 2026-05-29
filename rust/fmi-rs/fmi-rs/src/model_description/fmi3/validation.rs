use std::collections::HashMap;

use crate::{
    model_description::{ValidationError, fmi3::{Causality, ModelDescription, ModelVariable, Unknown, VariableNamingConvention, VariableType}, validation::validate_structured_variable_name},
    types::fmiValueReference,
};

impl ModelDescription {
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut problems = vec![];

        let mut value_references: HashMap<u32, &ModelVariable> = HashMap::new();
        let mut variable_names: HashMap<&String, &ModelVariable> = HashMap::new();

        for variable in &self.modelVariables {
            if variable.name.is_empty() {
                problems.push(ValidationError {
                    range: vec![],
                    message: "Variable name cannot be empty.".to_string(),
                });
            } else if self.variableNamingConvention == VariableNamingConvention::Structured {
                if let Err(message) = validate_structured_variable_name(&variable.name) {
                    problems.push(ValidationError {
                    range: vec![],
                        message: format!("Variable name '{}' does not conform to the structured naming convention: {}", variable.name, message),
                    });
                }
            }

            if let Some(duplicate) = value_references.get(&variable.valueReference) {
                problems.push(ValidationError {
                    range: vec![duplicate.range.clone(), variable.range.clone()],
                    message: format!("Duplicate value reference: {}", variable.valueReference),
                });
            } else {
                value_references.insert(variable.valueReference, variable);
            }

            if let Some(duplicate) = variable_names.get(&variable.name) {
                problems.push(ValidationError {
                    range: vec![duplicate.range.clone(), variable.range.clone()],
                    message: format!("Duplicate variable name: '{}'", variable.name),
                });
            } else {
                variable_names.insert(&variable.name, variable);
            }
        }

        let independent_variables: Vec<_> = self.modelVariables.iter().filter(|v| v.causality == Causality::Independent).collect();

        if independent_variables.len() == 1 {
            let independent_variable = independent_variables[0];
            if !matches!(independent_variable.variableType, VariableType::Float32 {..} | VariableType::Float64 {..}) {
                problems.push(ValidationError {
                    range: vec![],
                    message: "The independent variable must be of type Float32 or Float64.".to_string(),
                });
            }
        } else {
            problems.push(ValidationError {
                    range: vec![],
                message: "There must be exactly one independent variable.".to_string(),
            });
        }

        for unknown in &self.outputs {
            problems.extend(self.validate_unknown(unknown));
        }
        for unknown in &self.derivatives {
            problems.extend(self.validate_unknown(unknown));
        }
        for unknown in &self.initialUnknowns {
            problems.extend(self.validate_unknown(unknown));
        }

        problems
    }

    fn validate_unknown(&self, unknown: &Unknown) -> Vec<ValidationError> {
        let mut problems = vec![];

        if !self.is_valid_value_reference(unknown.valueReference) {
            problems.push(ValidationError {
                range: vec![unknown.range.clone()],
                message: format!("Illegal value reference: {}", unknown.valueReference),
            });
        }

        if let Some(dependencies) = &unknown.dependencies {
            for dependency_vr in dependencies {
                if !self.is_valid_value_reference(*dependency_vr) {
                    problems.push(ValidationError {
                        range: vec![unknown.range.clone()],
                        message: format!("Illegal value reference in dependencies: {}", dependency_vr),
                    });
                }
            }

            if let Some(dependencies_kind) = &unknown.dependenciesKind
                && dependencies.len() != dependencies_kind.len()
            {
                problems.push(ValidationError {
                    range: vec![unknown.range.clone()],
                    message: "The number of elements in dependenciesKind does not match the number of elements in dependencies.".to_string(),
                });
            }
        }

        problems
    }

    fn is_valid_value_reference(&self, valueReference: fmiValueReference) -> bool {
        self.modelVariables
            .iter()
            .any(|v| v.valueReference == valueReference)
    }
}
