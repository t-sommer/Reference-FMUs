use colored::Colorize;
use std::{ffi::c_uint, path::PathBuf, process::ExitCode};

use fmi_rs::{
    fmi3::{
        FMU3,
        log::{DefaultLogger, Logger},
        types::{fmi3FMUState, fmi3Status},
    },
    model_description::fmi3::ModelDescription,
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
    pub logFile: Option<PathBuf>,
}

impl FMUFactory {
    pub fn instantiate_me(&self) -> Result<FMU3, Box<dyn std::error::Error>> {
        if let Some(me) = &self.model_description.modelExchange {
            let logger = if let Some(log_file) = &self.logFile {
                let stream = std::fs::File::create(log_file)
                    .map_err(|e| format!("Failed to create log file: {}", e))?;
                DefaultLogger::new(stream)
            } else {
                DefaultLogger::new(std::io::stderr())
            };

            FMU3::instantiateModelExchange(
                self.unzipdir.as_path(),
                &me.modelIdentifier,
                DEFAULT_INSTANCE_NAME,
                &self.model_description.instantiationToken,
                self.visible,
                self.loggingOn,
                Box::new(logger),
                self.logCalls,
            )
        } else {
            Err("Model-Exchange is not supported.".into())
        }
    }

    pub fn instantiate_cs(&self) -> Result<FMU3, Box<dyn std::error::Error>> {
        if let Some(cs) = &self.model_description.coSimulation {
            let logger = if let Some(log_file) = &self.logFile {
                let stream = std::fs::File::create(log_file)
                    .map_err(|e| format!("Failed to create log file: {}", e))?;
                DefaultLogger::new(stream)
            } else {
                DefaultLogger::new(std::io::stderr())
            };

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
                Box::new(logger),
                self.logCalls,
            )
        } else {
            Err("Co-Simulation is not supported.".into())
        }
    }
}

fn call(status: fmi3Status) -> Result<(), Box<dyn std::error::Error>> {
    if matches!(status, fmi3Status::fmi3OK | fmi3Status::fmi3Warning) {
        Ok(())
    } else {
        Err("FMI call failed".into())
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
        get_and_set_fmu_state(fmu)?;
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
        get_and_set_fmu_state(fmu)?;
    }

    Ok(())
}

fn get_and_set_fmu_state(fmu: FMU3) -> Result<(), Box<dyn std::error::Error>> {
    let mut fmu_state: fmi3FMUState = std::ptr::null_mut();
    call(fmu.getFMUState(&mut fmu_state))?;
    call(fmu.setFMUState(fmu_state))?;
    call(fmu.freeFMUState(&mut fmu_state))?;
    Ok(())
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
        serialize_fmu_state(fmu)?;
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
        serialize_fmu_state(fmu)?;
    }

    Ok(())
}

fn serialize_fmu_state(fmu: FMU3) -> Result<(), Box<dyn std::error::Error>> {
    let mut fmu_state: fmi3FMUState = std::ptr::null_mut();
    call(fmu.getFMUState(&mut fmu_state))?;
    let mut size = 0;
    call(fmu.serializedFMUStateSize(fmu_state, &mut size))?;
    let mut serialized_fmu_state = vec![0; size];
    call(fmu.serializeFMUState(fmu_state, &mut serialized_fmu_state))?;
    call(fmu.freeFMUState(&mut fmu_state))?;
    let mut deserialized_fmu_state: fmi3FMUState = std::ptr::null_mut();
    call(fmu.deserializeFMUState(&serialized_fmu_state, &mut deserialized_fmu_state))?;
    call(fmu.setFMUState(deserialized_fmu_state))?;
    Ok(())
}

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

pub fn test_get_all_variables(factory: &FMUFactory) -> Result<(), Box<dyn std::error::Error>> {
    if factory.model_description.modelExchange.is_some() {
        println!("{}", "    Testing serialize FMU state (ME)".green().bold());
        let fmu = factory.instantiate_me()?;
        get_all_variables(&fmu, &factory.model_description)?;
    }

    if factory.model_description.coSimulation.is_some() {
        println!("{}", "    Testing serialize FMU state (CS)".green().bold());
        let fmu = factory.instantiate_cs()?;
        get_all_variables(&fmu, &factory.model_description)?;
    }

    Ok(())
}

pub fn get_all_variables(
    fmu: &FMU3,
    model_description: &ModelDescription,
) -> Result<(), Box<dyn std::error::Error>> {
    call(fmu.enterInitializationMode(None, 0.0, None))?;
    call(fmu.exitInitializationMode())?;

    for variable in model_description.modelVariables.iter() {
        // match &variable.variableType {
        //     VariableType::Real { start, .. } => {
        //         let mut values = [0.0];
        //         call(fmu.getReal(&[variable.valueReference], &mut values))?;
        //         if variable.initial == Some(Initial::Exact)
        //             && let Some(literal) = start
        //         {
        //             let expected = literal.parse()?;
        //             assert_equal(&variable.name, expected, values[0])?;
        //         }
        //     }
        //     VariableType::Integer { start, .. } | VariableType::Enumeration { start, .. } => {
        //         let mut values = [0];
        //         call(fmu.getInteger(&[variable.valueReference], &mut values))?;
        //         if variable.initial == Some(Initial::Exact)
        //             && let Some(literal) = start
        //         {
        //             let expected = literal.parse()?;
        //             assert_equal(&variable.name, expected, values[0])?;
        //         }
        //     }
        //     VariableType::Boolean { start, .. } => {
        //         let mut values = [fmi2False];
        //         call(fmu.getBoolean(&[variable.valueReference], &mut values))?;
        //         if variable.initial == Some(Initial::Exact)
        //             && let Some(literal) = start
        //         {
        //             let expected = match literal.as_str() {
        //                 "true" => fmi2True,
        //                 "false" => fmi2False,
        //                 "1" => fmi2True,
        //                 "0" => fmi2False,
        //                 _ => return Err(format!("Invalid boolean literal: {literal}").into()),
        //             };
        //             assert_equal(&variable.name, expected, values[0])?;
        //         }
        //     }
        //     VariableType::String { start, .. } => {
        //         let mut values = [String::new()];
        //         call(fmu.getString(&[variable.valueReference], &mut values))?;
        //         if variable.initial == Some(Initial::Exact)
        //             && let Some(literal) = start
        //         {
        //             assert_equal(&variable.name, literal, &values[0])?;
        //         }
        //     }
        // }
    }

    call(fmu.terminate())
}

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
        logFile: None,
    };

    if let Err(e) = test_set_fmu_state(&factory) {
        println!("{}: {}", "error".red().bold(), e);
    }

    if let Err(e) = test_serialize_fmu_state(&factory) {
        println!("{}: {}", "error".red().bold(), e);
    }

    if let Err(e) = test_get_all_variables(&factory) {
        println!("{}: {}", "error".red().bold(), e);
    }

    ExitCode::SUCCESS
}
