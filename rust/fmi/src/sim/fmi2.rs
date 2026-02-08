#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod input;
pub mod recorder;

use tempfile::TempDir;

use crate::{SHARED_LIBRARY_EXTENSION, fmi2::{self, FMU2, types::fmi2Boolean}, model_description::{Causality, ModelDescription, ModelVariable, VariableType}, sim::{SimulationSettings, fmi2::{input::CSVInput, recorder::Recorder}}, types::{fmiStatus::{self, fmiOK, fmiWarning}, fmiValueReference}, util::VariableValue};
use std::{collections::HashMap, error::Error, fs::File, io::{Write, stdout}};


fn call(status: fmiStatus) -> Result<fmiStatus, Box<dyn Error>> {
    if matches!(status, fmiOK | fmiWarning) {
        Ok(status)
    } else {
        Err(format!("FMI call failed with status: {:?}", status).into())
    }
}

pub fn parse_variable_value(variable_type: &VariableType, literal: &str) -> Result<VariableValue, Box<dyn Error>> {
    match variable_type {
        VariableType::Float64 => {
            let value: Result<f64, _> = literal.parse();
            Ok(VariableValue::Float64(vec![value?]))
        },
        VariableType::Int32 | VariableType::Enumeration => {
            let value: Result<i32, _> = literal.parse();
            Ok(VariableValue::Int32(vec![value?]))
        },
        VariableType::Boolean => {
            let value: Result<bool, _> = literal.parse();
            Ok(VariableValue::Boolean(vec![value?]))
        },
        VariableType::String => {
            let values: Vec<String> = literal.split_whitespace().map(|v| v.to_string()).collect();
            Ok(VariableValue::String(values))
        },
        _ => Err(format!("Unsupported variable type {variable_type:?}.").into())
    }
}

fn set_variable_value(fmu: &FMU2, value_reference: fmiValueReference, value: &VariableValue) -> Result<fmiStatus, Box<dyn Error>> {
    match value {
        VariableValue::Float64(values) => {
            call(fmu.setReal(&[value_reference], values))
        },
        VariableValue::Int32(values) => {
            call(fmu.setInteger(&[value_reference], values))
        },
        VariableValue::Boolean(values) => {
            let values: Vec<fmi2Boolean> = values.iter().map(|v| if *v { 1 } else { 0 }).collect();
            call(fmu.setBoolean(&[value_reference], &values))
        },
        VariableValue::String(values) => {
            let string_refs: Vec<&str> = values.iter().map(|x| x.as_str()).collect();
            call(fmu.setString(&[value_reference], &string_refs))
        },
        _ => Err("Unsupported variable type {value:?}.".into())
    }
}

fn set_start_values(start_values: &Vec<(String, String)>, model_description: &ModelDescription, fmu: &FMU2) -> Result<fmiStatus, Box<dyn Error>> {
    
    // Create a map for quick lookup of variables by name
    let variable_map: HashMap<&str, &ModelVariable> = model_description.modelVariables
        .iter()
        .map(|var| (var.name.as_str(), var))
        .collect();

    // then the remaining start values
    for (var_name, literal) in start_values {

        if let Some(variable) = variable_map.get(var_name.as_str()) {

            if variable.causality == Causality::StructuralParameter { continue; }
                
            match parse_variable_value(&variable.variableType, literal) {
                Ok(value) => { 
                    set_variable_value(fmu, variable.valueReference, &value)?;
                }
                Err(e) => {
                    return Err(format!("Invalid value {literal:?} for variable {var_name:?}. {e}").into());
                }
            }
        }

    }

    Ok(fmiOK)
}

