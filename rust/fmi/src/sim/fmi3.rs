pub mod input;
pub mod recorder;

use std::{collections::HashMap, error::Error, fs::File, io::{Write, stdout}};
use libloading::Library;
use tempfile::TempDir;

use crate::{SHARED_LIBRARY_EXTENSION, fmi3::{FMU3, PLATFORM_TUPLE}, model_description::{ModelVariable, VariableType}, sim::{SimulationSettings, fmi3::{input::CSVInput, recorder::Recorder}}, types::*, util::VariableValue};
use crate::{model_description::{Causality, ModelDescription}, types::fmiStatus::{self, fmiOK, fmiWarning}};


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

pub fn call(status: fmiStatus) -> Result<fmiStatus, Box<dyn Error>> {
    if matches!(status, fmiOK | fmiWarning) {
        Ok(status)
    } else {
        Err(format!("FMI call failed with status: {:?}", status).into())
    }
}

fn set_start_values(start_values: &Vec<(String, String)>, model_description: &ModelDescription, fmu: &FMU3) -> Result<fmiStatus, Box<dyn Error>> {

    let mut configuration_mode = false;

    // Create a map for quick lookup of variables by name
    let variable_map: HashMap<&str, &ModelVariable> = model_description.modelVariables
        .iter()
        .map(|var| (var.name.as_str(), var))
        .collect();

    // set structural parameters first
    for (var_name, value) in start_values {

        if let Some(variable) = variable_map.get(var_name.as_str()) {

            if variable.causality == Causality::StructuralParameter {

                if !configuration_mode {
                    call(fmu.enterConfigurationMode())?;
                    configuration_mode = true;
                }
                
                let value_references = [variable.valueReference];
                let values: Result<Vec<u64>, _> = value.split_whitespace().map(|v| v.parse()).collect();
                match values {
                    Ok(vals) => {
                        fmu.setUInt64(&value_references, &vals);
                    }
                    Err(_) => {
                        return Err(format!("Invalid integer value {value:?} for variable {var_name:?}.").into());
                    }
                }

                
            }
        }
    }

    if configuration_mode {
        call(fmu.exitConfigurationMode())?;
    }
    
    // then the remaining start values
    for (var_name, literal) in start_values {

        if let Some(variable) = variable_map.get(var_name.as_str()) {

            if variable.causality == Causality::StructuralParameter { continue; }
                
            match parse_variable_value(&variable.variableType, literal) {
                Ok(value) => { 
                    set_variable_value(fmu, variable.valueReference, &value); 
                }
                Err(e) => {
                    return Err(format!("Invalid value {literal:?} for variable {var_name:?}. {e}").into());
                }
            }
        }

    }

    Ok(fmiOK)
}

pub fn simulate_cs(settings: &SimulationSettings) -> Result<(), Box<dyn Error>> {

    let co_simulation = match &settings.model_description.coSimulation {
        Some(cs) => cs,
        None => {
            return Err("The FMU does not support Co-Simulation.".into());
        }
    };

    let input = if let Some(path) = &settings.input_file {
        match File::open(&path) {
            Ok(file) => {
                match CSVInput::new(&file, &settings.model_description) {
                    Ok(input) => Some(input),
                    Err(e) => {
                        return Err(format!("Failed to load input from {path:?}. {e}").into());
                    }
                }
            }
            Err(e) => {
                return Err(format!("Failed to open input file {path:?}. {e}").into());
            }
        }
    } else {
        None
    };

    let log_fmi_call = if settings.log_fmi_calls {
        Some(Box::new(|status: &fmiStatus, message: &str| {
            eprintln!("{message} -> {status:?}");
        }) as Box<dyn Fn(&fmiStatus, &str) + Send + Sync>)
    } else {
        None
    };

    let log_message = if settings.log_fmi_calls {
        Some(Box::new(|status: &fmiStatus, category: &str, message: &str| {
            eprintln!("[Message][{:?}][{}] {}", status, category, message);
        }) as Box<dyn Fn(&fmiStatus, &str, &str) + Send + Sync>)
    } else {
        None
    };

    let fmu = FMU3::instantiateCoSimulation(
        settings.unzipdir.as_ref(),
        &co_simulation.modelIdentifier,
        "instance1",
        &settings.model_description.instantiationToken,
        false,
        false,
        false,
        false,
        &[],
        log_fmi_call, 
        log_message
    )?;

    if let Err(e) = set_start_values(&settings.start_values, &settings.model_description, &fmu) {
        return Err(format!("Failed to set start values: {e}").into());
    }

    let mut time = 0.0;

    let output_interval = settings.output_interval;

    fmu.enterInitializationMode(settings.tolerance, settings.start_time, Some(settings.stop_time));
    fmu.exitInitializationMode();

    let mut recorder = if let Some(path) = &settings.output_file {
        let file = File::create(path).expect("Failed to create output file");
        Recorder::new(&settings.output_variables, Box::new(file) as Box<dyn Write>, &fmu)
    } else {
        let stdout_handle = stdout();
        Recorder::new(&settings.output_variables, Box::new(stdout_handle) as Box<dyn Write>, &fmu)
    };

    while time < settings.stop_time {
        
        if let Some(input) = &input {
            input.set_discrete_inputs(time, true, &fmu)?;
            input.set_continuous_inputs(time, true, &fmu)?;
        }

        let mut eventHandlingNeeded = false;
        let mut terminateSimulation = false;
        let mut earlyReturn = false;

        call(fmu.doStep(time, output_interval, true, &mut eventHandlingNeeded, &mut terminateSimulation, &mut earlyReturn, &mut time))?; 

        recorder.sample(time)?;
    }

    call(fmu.terminate())?;

    Ok(())
}