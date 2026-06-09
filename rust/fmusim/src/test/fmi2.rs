use colored::Colorize;
use std::path::Path;

use fmi::{
    fmi2::{CS, FMU2, ME, types::fmi2FMUstate},
    model_description,
};

const DEFAULT_INSTANCE_NAME: &str = "instance";

pub fn test_set_fmu_state(
    model_description: &model_description::fmi2::ModelDescription,
    unzipdir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "    Testing set FMU state".green().bold());

    let guid = &model_description.guid;
    let visible = false;
    let loggingOn = false;
    let logCalls = true;
    let printCalls = true;
    let logMessages = true;
    let printMessages = true;
    let provideMemoryManagementFunctions = true;

    if let Some(co_simulation) = &model_description.coSimulation {
        let modelIdentifier = &co_simulation.modelIdentifier;

        let fmu = FMU2::<CS>::new(
            unzipdir,
            modelIdentifier,
            DEFAULT_INSTANCE_NAME,
            guid,
            visible,
            loggingOn,
            logCalls,
            printCalls,
            logMessages,
            printMessages,
            provideMemoryManagementFunctions,
        )?;

        get_and_set_fmu_state(fmu);
    }

    if let Some(model_exchange) = &model_description.modelExchange {
        let modelIdentifier = &model_exchange.modelIdentifier;

        let fmu = FMU2::<ME>::new(
            unzipdir,
            modelIdentifier,
            DEFAULT_INSTANCE_NAME,
            guid,
            visible,
            loggingOn,
            logCalls,
            printCalls,
            logMessages,
            printMessages,
            provideMemoryManagementFunctions,
        )?;

        get_and_set_fmu_state(fmu);
    }

    Ok(())
}

fn get_and_set_fmu_state<T>(fmu: FMU2<T>) {
    let mut fmu_state: fmi2FMUstate = std::ptr::null_mut();
    fmu.getFMUstate(&mut fmu_state);
    fmu.setFMUstate(fmu_state);
    fmu.freeFMUstate(&mut fmu_state);
}

pub fn test_serialize_fmu_state(
    model_description: &model_description::fmi2::ModelDescription,
    unzipdir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "    Testing serilize FMU state".green().bold());

    let guid = &model_description.guid;
    let visible = false;
    let loggingOn = false;
    let logCalls = true;
    let printCalls = true;
    let logMessages = true;
    let printMessages = true;
    let provideMemoryManagementFunctions = true;

    if let Some(co_simulation) = &model_description.coSimulation {
        let modelIdentifier = &co_simulation.modelIdentifier;

        let fmu = FMU2::<CS>::new(
            unzipdir,
            modelIdentifier,
            DEFAULT_INSTANCE_NAME,
            guid,
            visible,
            loggingOn,
            logCalls,
            printCalls,
            logMessages,
            printMessages,
            provideMemoryManagementFunctions,
        )?;

        serialize_fmu_state(fmu);
    }

    if let Some(model_exchange) = &model_description.modelExchange {
        let modelIdentifier = &model_exchange.modelIdentifier;

        let fmu = FMU2::<ME>::new(
            unzipdir,
            modelIdentifier,
            DEFAULT_INSTANCE_NAME,
            guid,
            visible,
            loggingOn,
            logCalls,
            printCalls,
            logMessages,
            printMessages,
            provideMemoryManagementFunctions,
        )?;

        serialize_fmu_state(fmu);
    }

    Ok(())
}

fn serialize_fmu_state<T>(fmu: FMU2<T>) {
    let mut fmu_state: fmi2FMUstate = std::ptr::null_mut();

    fmu.getFMUstate(&mut fmu_state);

    let mut serialized_fmu_state = vec![];

    fmu.serializeFMUstate(fmu_state, &mut serialized_fmu_state);

    fmu.freeFMUstate(&mut fmu_state);

    let mut deserialized_fmu_state: fmi2FMUstate = std::ptr::null_mut();

    fmu.deSerializeFMUstate(&serialized_fmu_state, &mut deserialized_fmu_state);

    fmu.setFMUstate(deserialized_fmu_state);
}
