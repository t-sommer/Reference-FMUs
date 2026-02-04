#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use fmi::{fmi2::self, input::CSVInput, model_description::{self, Causality, CoSimulation, ModelDescription, ModelVariable, read_model_description}, sim::{self, SimulationSettings, fmi2::simulate_cs, fmi3::{call}}, util::extract_fmu};
use tempfile::TempDir;
use std::{error::Error, fs::File, path::PathBuf, process::ExitCode};
use fmi::{SHARED_LIBRARY_EXTENSION, fmi3::{FMU3, PLATFORM_TUPLE}, fmi2::FMU2};
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

// fn simulate_fmi3(args: &Args, model_description: &ModelDescription, unzipdir: &TempDir) -> ExitCode {

//     let input = if let Some(path) = &args.input_file {
//         match File::open(&path) {
//             Ok(file) => {
//                 match CSVInput::new(&file, &model_description) {
//                     Ok(input) => Some(input),
//                     Err(e) => {
//                         eprintln!("Failed to load input from {path:?}. {e}");
//                         return ExitCode::FAILURE; 
//                     }
//                 }
//             }
//             Err(e) => {
//                 eprintln!("Failed to open input file {path:?}. {e}");
//                 return ExitCode::FAILURE;
//             }
//         }
//     } else {
//         None
//     };

//     // Create logging callbacks only if requested
//     let log_fmi_call = if args.log_fmi_calls {
//         Some(Box::new(|status: &fmi::types::fmiStatus, message: &str| {
//             eprintln!("{message} -> {status:?}");
//         }) as Box<dyn Fn(&fmi::types::fmiStatus, &str) + Send + Sync>)
//     } else {
//         None
//     };

//     let log_message = if args.log_fmi_calls {
//         Some(Box::new(|status: &fmi::types::fmiStatus, category: &str, message: &str| {
//             eprintln!("[Message][{:?}][{}] {}", status, category, message);
//         }) as Box<dyn Fn(&fmi::types::fmiStatus, &str, &str) + Send + Sync>)
//     } else {
//         None
//     };

//     let co_simulation = match &model_description.coSimulation {
//         Some(cs) => cs,
//         None => {
//             eprintln!("ERROR: The FMU does not support Co-Simulation.");
//             return ExitCode::FAILURE;
//         }
//     };

//     let shared_library_filename = format!("{}{}", co_simulation.modelIdentifier, SHARED_LIBRARY_EXTENSION);

//     let shared_library_path = unzipdir.path().join("binaries").join(PLATFORM_TUPLE).join(shared_library_filename);

//     if !shared_library_path.is_file() {
//         eprintln!("ERROR: The FMU contains no platform binary for {PLATFORM_TUPLE}.");
//         return ExitCode::FAILURE;
//     }

//     let library = unsafe {
//         match  Library::new(&shared_library_path)  {
//             Ok(l) => l,
//             Err(e) => {
//                 eprintln!("Failed to load platform binary {shared_library_path:?}. {e}");
//                 return ExitCode::FAILURE;
//             }
//         }
//     };

//     let (start_time, stop_time, tolerance) = if let Some(default_experiment) = &model_description.defaultExperiment {
//         let start_time: f64 = if let Some(v) = &default_experiment.startTime { v.parse().unwrap() } else { 0.0 };
//         let stop_time: f64 = if let Some(v) = &default_experiment.stopTime { v.parse().unwrap() } else { start_time + 1.0 };
//         let tolerance: Option<f64> = default_experiment.tolerance.as_ref().map(|v| v.parse().unwrap());
//         (start_time, stop_time, tolerance)
//     } else {
//         (0.0, 1.0, None)
//     };

//     let internal_step_size: Option<f64> = co_simulation.fixedInternalStepSize.as_ref().map(|v| v.parse().unwrap());
 
//     let start_time = args.start_time.unwrap_or(start_time);
//     let stop_time = args.stop_time.unwrap_or(stop_time);
//     let tolerance = if let Some(v) = args.tolerance { Some(v) } else { tolerance };

//     let output_interval = if let Some(v) = args.output_interval {
//         v
//     } else {
//         if let Some(v) = internal_step_size {
//             v
//         } else {
//             (stop_time - start_time) / 10.0
//         }
//     };

//     let settings = SimulationSettings {
//         start_time,
//         stop_time,
//         output_interval,
//         tolerance,
//         start_values: args.start_values.clone(),
//         output_file: args.output_file.clone().map(|f| PathBuf::from(f)),
//         log_fmi_calls: args.log_fmi_calls,
//         instance_name: "instance1".to_string(),
//     };

//     let start_time = std::time::Instant::now();

//     let exit_code = match model_description.fmiVersion.as_str() {
//         "2.0" => {
//             let mut fmu = FMU2::new(
//                 shared_library_path.as_path(),
//                 "",
//                 log_fmi_call,
//                 log_message
//             ).unwrap();

//             fmu.freeInstance();

//             ExitCode::FAILURE
//         },
//         _ => {
//             let mut fmu = match FMU3::new(
//                 &library,
//                 "instance1", 
//                 log_fmi_call, 
//                 log_message
//             ) {
//                 Ok(fmu) => fmu,
//                 Err(e) => {
//                     eprintln!("Failed to load shared library. {e}");
//                     return ExitCode::FAILURE;
//                 }
//             };
        
