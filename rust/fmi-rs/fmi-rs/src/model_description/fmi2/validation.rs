use crate::model_description::fmi2::{ModelDescription, Unknown, VariableNamingConvention, VariableType};
use crate::model_description::ValidationError;
use crate::model_description::validation::validate_structured_variable_name;

impl ModelDescription {
    /// Checks if the model description is valid according to the FMI 2.0 specification.
    /// Returns a list of problems found in the model description.
    /// If the list is empty, the model description is valid.
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut problems = vec![];

        let mut variable_names = std::collections::HashSet::new();

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

            if !variable_names.insert(&variable.name) {
                problems.push(ValidationError {
                    range: vec![],
                    message: format!("Duplicate variable name: '{}'", variable.name),
                });
            }
        }

        // check outputs
        for unknown in &self.outputs {
            problems.extend(self.validate_unknown(unknown));
        }

        // check derivatives
        for unknown in &self.derivatives {
            problems.extend(self.validate_unknown(unknown));
        }

        // check continuous states
        for unknown in &self.derivatives {
            let derivative_variable = match self.get_variable_by_index(unknown.index) {
                Some(variable) => variable,
                None => {
                    problems.push(ValidationError {
                        range: vec![],
                        message: format!("Illegal variable index in <Unknown index=\"{}\">", unknown.index),
                    });
                    continue;
                }
            };

            if let VariableType::Real { derivative, .. } = &derivative_variable.variableType {
                if let Some(derivative_index) = derivative {
                    match self.get_variable_by_index(*derivative_index) {
                        Some(state_variable) => {
                            if !matches!(state_variable.variableType, VariableType::Real { .. }) {
                                problems.push(ValidationError {
                    range: vec![],
                                    message: format!("The continuous state variable {} referenced by the derivative {} is not a Real variable.", state_variable.name, derivative_variable.name),
                                });
                            }
                        }
                        None => {
                            problems.push(ValidationError {
                    range: vec![],
                                message: format!("Attribute derivative of variable {} is not a valid variable index.", derivative_variable.name),
                            });
                            continue;
                        }
                    };
                } else {
                    problems.push(ValidationError {
                    range: vec![],
                        message: format!("Variable {} is not a derivative.", derivative_variable.name),
                    });
                }
            } else {
                problems.push(ValidationError {
                    range: vec![],
                    message: format!("Variable {} is not a real variable.", derivative_variable.name),
                });
            }
        }

        // check initial unknowns
        for unknown in &self.initialUnknowns {
            problems.extend(self.validate_unknown(unknown));
        }

        problems
    }

    fn validate_unknown(&self, unknown: &Unknown) -> Vec<ValidationError> {
        let mut problems = vec![];

        if !self.is_valid_variable_index(unknown.index) {
            problems.push(ValidationError {
                range: vec![],
                message: format!("Illegal variable index: {}", unknown.index),
            });
        }

        if let Some(dependencies) = &unknown.dependencies {
            for dependency_index in dependencies {
                if !self.is_valid_variable_index(*dependency_index) {
                    problems.push(ValidationError {
                        range: vec![],
                        message: format!("Illegal variable index in dependencies: {}", dependency_index),
                    });
                }
            }

            if let Some(dependencies_kind) = &unknown.dependenciesKind
                && dependencies.len() != dependencies_kind.len()
            {
                problems.push(ValidationError {
                    range: vec![],
                    message: "The number of elements in dependenciesKind does not match the number of elements in dependencies.".to_string(),
                });
            }
        }

        problems
    }

    fn is_valid_variable_index(&self, index: u32) -> bool {
        index > 0 && index <= self.modelVariables.len() as u32
    }
}
