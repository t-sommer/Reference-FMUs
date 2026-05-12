use crate::{model_description::fmi3::{ModelDescription, Unknown}, types::fmiValueReference};

impl ModelDescription {

    pub fn validate(&self) -> Vec<String> {
        let mut problems: Vec<String> = vec![];

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

    fn validate_unknown(&self, unknown: &Unknown) -> Vec<String> {
        let mut problems = vec![];

        if !self.is_valid_value_reference(unknown.valueReference) {
            problems.push(format!(
                "Illegal value reference: {}",
                unknown.valueReference
            ));
        }

        if let Some(dependencies) = &unknown.dependencies {
            for dependency_vr in dependencies {
                if !self.is_valid_value_reference(*dependency_vr) {
                    problems.push(format!(
                        "Illegal value reference in dependencies of unknown: {}",
                        dependency_vr
                    ));
                }
            }

            if let Some(dependencies_kind) = &unknown.dependenciesKind {
                if dependencies.len() != dependencies_kind.len() {
                    problems.push(format!("The number of elements in dependenciesKind does not match the number of elements in dependencies for unknown VR {}.", unknown.valueReference));
                }
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