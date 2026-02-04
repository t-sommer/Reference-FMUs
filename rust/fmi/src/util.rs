use std::{error::Error, fmt::Display, fs::File, io::{self, Write}};
use zip::ZipArchive;
use tempfile::TempDir;

use crate::{fmi3::FMU3, model_description::{Dimension, ModelVariable, VariableType}, types::{*}};

#[derive(Debug, PartialEq)]
pub enum VariableValue {
    Float32(Vec<fmiFloat32>),
    Float64(Vec<fmiFloat64>),
    Int8(Vec<fmiInt8>),
    UInt8(Vec<fmiUInt8>),
    Int16(Vec<fmiInt16>),
    UInt16(Vec<fmiUInt16>),
    Int32(Vec<fmiInt32>),
    UInt32(Vec<fmiUInt32>),
    Int64(Vec<fmiInt64>),
    UInt64(Vec<fmiUInt64>),
    Boolean(Vec<fmiBoolean>),
    String(Vec<String>),
    Binary(Vec<Vec<fmiByte>>),
    // Clock(fmiClock),
}

pub fn extract_fmu(fmu_path: &str) -> Result<TempDir, Box<dyn std::error::Error>> {

    // Create temporary directory
    let temp_dir = TempDir::new()?;

    // Open the FMU file (which is a ZIP archive)
    let file = File::open(fmu_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Extract all files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => temp_dir.path().join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            // Directory
            std::fs::create_dir_all(&outpath)?;
        } else {
            // File
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(temp_dir)
}

pub fn parse_variable_value(variable_type: &VariableType, literal: &str) -> Result<VariableValue, Box<dyn Error>> {
    match variable_type {
        VariableType::Float32 => {
            let values: Result<Vec<fmiFloat32>, _> = literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Float32(values?))
        },
        VariableType::Float64 => {
            let values: Result<Vec<fmiFloat64>, _> = literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Float64(values?))
        },
        VariableType::Int8 => {
            let values: Result<Vec<fmiInt8>, _> = literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Int8(values?))
        },
        VariableType::UInt8 => {
            let values: Result<Vec<fmiUInt8>, _> = literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::UInt8(values?))
        },
        VariableType::Int16 => {
            let values: Result<Vec<fmiInt16>, _> = literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Int16(values?))
        },
        VariableType::UInt16 => {
            let values: Result<Vec<fmiUInt16>, _> = literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::UInt16(values?))
        },
        VariableType::Int32 => {
            let values: Result<Vec<fmiInt32>, _> = literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Int32(values?))
        },
        VariableType::UInt32 => {
            let values: Result<Vec<fmiUInt32>, _> = literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::UInt32(values?))
        },
        VariableType::Int64 | VariableType::Enumeration => {
            let values: Result<Vec<fmiInt64>, _> = literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Int64(values?))
        },
        VariableType::UInt64 => {
            let values: Result<Vec<fmiUInt64>, _> = literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::UInt64(values?))
        },
        VariableType::Boolean | VariableType::Clock => {
            let values: Result<Vec<fmiBoolean>, _> = literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Boolean(values?))
        },
        VariableType::String => {
            let values: Vec<String> = literal.split_whitespace().map(|v| v.to_string()).collect();
            Ok(VariableValue::String(values))
        },
        VariableType::Binary => {
            let values: Result<Vec<Vec<fmiByte>>, Box<dyn Error>> = literal.split_whitespace()
                .map(|hex_str| {
                    
                    if hex_str.len() % 2 != 0 {
                        return Err(format!("Invalid hex string length: {}", hex_str).into());
                    }
                    
                    let mut bytes = Vec::new();

                    for i in (0..hex_str.len()).step_by(2) {
                        let byte_str = &hex_str[i..i+2];
                        match u8::from_str_radix(byte_str, 16) {
                            Ok(byte) => bytes.push(byte),
                            Err(e) => return Err(format!("Invalid hex byte '{}': {}", byte_str, e).into()),
                        }
                    }

                    Ok(bytes)
                })
                .collect();
            Ok(VariableValue::Binary(values?))
        },
    }
}

pub fn set_variable_value(fmu: &FMU3, value_reference: fmiValueReference, value: &VariableValue) -> fmiStatus {
    match value {
        VariableValue::Float32(values) => {
            fmu.setFloat32(&[value_reference], values)
        },
        VariableValue::Float64(values) => {
            fmu.setFloat64(&[value_reference], values)
        },
        VariableValue::Int8(values) => {
            fmu.setInt8(&[value_reference], values)
        },
        VariableValue::UInt8(values) => {
            fmu.setUInt8(&[value_reference], values)
        },
        VariableValue::Int16(values) => {
            fmu.setInt16(&[value_reference], values)
        },
        VariableValue::UInt16(values) => {
            fmu.setUInt16(&[value_reference], values)
        },
        VariableValue::Int32(values) => {
            fmu.setInt32(&[value_reference], values)
        },
        VariableValue::UInt32(values) => {
            fmu.setUInt32(&[value_reference], values)
        },
        VariableValue::Int64(values) => {
            fmu.setInt64(&[value_reference], values)
        },
        VariableValue::UInt64(values) => {
            fmu.setUInt64(&[value_reference], values)
        },
        VariableValue::Boolean(values) => {
            fmu.setBoolean(&[value_reference], values)
        },
        VariableValue::String(values) => {
            let string_refs: Vec<&str> = values.iter().map(|x| x.as_str()).collect();
            fmu.setString(&[value_reference], &string_refs)
        },
        VariableValue::Binary(values) => {
            let sizes: Vec<usize> = values.iter().map(|v| v.len()).collect();
            let values: Vec<*const u8> = values.iter().map(|v| v.as_ptr()).collect();
            fmu.setBinary(&[value_reference], &sizes, &values)
        },
    }
}