pub fn simulate_cs(settings: &SimulationSettings, model_description: &ModelDescription, unzipdir: &TempDir) -> Result<(), Box<dyn Error>> {
    
    let start_time = settings.start_time;
    let stop_time = settings.stop_time;
    let set_stop_time = settings.set_stop_time;
    let output_interval = settings.output_interval;
    
    let co_simulation = match &model_description.coSimulation {
        Some(cs) => cs,
        None => {
            return Err("The FMU does not support Co-Simulation.".into());
        }
    };
    
    let can_handle_variable_communication_step_size = co_simulation.canHandleVariableCommunicationStepSize.clone();
    
    let input = if let Some(path) = &settings.input_file {
        match File::open(&path) {
            Ok(file) => {
                match CSVInput::new(&file, &model_description) {
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
    
    let mut time = settings.start_time;
    
    let log_fmi_call = if settings.log_fmi_calls {
        Some(Box::new(|status: &fmiStatus, message: &str| {
            eprintln!("{message} -> {status:?}");
        }) as Box<dyn Fn(&fmiStatus, &str) + Send + Sync>)
    } else {
        None
    };

    let log_message = if settings.log_fmi_calls {
        Some(Box::new(|status: &fmiStatus, category: &str, message: &str| {
            eprintln!("[{status:?}][{category}] {message}");
        }) as Box<dyn Fn(&fmiStatus, &str, &str) + Send + Sync>)
    } else {
        None
    };

    let fmu = FMU2::new(
        unzipdir.as_ref(),
        &co_simulation.modelIdentifier,
        "instance1",
        fmi2::types::fmi2Type::fmi2CoSimulation,
        &model_description.instantiationToken,
        false,
        false,
        log_fmi_call,
        log_message
    )?;

    set_start_values(&settings.start_values, &model_description, &fmu)?;

    call(fmu.setupExperiment(
        settings.tolerance, 
        time, 
        if set_stop_time { Some(stop_time) } else { None }
    ))?;

    call(fmu.enterInitializationMode())?;

    if let Some(input) = &input {
        input.set_discrete_inputs(time, true, &fmu)?;
        input.set_continuous_inputs(time, true, &fmu)?;
    }

    call(fmu.exitInitializationMode())?;
    
    let output_variables: Vec<&ModelVariable> = model_description.modelVariables.iter().filter(|v| v.causality == Causality::Output).collect();

    let mut recorder = if let Some(path) = &settings.output_file {
        let file = File::create(path).expect("Failed to create output file");
        Recorder::new(output_variables, Box::new(file) as Box<dyn Write>, &fmu)
    } else {
        let stdout_handle = stdout();
        Recorder::new(output_variables, Box::new(stdout_handle) as Box<dyn Write>, &fmu)
    };

    recorder.sample(time)?;

    let mut n_steps = 0;

    loop {
        if time > stop_time || relative_eq!(time, stop_time) { 
            break; 
        }

        let next_regular_point = start_time + (n_steps + 1) as f64 * output_interval;

        let mut next_communication_point = next_regular_point;

        if can_handle_variable_communication_step_size {
            if let Some(input) = &input {
                if let Some(next_input_event_time) = input.next_event_time(time) {
                    if next_regular_point > next_input_event_time && !relative_eq!(next_regular_point, next_input_event_time) {
                        next_communication_point = next_input_event_time;
                    }
                }
            }            
        };

        if next_communication_point > stop_time && !relative_eq!(next_communication_point, stop_time) {
            if can_handle_variable_communication_step_size {
                next_communication_point = stop_time;
            } else {
                break;
            }
        }

        let communication_step_size = next_communication_point - time;
        
        if let Some(input) = &input {
            input.set_discrete_inputs(time, true, &fmu)?;
            input.set_continuous_inputs(time, true, &fmu)?;
        }

        let do_step_status = fmu.doStep(time, communication_step_size, 0);

        let mut terminate_simulation = 0;

        if do_step_status == fmiStatus::fmiDiscard {
            call(fmu.getRealStatus(&fmi2::types::fmi2StatusKind::fmi2LastSuccessfulTime, &mut time))?;
            call(fmu.getBooleanStatus(&fmi2::types::fmi2StatusKind::fmi2Terminated, &mut terminate_simulation))?;
        } else {
            call(do_step_status)?;
            time = next_communication_point;
        }

        if relative_eq!(time, next_communication_point) {
            n_steps += 1;
        }

        recorder.sample(time)?;

        if terminate_simulation != 0 {
            break;
        }
    }

    call(fmu.terminate())?;

    Ok(())
}
