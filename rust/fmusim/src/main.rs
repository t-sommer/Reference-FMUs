#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use fmi::{model_description::{Causality, ModelDescription, ModelVariable, VariableType, read_model_description}, types::fmiStatus::{self, fmiOK, fmiWarning}, util::{Recorder, extract_fmu}};
use std::{collections::HashMap, error::Error, fs::File, io::{Write, stdout}, process::ExitCode};
use fmi::{SHARED_LIBRARY_EXTENSION, fmi3::{FMU3, PLATFORM_TUPLE}};
use clap::{Parser, parser::ValueSource};


#[derive(Parser)]
#[command(name = "fmusim")]
#[command(about = "FMU simulation tool")]
struct Args {
    /// Path to the FMU file
    filename: String,
    
    /// Enable logging of FMI function calls
    #[arg(long)]
    log_fmi_calls: bool,

    /// Interval for sampling the output variables
    #[arg(long)]
    output_interval: Option<f64>,

    /// Stop time for the simulation
    #[arg(long)]
    stop_time: Option<f64>,

    /// File to store the output as CSV
    #[arg(long)]
    output_file: Option<String>,

    /// Set start values for variables (format: variable_name=value)
    #[arg(long = "start-value", value_parser = parse_start_value)]
    start_values: Vec<(String, String)>,
}

fn parse_start_value(s: &str) -> Result<(String, String), String> {
    
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    
    if parts.len() != 2 {
        return Err(format!("Invalid format {s:?}. Expected \"variable_name=value\"."));
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn call(status: fmiStatus) -> Result<fmiStatus, Box<dyn Error>> {
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

fn simulate() -> Result<(), Box<dyn Error>> {

    let args = match Args::try_parse() {
        Ok(a) => a,
        Err(e) => {
            return Err(format!("Failed to parse command line arguments. {e}").into());
        }
    };

    // Extract FMU to temporary directory
    let temp_dir = match extract_fmu(&args.filename) {
        Ok(dir) => dir,
        Err(e) => {
            return Err(format!("Failed to extract FMU: {}", e).into());
        }
    };

    // Path to modelDescription.xml in the extracted directory
    let xml_path = temp_dir.path().join("modelDescription.xml");

    let model_description = match read_model_description(xml_path.as_path()) {
        Ok(desc) => desc,
        Err(e) => {
            return Err(format!("ERROR: Failed to read model description: {}", e).into());
        }
    };
    
    // println!("{model_description:#?}");

    // Create logging callbacks only if requested
    let log_fmi_call = if args.log_fmi_calls {
        Some(Box::new(|status: &fmi::types::fmiStatus, message: &str| {
            eprintln!("{message} -> {status:?}");
        }) as Box<dyn Fn(&fmi::types::fmiStatus, &str) + Send + Sync>)
    } else {
        None
    };

    let log_message = if args.log_fmi_calls {
        Some(Box::new(|status: &fmi::types::fmiStatus, category: &str, message: &str| {
            eprintln!("[Message][{:?}][{}] {}", status, category, message);
        }) as Box<dyn Fn(&fmi::types::fmiStatus, &str, &str) + Send + Sync>)
    } else {
        None
    };

    let unzipdir = temp_dir.path();

    let shared_library_filename = format!("{}{}", model_description.coSimulation.as_ref().unwrap().modelIdentifier, SHARED_LIBRARY_EXTENSION);

    let shared_library_path = unzipdir.join("binaries").join(PLATFORM_TUPLE).join(shared_library_filename);

    let mut fmu = FMU3::new(
        shared_library_path.as_path(), 
        "instance1", 
        log_fmi_call, 
        log_message
    ).expect("Failed to load FMU");

    fmu.instantiateCoSimulation(
        &model_description.modelName,
        &model_description.instantiationToken,
        None, 
        false, 
        false, 
        false, 
        false, 
        &[]
    );

    if let Err(e) = set_start_values(&args.start_values, &model_description, &fmu) {
        return Err(format!("Failed to set start values: {e}").into());
    }

    let output_variables: Vec<&ModelVariable> = model_description.modelVariables.iter().filter(|v| v.causality == Causality::Output).collect();

    let mut time = 0.0;
    let stop_time = args.stop_time
        .or_else(|| model_description.defaultExperiment.as_ref().and_then(|exp| exp.stopTime.as_ref().and_then(|s| s.parse().ok())))
        .unwrap_or(1.0);

    let output_interval = args.output_interval.unwrap_or(stop_time / 10.0);

    fmu.enterInitializationMode(None, 0.0, Some(stop_time));

    fmu.exitInitializationMode();

    let mut recorder = if let Some(path) = args.output_file {
        let file = File::create(path).expect("Failed to create output file");
        Recorder::new(output_variables, Box::new(file) as Box<dyn Write>, &fmu)
    } else {
        let stdout_handle = stdout();
        Recorder::new(output_variables, Box::new(stdout_handle) as Box<dyn Write>, &fmu)
    };

    while time < stop_time {
        
        let mut eventHandlingNeeded = false;
        let mut terminateSimulation = false;
        let mut earlyReturn = false;

        call(fmu.doStep(time, output_interval, true, &mut eventHandlingNeeded, &mut terminateSimulation, &mut earlyReturn, &mut time))?; 

        recorder.sample(time)?;
    }

    call(fmu.terminate())?;

    fmu.freeInstance();

    Ok(())
}

fn main() -> ExitCode {
    if let Err(e) = simulate() {
        eprintln!("ERROR: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}