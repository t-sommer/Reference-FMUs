use crate::model_description::fmi2::{ModelDescription, Unknown, VariableType};

impl ModelDescription {
    /// Checks if the model description is valid according to the FMI 2.0 specification.
    /// Returns a list of problems found in the model description.
    /// If the list is empty, the model description is valid.
    pub fn validate(&self) -> Vec<String> {
        let mut problems: Vec<String> = vec![];

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
                    problems.push(format!(
                        "Illegal variable index in <Unknown index=\"{}\"> ({}:{})",
                        unknown.index, unknown.textPos.row, unknown.textPos.col
                    ));
                    continue;
                }
            };

            if let VariableType::Real { derivative, .. } = &derivative_variable.variableType {
                if let Some(derivative_index) = derivative {
                    match self.get_variable_by_index(*derivative_index) {
                        Some(state_variable) => {
                            if !matches!(state_variable.variableType, VariableType::Real { .. }) {
                                problems.push(format!("The continuous state variable {} ({}:{}) referenced by the derivative {} ({}:{}) is not a Real variable.", state_variable.name, state_variable.textPos.row, state_variable.textPos.col, derivative_variable.name, derivative_variable.textPos.row, derivative_variable.textPos.col));
                            }
                        }
                        None => {
                            problems.push(format!(
                                "Attribute derivative of variable {} ({}:{}) is not a valid variable index.",
                                derivative_variable.name, derivative_variable.textPos.row, derivative_variable.textPos.col
                            ));
                            continue;
                        }
                    };
                } else {
                    problems.push(format!(
                        "Variable {} ({}:{}) is not a derivative.",
                        derivative_variable.name,
                        derivative_variable.textPos.row,
                        derivative_variable.textPos.col
                    ));
                }
            } else {
                problems.push(format!(
                    "Variable {} ({}:{}) is not a real variable.",
                    derivative_variable.name,
                    derivative_variable.textPos.row,
                    derivative_variable.textPos.col
                ));
            }
        }

        // check initial unknowns
        for unknown in &self.initialUnknowns {
            problems.extend(self.validate_unknown(unknown));
        }

        problems
    }

    fn validate_unknown(&self, unknown: &Unknown) -> Vec<String> {
        let mut problems = vec![];

        if !self.is_valid_variable_index(unknown.index) {
            problems.push(format!(
                "Illegal variable index in <Unknown index=\"{}\"> ({}:{})",
                unknown.index, unknown.textPos.row, unknown.textPos.col
            ));
        }

        if let Some(dependencies) = &unknown.dependencies {
            for dependency_index in dependencies {
                if !self.is_valid_variable_index(*dependency_index) {
                    problems.push(format!(
                        "Illegal variable index in dependencies of <Unknown index=\"{}\"> ({}:{}): {}",
                        unknown.index, unknown.textPos.row, unknown.textPos.col, dependency_index
                    ));
                }
            }

            if let Some(dependencies_kind) = &unknown.dependenciesKind
                && dependencies.len() != dependencies_kind.len()
            {
                problems.push(format!("The number of elements in dependenciesKind does not match the number of elements in dependencies of <Unknown index=\"{}\"> ({}:{}).", unknown.index, unknown.textPos.row, unknown.textPos.col));
            }
        }

        problems
    }

    fn is_valid_variable_index(&self, index: u32) -> bool {
        index > 0 && index <= self.modelVariables.len() as u32
    }
}
