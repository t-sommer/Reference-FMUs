#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use fmi::{model_description::{Causality, ModelVariable, read_model_description}, types::fmiStatus::fmiOK, util::{Recorder, extract_fmu, sample, write_header}};
use std::fs::File;
use fmi::{SHARED_LIBRARY_EXTENSION, fmi3::{FMU3, PLATFORM_TUPLE}};
use clap::Parser;


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
}


fn main() {

    let args = Args::parse();

    // Extract FMU to temporary directory
    let temp_dir = match extract_fmu(&args.filename) {
        Ok(dir) => dir,
        Err(e) => {
            println!("ERROR: Failed to extract FMU: {}", e);
            return;
        }
    };

    // Path to modelDescription.xml in the extracted directory
    let xml_path = temp_dir.path().join("modelDescription.xml");

    let model_description = read_model_description(xml_path.as_path()).unwrap();
    
    println!("{model_description:#?}");

    // Create logging callbacks only if requested
    let log_fmi_call = if args.log_fmi_calls {
        Some(Box::new(|status: &fmi::types::fmiStatus, message: &str| {
            println!("{message} -> {status:?}");
        }) as Box<dyn Fn(&fmi::types::fmiStatus, &str) + Send + Sync>)
    } else {
        None
    };

    let log_message = if args.log_fmi_calls {
        Some(Box::new(|status: &fmi::types::fmiStatus, category: &str, message: &str| {
            println!("[Message][{:?}][{}] {}", status, category, message);
        }) as Box<dyn Fn(&fmi::types::fmiStatus, &str, &str) + Send + Sync>)
    } else {
        None
    };

    let unzipdir = temp_dir.path();

    let shared_library_filename = format!("{}{}", model_description.coSimulation.unwrap().modelIdentifier, SHARED_LIBRARY_EXTENSION);

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

    let output_variables: Vec<&ModelVariable> = model_description.modelVariables.iter().filter(|v| v.causality == Causality::Output).collect();
    let mut outfile = File::create("out.csv").unwrap();

    let mut recorder = Recorder {
        variables: output_variables,
        stream: &mut outfile,
        fmu: &fmu,
        sizes: vec![]
    };

    let mut time = 0.0;
    let stop_time = args.stop_time
        .or_else(|| model_description.defaultExperiment.as_ref().and_then(|exp| exp.stopTime.as_ref().and_then(|s| s.parse().ok())))
        .unwrap_or(1.0);

    let output_interval = args.output_interval.unwrap_or(stop_time / 10.0);

    fmu.enterInitializationMode(None, 0.0, Some(stop_time));

    fmu.exitInitializationMode();

    // let mut buffer = if let Some(path) = args.output_file {
    //     Some(File::create(path).expect("Failed to create output file"))
    // } else {
    //     None
    // };

    // let output_variables: Vec<&ModelVariable> = model_description.modelVariables.iter().filter(|v| v.causality == Causality::Output).collect();
    
    recorder.write_header().unwrap();

    // if let Some(ref mut file) = buffer {
    //     write_header(&output_variables, file).unwrap();
    //     sample(0.0, &output_variables, &fmu, file).unwrap();
    // }

    while time < stop_time {
        
        let mut eventHandlingNeeded = false;
        let mut terminateSimulation = false;
        let mut earlyReturn = false;

        let status = fmu.doStep(time, output_interval, true, &mut eventHandlingNeeded, &mut terminateSimulation, &mut earlyReturn, &mut time); 
        
        if status != fmiOK {
            return;
        }

        recorder.sample(time).unwrap();

        // if let Some(ref mut file) = buffer {
        //     sample(time, &output_variables, &fmu, file).unwrap();
        // }
    }

    fmu.terminate();

    fmu.freeInstance();
}
