use std::{any::TypeId, error::Error, str::FromStr};

use roxmltree::Node;

use crate::model_description::{BaseUnit, DisplayUnit, TextPos, Unit};

pub(crate) trait NodeExt<'a, 'input> {
    fn get_child(&self, name: &str) -> Option<Node<'a, 'input>>;
    fn get_required_child(&self, name: &str) -> Result<Node<'a, 'input>, Box<dyn Error>>;
    fn get_children(&self, name: &str) -> Vec<Node<'a, 'input>>;
    fn required_attribute(&self, name: &str) -> Result<String, Box<dyn Error>>;
    fn attribute_as<T: FromStr + 'static>(&self, name: &str) -> Result<Option<T>, Box<dyn Error>>;
    fn required_attribute_as<T: FromStr + 'static>(&self, name: &str) -> Result<T, Box<dyn Error>>;
    fn line_number(&self) -> u32;
    fn text_pos(&self) -> TextPos;
}

impl<'a, 'input> NodeExt<'a, 'input> for Node<'a, 'input> {
    fn line_number(&self) -> u32 {
        self.document().text_pos_at(self.range().start).row
    }

    fn text_pos(&self) -> TextPos {
        let pos = self.document().text_pos_at(self.range().start);
        TextPos {
            row: pos.row,
            col: pos.col,
        }
    }

    fn get_child(&self, name: &str) -> Option<Node<'a, 'input>> {
        self.children().find(|n| n.has_tag_name(name))
    }

    fn get_required_child(&self, name: &str) -> Result<Node<'a, 'input>, Box<dyn Error>> {
        self.children()
            .find(|n| n.has_tag_name(name))
            .ok_or_else(|| {
                format!(
                    "Missing required element <{}> in <{}>",
                    name,
                    self.tag_name().name()
                )
                .into()
            })
    }

    fn get_children(&self, name: &str) -> Vec<Node<'a, 'input>> {
        self.children().filter(|n| n.has_tag_name(name)).collect()
    }

    fn required_attribute(&self, name: &str) -> Result<String, Box<dyn Error>> {
        self.attribute(name)
            .ok_or_else(|| format!("Missing required attribute '{}'", name).into())
            .map(|s| s.to_string())
    }

    fn attribute_as<T: FromStr + 'static>(&self, name: &str) -> Result<Option<T>, Box<dyn Error>> {
        if let Some(literal) = self.attribute(name) {
            let normalized = if TypeId::of::<T>() == TypeId::of::<bool>() {
                match literal {
                    "1" => "true",
                    "0" => "false",
                    _ => literal,
                }
            } else {
                literal
            };
            let result = normalized.parse::<T>().map_err(|_| {
                format!(
                    "Line {}: Illegal value '{}' for attribute '{}' in <{}>.",
                    self.line_number(),
                    literal,
                    name,
                    self.tag_name().name()
                )
            })?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    fn required_attribute_as<T: FromStr + 'static>(&self, name: &str) -> Result<T, Box<dyn Error>> {
        if let Some(value) = self.attribute(name) {
            let normalized = if TypeId::of::<T>() == TypeId::of::<bool>() {
                match value {
                    "1" => "true",
                    "0" => "false",
                    _ => value,
                }
            } else {
                value
            };
            normalized.parse::<T>().map_err(|_| {
                format!(
                    "Line {}: Illegal value '{}' for attribute '{}' in <{}>.",
                    self.line_number(),
                    value,
                    name,
                    self.tag_name().name()
                )
                .into()
            })
        } else {
            Err(format!(
                "Missing required attribute '{}' in <{}>.",
                name,
                self.tag_name().name()
            )
            .into())
        }
    }
}

impl BaseUnit {
    pub(crate) fn from_node(node: &roxmltree::Node) -> Result<Self, Box<dyn Error>> {
        Ok(BaseUnit {
            kg: node.attribute_as("kg")?.unwrap_or_default(),
            m: node.attribute_as("m")?.unwrap_or_default(),
            s: node.attribute_as("s")?.unwrap_or_default(),
            A: node.attribute_as("A")?.unwrap_or_default(),
            K: node.attribute_as("K")?.unwrap_or_default(),
            mol: node.attribute_as("mol")?.unwrap_or_default(),
            cd: node.attribute_as("cd")?.unwrap_or_default(),
            rad: node.attribute_as("rad")?.unwrap_or_default(),
            factor: node.attribute_as("factor")?.unwrap_or(1.0),
            offset: node.attribute_as("offset")?.unwrap_or_default(),
            textPos: node.text_pos(),
        })
    }
}

impl DisplayUnit {
    pub(crate) fn from_node(node: &roxmltree::Node) -> Result<Self, Box<dyn Error>> {
        Ok(DisplayUnit {
            factor: node.attribute_as("factor")?.unwrap_or(1.0),
            offset: node.attribute_as("offset")?.unwrap_or_default(),
            inverse: node.attribute_as("inverse")?.unwrap_or_default(),
            name: node.required_attribute("name")?,
            textPos: node.text_pos(),
        })
    }
}

impl Unit {
    pub(crate) fn from_node(node: &roxmltree::Node) -> Result<Self, Box<dyn Error>> {
        Ok(Unit {
            name: node.required_attribute("name")?,
            baseUnit: node
                .get_child("BaseUnit")
                .map(|n| BaseUnit::from_node(&n))
                .transpose()?,
            displayUnits: node
                .get_children("DisplayUnit")
                .into_iter()
                .map(|n| DisplayUnit::from_node(&n))
                .collect::<Result<Vec<_>, _>>()?,
            textPos: node.text_pos(),
        })
    }
}
