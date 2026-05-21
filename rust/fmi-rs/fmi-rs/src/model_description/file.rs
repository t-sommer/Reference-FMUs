use std::{error::Error, str::FromStr};

use roxmltree::Node;

use crate::model_description::{BaseUnit, DisplayUnit, Unit};

pub(crate) trait StringAttribute {
    // fn optional_attribute(&self, name: &str) -> Option<String>;
    fn required_attribute(&self, name: &str) -> Result<String, Box<dyn Error>>;
    fn bool_attribute(&self, name: &str, default: bool) -> bool;
    fn attribute_as<T: FromStr>(&self, name: &str) -> Result<Option<T>, Box<dyn Error>>;
    fn required_attribute_as<T: FromStr>(&self, name: &str) -> Result<T, Box<dyn Error>>;
    fn optional_u32_attribute(&self, name: &str) -> Result<Option<u32>, Box<dyn Error>>;
    // fn optional_i32_attribute(&self, name: &str) -> Result<Option<i32>, Box<dyn Error>>;
    fn optional_bool_attribute(&self, name: &str) -> Result<Option<bool>, Box<dyn Error>>;
}

pub(crate) trait NodeExt<'a, 'input> {
    fn get_child(&self, name: &str) -> Option<Node<'a, 'input>>;
    fn get_required_child(&self, name: &str) -> Result<Node<'a, 'input>, Box<dyn Error>>;
    fn get_children(&self, name: &str) -> Vec<Node<'a, 'input>>;
}

impl<'a, 'input> StringAttribute for Node<'a, 'input> {
    fn required_attribute(&self, name: &str) -> Result<String, Box<dyn Error>> {
        self.attribute(name)
            .ok_or_else(|| format!("Missing required attribute '{}'", name).into())
            .map(|s| s.to_string())
    }

    // fn optional_attribute(&self, name: &str) -> Option<String> {
    //     self.attribute(name).map(|v| v.to_string())
    // }

    fn bool_attribute(&self, name: &str, default: bool) -> bool {
        if let Some(value) = self.attribute(name) {
            value.parse().unwrap_or(default)
        } else {
            default
        }
    }

    fn attribute_as<T: FromStr>(&self, name: &str) -> Result<Option<T>, Box<dyn Error>> {
        if let Some(literal) = self.attribute(name) {
            let result = literal.parse().map_err(|_| {
                format!(
                    "Illegal value '{}' for attribute '{}' in <{}>.",
                    literal,
                    name,
                    self.tag_name().name()
                )
            });
            Ok(Some(result?))
        } else {
            Ok(None)
        }
    }

    fn required_attribute_as<T: FromStr>(&self, name: &str) -> Result<T, Box<dyn Error>> {
        if let Some(value) = self.attribute(name) {
            value.parse().map_err(|_| {
                format!(
                    "Illegal value '{}' for attribute '{}' in <{}>.",
                    value,
                    name,
                    self.tag_name().name()
                )
                .into()
            })
        } else {
            Err(format!("Missing required attribute '{}' in <{}>.", name, self.tag_name().name()).into())
        }
    }


    fn optional_u32_attribute(&self, name: &str) -> Result<Option<u32>, Box<dyn Error>> {
        if let Some(value) = self.attribute(name) {
            if let Ok(index) = value.parse::<u32>() {
                Ok(Some(index))
            } else {
                Err(format!("Illegal value {} for attribute {} in <{}>.", value, name, self.tag_name().name()).into())
            }
        } else {
            Ok(None)
        }
    }

    fn optional_bool_attribute(&self, name: &str) -> Result<Option<bool>, Box<dyn Error>> {
        if let Some(value) = self.attribute(name) {
            match value {
                "false" | "0" => Ok(Some(false)),
                "true" | "1" => Ok(Some(true)),
                _ => Err(format!("Illegal value {} for attribute {} in <{}>.", value, name, self.tag_name().name()).into()),
            }   
        } else {
            Ok(None)
        }
    }
}

impl<'a, 'input> NodeExt<'a, 'input> for Node<'a, 'input> {
    fn get_child(&self, name: &str) -> Option<Node<'a, 'input>> {
        self.children().find(|n| n.has_tag_name(name))
    }
    
    fn get_required_child(&self, name: &str) -> Result<Node<'a, 'input>, Box<dyn Error>> {
        self.children()
            .find(|n| n.has_tag_name(name))
            .ok_or_else(|| format!("Missing required element <{}> in <{}>", name, self.tag_name().name()).into())
    }
    
    fn get_children(&self, name: &str) -> Vec<Node<'a, 'input>> {
        self.children().filter(|n| n.has_tag_name(name)).collect()
    }
}

impl Unit {
    pub(crate) fn from_node(node: &roxmltree::Node) -> Result<Self, Box<dyn Error>> {
        let name = node.required_attribute("name")?;

        let baseUnit = node
            .descendants()
            .find(|n| n.has_tag_name("BaseUnit"))
            .map(|n| BaseUnit {
                kg: n.attribute_as("kg").unwrap().unwrap_or(0),
                m: n.attribute_as("m").unwrap().unwrap_or(0),
                s: n.attribute_as("s").unwrap().unwrap_or(0),
                A: n.attribute_as("A").unwrap().unwrap_or(0),
                K: n.attribute_as("K").unwrap().unwrap_or(0),
                mol: n.attribute_as("mol").unwrap().unwrap_or(0),
                cd: n.attribute_as("cd").unwrap().unwrap_or(0),
                rad: n.attribute_as("rad").unwrap().unwrap_or(0),
                factor: n.attribute_as("factor").unwrap().unwrap_or(1.0),
                offset: n.attribute_as("offset").unwrap().unwrap_or(0.0),
            });

        let displayUnits = node
            .descendants()
            .filter(|n| n.has_tag_name("DisplayUnit"))
            .map(|n| DisplayUnit {
                name: n.required_attribute("name").unwrap(),
                factor: n.attribute_as("factor").unwrap().unwrap_or(1.0),
                offset: n.attribute_as("offset").unwrap().unwrap_or(0.0),
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
