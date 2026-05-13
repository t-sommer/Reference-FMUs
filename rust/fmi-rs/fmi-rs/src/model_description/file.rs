use std::{error::Error, str::FromStr};

use roxmltree::Node;

use crate::model_description::{BaseUnit, DisplayUnit, Unit};

pub(crate) trait StringAttribute {
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

impl Unit {
    pub(crate) fn from_node(node: &roxmltree::Node) -> Result<Self, Box<dyn Error>> {
        let name = node.required_attribute("name")?;

        let baseUnit = node
            .descendants()
            .find(|n| n.has_tag_name("BaseUnit"))
            .map(|n| BaseUnit {
                kg: n.optional_attribute_as("kg").unwrap_or(0),
                m: n.optional_attribute_as("m").unwrap_or(0),
                s: n.optional_attribute_as("s").unwrap_or(0),
                A: n.optional_attribute_as("A").unwrap_or(0),
                K: n.optional_attribute_as("K").unwrap_or(0),
                mol: n.optional_attribute_as("mol").unwrap_or(0),
                cd: n.optional_attribute_as("cd").unwrap_or(0),
                rad: n.optional_attribute_as("rad").unwrap_or(0),
                factor: n.optional_attribute_as("factor").unwrap_or(1.0),
                offset: n.optional_attribute_as("offset").unwrap_or(0.0),
            });

        let displayUnits = node
            .descendants()
            .filter(|n| n.has_tag_name("DisplayUnit"))
            .map(|n| DisplayUnit {
                name: n.required_attribute("name").unwrap(),
                factor: n.optional_attribute_as("factor").unwrap_or(1.0),
                offset: n.optional_attribute_as("offset").unwrap_or(0.0),
                inverse: n.bool_attribute("inverse", false),
            })
            .collect();

        Ok(Unit {
            name,
            baseUnit,
            displayUnits,
        })
    }
}
