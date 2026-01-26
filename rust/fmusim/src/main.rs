#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use fmi::{model_description::{Causality, ModelVariable, VariableType, read_model_description}, util::{extract_fmu, sample, write_header}};
use std::{fs::File, io::Write};
use fmi::{SHARED_LIBRARY_EXTENSION, fmi3::{FMU3, PLATFORM_TUPLE}};
use clap::Parser;


#[derive(Parser)]
#[command(name = "fmusim")]
#[command(about = "FMU simulation tool")]
struct Args {
    /// Path to the FMU file or directory containing modelDescription.xml
    filename: String,
    
    /// Enable logging of FMI function calls
    #[arg(long)]
    log_fmi_calls: bool,
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
    
    // println!("{model_description:#?}");

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

    fmu.enterInitializationMode(None, 0.0, Some(1.0));

    fmu.exitInitializationMode();

    let value_references: Vec<u32> = model_description.modelVariables
        .iter()
        .filter(|v| v.causality == Causality::Output)
        .map(|v| v.valueReference)
        .collect();

    let mut values = vec![0.0; value_references.len()];

    let mut eventHandlingNeeded: bool = false;
    let mut terminateSimulation: bool = false;
    let mut earlyReturn: bool = false;
    let mut lastSuccessfulTime: fmi::types::fmiFloat64 = 0.0;

    let mut buffer = File::create("BouncingBall_out.txt").unwrap();
    let output_variables: Vec<&ModelVariable> = model_description.modelVariables.iter().filter(|v| v.causality == Causality::Output).collect();
    
    write_header(&output_variables, &mut buffer).unwrap();

    sample(0.0, &output_variables, &fmu, &mut buffer).unwrap();

    fmu.doStep(0.0, 0.1, true, &mut eventHandlingNeeded, &mut terminateSimulation, &mut earlyReturn, &mut lastSuccessfulTime); 
    
    sample(0.1, &output_variables, &fmu, &mut buffer).unwrap();

    fmu.getFloat64(&value_references, &mut values);

    // Print values separated by spaces
    let values_str: Vec<String> = values.iter().map(|v| v.to_string()).collect();
    println!("{}", values_str.join(" "));

    fmu.terminate();

    fmu.freeInstance();
}
