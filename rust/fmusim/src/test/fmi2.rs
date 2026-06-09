use colored::Colorize;
use std::path::{Path, PathBuf};

use fmi::{
    fmi2::{CS, FMU2, ME, types::fmi2FMUstate},
    model_description,
};

const DEFAULT_INSTANCE_NAME: &str = "instance";

pub struct FMU2Factory {
    pub model_description: model_description::fmi2::ModelDescription,
    pub unzipdir: PathBuf,
    pub visible: bool,
    pub loggingOn: bool,
    pub logCalls: bool,
    pub printCalls: bool,
    pub logMessages: bool,
    pub printMessages: bool,
    pub provideMemoryManagementFunctions: bool,
}

impl FMU2Factory {
    pub fn instantiate_me(&self) -> Result<FMU2<ME>, Box<dyn std::error::Error>> {
        if let Some(me) = &self.model_description.modelExchange {
            FMU2::<ME>::new(
                self.unzipdir.as_path(),
                &me.modelIdentifier,
                DEFAULT_INSTANCE_NAME,
                &self.model_description.guid,
                self.visible,
                self.loggingOn,
                self.logCalls,
                self.printCalls,
                self.logMessages,
                self.printMessages,
                !me.canNotUseMemoryManagementFunctions,
            )
        } else {
            return Err("Model-Exchange is not supported.".into());
        }
    }

    pub fn instantiate_cs(&self) -> Result<FMU2<CS>, Box<dyn std::error::Error>> {
        if let Some(cs) = &self.model_description.coSimulation {
            FMU2::<CS>::new(
                self.unzipdir.as_path(),
                &cs.modelIdentifier,
                DEFAULT_INSTANCE_NAME,
                &self.model_description.guid,
                self.visible,
                self.loggingOn,
                self.logCalls,
                self.printCalls,
                self.logMessages,
                self.printMessages,
                self.provideMemoryManagementFunctions,
            )
        } else {
            return Err("Co-Simulation is not supported.".into());
        }
    }
}

pub fn test_set_fmu_state(
    factory: &FMU2Factory,
) -> Result<(), Box<dyn std::error::Error>> {

    if factory.model_description.modelExchange.as_ref().map(|me| me.canGetAndSetFMUstate).unwrap_or(false) {
        println!("{}", "    Testing set FMU state (ME)".green().bold());
        let fmu = factory.instantiate_me()?;
        get_and_set_fmu_state(fmu);
    }

    if factory.model_description.coSimulation.as_ref().map(|cs| cs.canGetAndSetFMUstate).unwrap_or(false) {
        println!("{}", "    Testing set FMU state (CS)".green().bold());
        let fmu = factory.instantiate_cs()?;
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
    factory: &FMU2Factory,
) -> Result<(), Box<dyn std::error::Error>> {
    if factory.model_description.modelExchange.as_ref().map(|me| me.canSerializeFMUstate).unwrap_or(false) {
        println!("{}", "    Testing serialize FMU state (ME)".green().bold());
        let fmu = factory.instantiate_me()?;
        serialize_fmu_state(fmu);
    }

    if factory.model_description.coSimulation.as_ref().map(|cs| cs.canSerializeFMUstate).unwrap_or(false) {
        println!("{}", "    Testing serialize FMU state (CS)".green().bold());
        let fmu = factory.instantiate_cs()?;
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
