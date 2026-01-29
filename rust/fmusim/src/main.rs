#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use fmi::{model_description::{Causality, ModelDescription, ModelVariable, VariableType, read_model_description}, types::fmiStatus::{self, fmiOK, fmiWarning}, util::{Recorder, extract_fmu}};
use std::{collections::HashMap, error::Error, fs::File, io::{Write, stdout}, path::PathBuf, process::ExitCode};
use fmi::{SHARED_LIBRARY_EXTENSION, fmi3::{FMU3, PLATFORM_TUPLE}};
use clap::Parser;
use libloading::Library;


#[derive(Parser)]
#[command(name = "fmusim", version, about = "FMU simulation tool")]
struct Args {
    /// Path to the FMU file
    filename: String,
    
    /// Enable logging of FMI function calls
    #[arg(long)]
    log_fmi_calls: bool,

    /// Interval for sampling the output variables
    #[arg(long)]
    output_interval: Option<f64>,

    /// Start time for the simulation
    #[arg(long)]
    start_time: Option<f64>,

    /// Stop time for the simulation
    #[arg(long)]
    stop_time: Option<f64>,

    /// Relative tolerance for the simulation
    #[arg(long)]
    tolerance: Option<f64>,

    /// File to store the output as CSV
    #[arg(long)]
    output_file: Option<String>,

    /// Set start values for variables (format: variable_name=value)
    #[arg(long = "start-value", value_parser = parse_start_value)]
    start_values: Vec<(String, String)>,
}

struct SimulationSettings {
    start_time: f64,
    stop_time: f64,
    output_interval: f64,
    tolerance: Option<f64>,
    start_values: Vec<(String, String)>,
    output_file: Option<PathBuf>,
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

fn simulate_fmi3_cs(settings: &SimulationSettings, model_description: &ModelDescription, fmu: &FMU3) -> Result<(), Box<dyn Error>> {

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
        
        let mut eventHandlingNeeded = false;
        let mut terminateSimulation = false;
        let mut earlyReturn = false;

        call(fmu.doStep(time, output_interval, true, &mut eventHandlingNeeded, &mut terminateSimulation, &mut earlyReturn, &mut time))?; 

        recorder.sample(time)?;
    }

    call(fmu.terminate())?;

    Ok(())
}

fn main() -> ExitCode {

    // Parse command line arguments
    let args = Args::parse();

    // Extract FMU to temporary directory
    let unzipdir = match extract_fmu(&args.filename) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Failed to extract FMU: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Read the modelDescription.xml
    let xml_path = unzipdir.path().join("modelDescription.xml");

    let model_description = match read_model_description(xml_path.as_path()) {
        Ok(desc) => desc,
        Err(e) => {
            eprintln!("ERROR: Failed to read model description: {e}");
            return ExitCode::FAILURE;
        }
    };

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

    let co_simulation = match &model_description.coSimulation {
        Some(cs) => cs,
        None => {
            eprintln!("ERROR: The FMU does not support Co-Simulation.");
            return ExitCode::FAILURE;
        }
    };

    let shared_library_filename = format!("{}{}", co_simulation.modelIdentifier, SHARED_LIBRARY_EXTENSION);

    let shared_library_path = unzipdir.path().join("binaries").join(PLATFORM_TUPLE).join(shared_library_filename);

    if !shared_library_path.is_file() {
        eprintln!("ERROR: The FMU contains no platform binary for {PLATFORM_TUPLE}.");
        return ExitCode::FAILURE;
    }

    let library = unsafe {
        match  Library::new(&shared_library_path)  {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to load platform binary {shared_library_path:?}. {e}");
                return ExitCode::FAILURE;
            }
        }
    };

    let mut fmu = match FMU3::new(
        &library,
        "instance1", 
        log_fmi_call, 
        log_message
    ) {
        Ok(fmu) => fmu,
        Err(e) => {
            eprintln!("Failed to load shared library. {e}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = call(fmu.instantiateCoSimulation(
        &model_description.modelName,
        &model_description.instantiationToken,
        None, 
        false, 
        false, 
        false, 
        false, 
        &[]
    )) {
        eprintln!("ERROR: {e}");
        return ExitCode::FAILURE;
    }

    let (start_time, stop_time, tolerance) = if let Some(default_experiment) = &model_description.defaultExperiment {
        let start_time: f64 = if let Some(v) = &default_experiment.startTime { v.parse().unwrap() } else { 0.0 };
        let stop_time: f64 = if let Some(v) = &default_experiment.stopTime { v.parse().unwrap() } else { start_time + 1.0 };
        let tolerance: Option<f64> = default_experiment.tolerance.as_ref().map(|v| v.parse().unwrap());
        (start_time, stop_time, tolerance)
    } else {
        (0.0, 1.0, None)
    };

    let internal_step_size: Option<f64> = co_simulation.fixedInternalStepSize.as_ref().map(|v| v.parse().unwrap());
 
    let start_time = args.start_time.unwrap_or(start_time);
    let stop_time = args.stop_time.unwrap_or(stop_time);
    let tolerance = if let Some(v) = args.tolerance { Some(v) } else { tolerance };

    let output_interval = if let Some(v) = args.output_interval {
        v
    } else {
        if let Some(v) = internal_step_size {
            v
        } else {
            (stop_time - start_time) / 10.0
        }
    };

    let settings = SimulationSettings {
        start_time,
        stop_time,
        output_interval,
        tolerance,
        start_values: args.start_values,
        output_file: args.output_file.map(|f| PathBuf::from(f)),
    };

    let exit_code = if let Err(e) = simulate_fmi3_cs(&settings, &model_description, &fmu) {
        eprintln!("ERROR: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    };

    fmu.freeInstance();

    exit_code
}