use colored::Colorize;
use std::{
    ffi::c_uint,
    path::{Path, PathBuf},
    process::ExitCode,
};

use fmi::{
    fmi3::{FMU3, types::fmi3FMUState},
    model_description::{self, fmi3::ModelDescription},
    util::extract_fmu,
};

use crate::TestArgs;

const DEFAULT_INSTANCE_NAME: &str = "instance";

pub struct FMUFactory {
    pub model_description: ModelDescription,
    pub unzipdir: PathBuf,
    pub visible: bool,
    pub loggingOn: bool,
    pub eventModeUsed: bool,
    pub earlyReturnAllowed: bool,
    pub requiredIntermediateVariables: Vec<c_uint>,
    pub logCalls: bool,
    pub printCalls: bool,
    pub logMessages: bool,
    pub printMessages: bool,
}

impl FMUFactory {
    pub fn instantiate_me(&self) -> Result<FMU3, Box<dyn std::error::Error>> {
        if let Some(me) = &self.model_description.modelExchange {
            FMU3::instantiateModelExchange(
                self.unzipdir.as_path(),
                &me.modelIdentifier,
                DEFAULT_INSTANCE_NAME,
                &self.model_description.instantiationToken,
                self.visible,
                self.loggingOn,
                self.logCalls,
                self.printCalls,
                self.logMessages,
                self.printMessages,
            )
        } else {
            return Err("Model-Exchange is not supported.".into());
        }
    }

    pub fn instantiate_cs(&self) -> Result<FMU3, Box<dyn std::error::Error>> {
        if let Some(cs) = &self.model_description.coSimulation {
            FMU3::instantiateCoSimulation(
                self.unzipdir.as_path(),
                &cs.modelIdentifier,
                DEFAULT_INSTANCE_NAME,
                &self.model_description.instantiationToken,
                self.visible,
                self.loggingOn,
                self.eventModeUsed,
                self.earlyReturnAllowed,
                &self.requiredIntermediateVariables,
                self.logCalls,
                self.printCalls,
                self.logMessages,
                self.printMessages,
            )
        } else {
            return Err("Co-Simulation is not supported.".into());
        }
    }
}

pub fn test_set_fmu_state(factory: &FMUFactory) -> Result<(), Box<dyn std::error::Error>> {
    if factory
        .model_description
        .modelExchange
        .as_ref()
        .map(|me| me.canGetAndSetFMUState)
        .unwrap_or(false)
    {
        println!("{}", "    Testing set FMU state (ME)".green().bold());
        let fmu = factory.instantiate_me()?;
        get_and_set_fmu_state(fmu);
    }

    if factory
        .model_description
        .coSimulation
        .as_ref()
        .map(|cs| cs.canGetAndSetFMUState)
        .unwrap_or(false)
    {
        println!("{}", "    Testing set FMU state (CS)".green().bold());
        let fmu = factory.instantiate_cs()?;
        get_and_set_fmu_state(fmu);
    }

    Ok(())
}

fn get_and_set_fmu_state(fmu: FMU3) {
    let mut fmu_state: fmi3FMUState = std::ptr::null_mut();
    fmu.getFMUState(&mut fmu_state);
    fmu.setFMUState(fmu_state);
    fmu.freeFMUState(&mut fmu_state);
}

pub fn test_serialize_fmu_state(factory: &FMUFactory) -> Result<(), Box<dyn std::error::Error>> {
    if factory
        .model_description
        .modelExchange
        .as_ref()
        .map(|me| me.canSerializeFMUState)
        .unwrap_or(false)
    {
        println!("{}", "    Testing serialize FMU state (ME)".green().bold());
        let fmu = factory.instantiate_me()?;
        serialize_fmu_state(fmu);
    }

    if factory
        .model_description
        .coSimulation
        .as_ref()
        .map(|cs| cs.canSerializeFMUState)
        .unwrap_or(false)
    {
        println!("{}", "    Testing serialize FMU state (CS)".green().bold());
        let fmu = factory.instantiate_cs()?;
        serialize_fmu_state(fmu);
    }

    Ok(())
}

fn serialize_fmu_state(fmu: FMU3) {
    let mut fmu_state: fmi3FMUState = std::ptr::null_mut();
    fmu.getFMUState(&mut fmu_state);
    let mut size = 0;
    fmu.serializedFMUStateSize(fmu_state, &mut size);
    let mut serialized_fmu_state = vec![0; size];
    fmu.serializeFMUState(fmu_state, &mut serialized_fmu_state);
    fmu.freeFMUState(&mut fmu_state);
    let mut deserialized_fmu_state: fmi3FMUState = std::ptr::null_mut();
    fmu.deserializeFMUState(&serialized_fmu_state, &mut deserialized_fmu_state);
    fmu.setFMUState(deserialized_fmu_state);
}

