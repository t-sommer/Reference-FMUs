#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use fmi::{input::CSVInput, model_description::read_model_description, simulate::{SimulationSettings, call, simulate_fmi3_cs}, util::extract_fmu};
use std::{fs::File, path::PathBuf, process::ExitCode};
use fmi::{SHARED_LIBRARY_EXTENSION, fmi3::{FMU3, PLATFORM_TUPLE}};
use clap::Parser;
use libloading::Library;


fn parse_start_value(s: &str) -> Result<(String, String), String> {
    
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    
    if parts.len() != 2 {
        return Err(format!("Invalid format {s:?}. Expected \"variable_name=value\"."));
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

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

    /// CSV file to read the input from
    #[arg(long)]
    input_file: Option<String>,

    /// CSV file to store the output
    #[arg(long)]
    output_file: Option<String>,

    /// Set start values for variables (format: variable_name=value)
    #[arg(long = "start-value", value_parser = parse_start_value)]
    start_values: Vec<(String, String)>,
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
            eprintln!("Failed to read model description: {e}");
            return ExitCode::FAILURE;
        }
    };

    let input = if let Some(path) = &args.input_file {
        match File::open(&path) {
            Ok(file) => {
                match CSVInput::new(&file, &model_description) {
                    Ok(input) => Some(input),
                    Err(e) => {
                        eprintln!("Failed to load input from {path:?}. {e}");
                        return ExitCode::FAILURE; 
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to open input file {path:?}. {e}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        None
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

    let exit_code = if let Err(e) = simulate_fmi3_cs(&settings, &model_description, &fmu, input.as_ref()) {
        eprintln!("ERROR: {e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    };

    fmu.freeInstance();

    exit_code
}