//             if let Err(e) = call(fmu.instantiateCoSimulation(
//                 &model_description.modelName,
//                 &model_description.instantiationToken,
//                 None, 
//                 false, 
//                 false, 
//                 false, 
//                 false, 
//                 &[]
//             )) {
//                 eprintln!("ERROR: {e}");
//                 return ExitCode::FAILURE;
//             }
            
//             let exit_code = if let Err(e) = simulate_fmi3_cs(&settings, &model_description, &fmu, input.as_ref()) {
//                 eprintln!("ERROR: {e}");
//                 ExitCode::FAILURE
//             } else {
//                 ExitCode::SUCCESS
//             };
            
//             fmu.freeInstance();

//             exit_code            
//         },
//     };

//     let simulation_duration = start_time.elapsed();
    
//     eprintln!("Simulation took in {:.3} seconds", simulation_duration.as_secs_f64());

//     exit_code
// }

// fn simulate_fmi2(args: &Args, model_description: &ModelDescription, unzipdir: &TempDir) -> Result<(), Box<dyn Error>> {

//     // Create logging callbacks only if requested
//     let log_fmi_call = if args.log_fmi_calls {
//         Some(Box::new(|status: &fmi::types::fmiStatus, message: &str| {
//             eprintln!("{message} -> {status:?}");
//         }) as Box<dyn Fn(&fmi::types::fmiStatus, &str) + Send + Sync>)
//     } else {
//         None
//     };

//     let log_message = if args.log_fmi_calls {
//         Some(Box::new(|status: &fmi::types::fmiStatus, category: &str, message: &str| {
//             eprintln!("[Message][{:?}][{}] {}", status, category, message);
//         }) as Box<dyn Fn(&fmi::types::fmiStatus, &str, &str) + Send + Sync>)
//     } else {
//         None
//     };

//     let co_simulation = model_description.coSimulation.as_ref().unwrap();

//     let shared_library_filename = format!("{}{}", &co_simulation.modelIdentifier, SHARED_LIBRARY_EXTENSION);

//     let shared_library_path = unzipdir.path().join("binaries").join(fmi2::PLATFORM).join(shared_library_filename);

//     if !shared_library_path.is_file() {
//         return Err("The FMU contains no platform binary for {PLATFORM_TUPLE}.".into());
//     }

//     let mut fmu = FMU2::new(
//         shared_library_path.as_path(),
//         "BouncingBall",
//         log_fmi_call,
//         log_message
//     ).unwrap();

//     call(fmu.instantiate(
//         &co_simulation.modelIdentifier, 
//         fmi2Type::fmi2CoSimulation,
//         &model_description.instantiationToken,
//         None,
//         false,
//         false))?;
        
//     let mut time = 0.0;

//     call(fmu.setupExperiment(None, time, None))?;
//     call(fmu.enterInitializationMode())?;
//     call(fmu.exitInitializationMode())?;

//     let h = 0.1;

//     for i in 1..10 {
//         call(fmu.doStep(time, h, 1))?;
//         time = i as f64 * h;
//     }

//     call(fmu.terminate())?;

//     Ok(())
// }

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

    // let co_simulation = CoSimulation {
    //     modelIdentifier: "BouncingBall".to_string(),
    //     fixedInternalStepSize: None,
    // };

    // let h = ModelVariable {
    //     name: "h".to_string(),
    //     variableType: model_description::VariableType::Float64,
    //     valueReference: 1,
    //     causality: Causality::Output,
    //     variability: model_description::Variability::Continuous,
    //     dimensions: vec![],
    // };

    // let model_variables = vec![h];

    // let model_description = ModelDescription {
    //     fmiVersion: "2.0".to_string(),
    //     modelName: "BouncingBall".to_string(),
    //     defaultExperiment: None,
    //     instantiationToken: "{1AE5E10D-9521-4DE3-80B9-D0EAAA7D5AF1}".to_string(),
    //     coSimulation: Some(co_simulation),
    //     modelVariables: model_variables,
    // };

    let model_description = match read_model_description(xml_path.as_path()) {
        Ok(desc) => desc,
        Err(e) => {
            eprintln!("Failed to read model description: {e}");
            return ExitCode::FAILURE;
        }
    };

    let (start_time, stop_time, tolerance) = if let Some(default_experiment) = &model_description.defaultExperiment {
        let start_time: f64 = if let Some(v) = &default_experiment.startTime { v.parse().unwrap() } else { 0.0 };
        let stop_time: f64 = if let Some(v) = &default_experiment.stopTime { v.parse().unwrap() } else { start_time + 1.0 };
        let tolerance: Option<f64> = default_experiment.tolerance.as_ref().map(|v| v.parse().unwrap());
        (start_time, stop_time, tolerance)
    } else {
        (0.0, 1.0, None)
    };

    let internal_step_size: Option<f64> = model_description.coSimulation.as_ref().unwrap().fixedInternalStepSize.as_ref().map(|v| v.parse().unwrap());
 
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
        start_values: args.start_values.clone(),
        output_file: args.output_file.as_ref().map(|f| PathBuf::from(f)),
        log_fmi_calls: args.log_fmi_calls,
        instance_name: "instance1".to_string(),
        input_file: args.input_file.as_ref().map(|f| PathBuf::from(f)),
    };

    // return match sim::fmi2::simulate_cs(&settings, &model_description, &unzipdir) {
    //     Ok(_) => ExitCode::SUCCESS,
    //     Err(e) => ExitCode::FAILURE,
    // }

    return match sim::fmi3::simulate_cs(&settings, &model_description, &unzipdir) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => ExitCode::FAILURE,
    }


    // simulate_fmi3(&args, &model_description, &unzipdir)
}