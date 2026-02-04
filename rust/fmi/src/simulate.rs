#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use crate::{input::CSVInput, model_description::{Causality, ModelDescription, ModelVariable, Variability, VariableType, read_model_description}, types::fmiStatus::{self, fmiOK, fmiWarning}, util::{extract_fmu}, recorder::Recorder};
use std::{collections::HashMap, error::Error, fs::File, io::{Write, stdout}, path::PathBuf, process::ExitCode};
use crate::{SHARED_LIBRARY_EXTENSION, fmi3::{FMU3, PLATFORM_TUPLE}};
use libloading::Library;

pub struct SimulationSettings {
    pub start_time: f64,
    pub stop_time: f64,
    pub output_interval: f64,
    pub tolerance: Option<f64>,
    pub start_values: Vec<(String, String)>,
    pub output_file: Option<PathBuf>,
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
    for (var_name, value) in start_values {

        if let Some(variable) = variable_map.get(var_name.as_str()) {

            if variable.causality != Causality::StructuralParameter {
                
                let value_references = [variable.valueReference];
                
                let _status = match variable.variableType {
                    VariableType::Float64 => {
                        let values: Result<Vec<f64>, _> = value.split_whitespace().map(|v| v.parse()).collect();
                        match values {
                            Ok(vals) => call(fmu.setFloat64(&value_references, &vals))?,
                            Err(_) => {
                                return Err(format!("Invalid float value {value:?} for variable {var_name:?}.").into());
                            }
                        }
                    }
                    VariableType::UInt64 => {
                        let values: Result<Vec<u64>, _> = value.split_whitespace().map(|v| v.parse()).collect();
                        match values {
                            Ok(vals) => call(fmu.setUInt64(&value_references, &vals))?,
                            Err(_) => {
                                return Err(format!("Invalid integer value {value:?} for variable {var_name:?}.").into());
                            }
                        }
                    }
                    _ => todo!()
                };
                
            }
            
        }

    }

    Ok(fmiOK)
}

pub fn simulate_fmi3_cs(settings: &SimulationSettings, model_description: &ModelDescription, fmu: &FMU3, input: &CSVInput) -> Result<(), Box<dyn Error>> {

    if let Err(e) = set_start_values(&settings.start_values, &model_description, &fmu) {
        return Err(format!("Failed to set start values: {e}").into());
    }

    let output_variables: Vec<&ModelVariable> = model_description.modelVariables.iter().filter(|v| v.causality == Causality::Output).collect();

    let mut time = 0.0;

    let output_interval = settings.output_interval;

    fmu.enterInitializationMode(settings.tolerance, settings.start_time, Some(settings.stop_time));
    fmu.exitInitializationMode();

    let mut recorder = if let Some(path) = &settings.output_file {
        let file = File::create(path).expect("Failed to create output file");
        Recorder::new(output_variables, Box::new(file) as Box<dyn Write>, &fmu)
    } else {
        let stdout_handle = stdout();
        Recorder::new(output_variables, Box::new(stdout_handle) as Box<dyn Write>, &fmu)
    };

    while time < settings.stop_time {
        
        input.set_discrete_inputs(time, true, fmu)?;
        input.set_continuous_inputs(time, true, fmu)?;

        let mut eventHandlingNeeded = false;
        let mut terminateSimulation = false;
        let mut earlyReturn = false;

        call(fmu.doStep(time, output_interval, true, &mut eventHandlingNeeded, &mut terminateSimulation, &mut earlyReturn, &mut time))?; 

        recorder.sample(time)?;
    }

    call(fmu.terminate())?;

    Ok(())
}