// fn call(status: fmi2Status) -> Result<(), Box<dyn std::error::Error>> {
//     if matches!(status, fmi2Status::fmi2OK | fmi2Status::fmi2Warning) {
//         Ok(())
//     } else {
//         Err("FMI call failed".into())
//     }
// }

// fn assert_equal<T: PartialEq + std::fmt::Debug>(
//     variable_name: &str,
//     expected: T,
//     actual: T,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     if actual.eq(&expected) {
//         Ok(())
//     } else {
//         Err(format!(
//             "Expected start value for variable {variable_name:?} is {expected:?} but got {actual:?}"
//         )
//         .into())
//     }
// }

// pub fn test_get_all_variables(factory: &FMUFactory) -> Result<(), Box<dyn std::error::Error>> {
//     let fmu = factory.instantiate_me()?;

//     fmu.enterInitializationMode();
//     fmu.exitInitializationMode();

//     for variable in factory.model_description.modelVariables.iter() {
//         match &variable.variableType {
//             VariableType::Real { start, .. } => {
//                 let mut values = [0.0];
//                 call(fmu.getReal(&[variable.valueReference], &mut values))?;
//                 if variable.initial == Some(Initial::Exact)
//                     && let Some(literal) = start
//                 {
//                     let expected = literal.parse()?;
//                     assert_equal(&variable.name, expected, values[0])?;
//                 }
//             }
//             VariableType::Integer { start, .. } | VariableType::Enumeration { start, .. } => {
//                 let mut values = [0];
//                 call(fmu.getInteger(&[variable.valueReference], &mut values))?;
//                 if variable.initial == Some(Initial::Exact)
//                     && let Some(literal) = start
//                 {
//                     let expected = literal.parse()?;
//                     assert_equal(&variable.name, expected, values[0])?;
//                 }
//             }
//             VariableType::Boolean { start, .. } => {
//                 let mut values = [fmi2False];
//                 call(fmu.getBoolean(&[variable.valueReference], &mut values))?;
//                 if variable.initial == Some(Initial::Exact)
//                     && let Some(literal) = start
//                 {
//                     let expected = match literal.as_str() {
//                         "true" => fmi2True,
//                         "false" => fmi2False,
//                         "1" => fmi2True,
//                         "0" => fmi2False,
//                         _ => return Err(format!("Invalid boolean literal: {literal}").into()),
//                     };
//                     assert_equal(&variable.name, expected, values[0])?;
//                 }
//             }
//             VariableType::String { start, .. } => {
//                 let mut values = [String::new()];
//                 call(fmu.getString(&[variable.valueReference], &mut values))?;
//                 if variable.initial == Some(Initial::Exact)
//                     && let Some(literal) = start
//                 {
//                     assert_equal(&variable.name, literal, &values[0])?;
//                 }
//             }
//         }
//     }

//     Ok(())
// }

macro_rules! bail {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => {
                eprintln!("{}: {}", "error".red().bold(), e);
                return ExitCode::FAILURE;
            }
        }
    };
}

pub fn smoke_test_fmi3(args: &TestArgs) -> ExitCode {
    let unzipdir = bail!(extract_fmu(&args.fmu_file));

    let model_description = bail!(ModelDescription::from_path(
        &unzipdir.path().join("modelDescription.xml")
    ));

    let factory = FMUFactory {
        model_description,
        unzipdir: unzipdir.path().to_path_buf(),
        visible: false,
        loggingOn: false,
        eventModeUsed: false,
        earlyReturnAllowed: false,
        requiredIntermediateVariables: vec![],
        logCalls: args.log_fmi_calls,
        printCalls: true,
        logMessages: true,
        printMessages: true,
    };

    if let Err(e) = test_set_fmu_state(&factory) {
        println!("{}: {}", "error".red().bold(), e);
    }

    if let Err(e) = test_serialize_fmu_state(&factory) {
        println!("{}: {}", "error".red().bold(), e);
    }

    // if let Err(e) = test_get_all_variables(&factory) {
    //     println!("{}: {}", "error".red().bold(), e);
    // }

    ExitCode::SUCCESS
}
