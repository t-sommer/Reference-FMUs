#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod recorder;

use tempfile::TempDir;

use crate::{SHARED_LIBRARY_EXTENSION, fmi2::{self, FMU2, types::fmi2Boolean}, model_description::{Causality, ModelDescription, ModelVariable}, sim::{SimulationSettings, fmi2::recorder::Recorder}, types::{fmiStatus::{self, fmiOK, fmiWarning}, fmiValueReference}, util::VariableValue};
use std::{error::Error, fs::File, io::{Write, stdout}};


fn call(status: fmiStatus) -> Result<fmiStatus, Box<dyn Error>> {
    if matches!(status, fmiOK | fmiWarning) {
        Ok(status)
    } else {
        Err(format!("FMI call failed with status: {:?}", status).into())
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
    
    // // Create a map for quick lookup of variables by name
    // let variable_map: HashMap<&str, &ModelVariable> = model_description.modelVariables
    //     .iter()
    //     .map(|var| (var.name.as_str(), var))
    //     .collect();

    // // then the remaining start values
    // for (var_name, literal) in start_values {

    //     if let Some(variable) = variable_map.get(var_name.as_str()) {

    //         if variable.causality == Causality::StructuralParameter { continue; }
                
    //         match parse_variable_value(&variable.variableType, literal) {
    //             Ok(value) => { 
    //                 set_variable_value_fmi2(fmu, variable.valueReference, &value)?;
    //             }
    //             Err(e) => {
    //                 return Err(format!("Invalid value {literal:?} for variable {var_name:?}. {e}").into());
    //             }
    //         }
    //     }

    // }

    Ok(fmiOK)
}

pub fn simulate_cs(settings: &SimulationSettings, model_description: &ModelDescription, unzipdir: &TempDir) -> Result<(), Box<dyn Error>> {

    let co_simulation = &model_description.coSimulation.as_ref().unwrap();

    let shared_library_filename = format!("{}{}", &co_simulation.modelIdentifier, SHARED_LIBRARY_EXTENSION);

    let shared_library_path = unzipdir.path().join("binaries").join(fmi2::PLATFORM).join(shared_library_filename);

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

    let mut fmu = FMU2::new(
        shared_library_path.as_path(),
        "BouncingBall",
        log_fmi_call,
        log_message
    ).unwrap();

    fmu.instantiate(
        &settings.instance_name, 
        fmi2::types::fmi2Type::fmi2CoSimulation, 
        &model_description.instantiationToken,
        None,
        false, 
        false
    );

    if let Err(e) = set_start_values(&settings.start_values, &model_description, &fmu) {
        return Err(format!("Failed to set start values: {e}").into());
    }

    let output_variables: Vec<&ModelVariable> = model_description.modelVariables.iter().filter(|v| v.causality == Causality::Output).collect();

    let mut time = settings.start_time;

    let output_interval = settings.output_interval;

    fmu.setupExperiment(settings.tolerance, time, Some(settings.stop_time));
    fmu.enterInitializationMode();
    fmu.exitInitializationMode();

    let mut recorder = if let Some(path) = &settings.output_file {
        let file = File::create(path).expect("Failed to create output file");
        Recorder::new(output_variables, Box::new(file) as Box<dyn Write>, &fmu)
    } else {
        let stdout_handle = stdout();
        Recorder::new(output_variables, Box::new(stdout_handle) as Box<dyn Write>, &fmu)
    };

    for i in 1..10 {
        
        // if let Some(input) = input {
        //     input.set_discrete_inputs(time, true, fmu)?;
        //     input.set_continuous_inputs(time, true, fmu)?;
        // }

        call(fmu.doStep(time, output_interval, 0))?;

        time = i as f64 * settings.output_interval;

        recorder.sample(time)?;
    }

    call(fmu.terminate())?;

    Ok(())
}
