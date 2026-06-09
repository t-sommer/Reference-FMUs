use colored::Colorize;
use std::process::ExitCode;

use fmi::{
    fmi2::{CS, FMU2, types::fmi2FMUstate},
    model_description::fmi2::ModelDescription,
    util::extract_fmu,
};

use crate::{TestArgs, test::fmi2::test_set_fmu_state};

mod fmi2;

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

pub fn smoke_test(args: &TestArgs) -> ExitCode {
    let unzipdir = bail!(extract_fmu(&args.fmu_file));

    let model_description = bail!(ModelDescription::from_path(
        &unzipdir.path().join("modelDescription.xml")
    ));

    // let modelIdentifier = &model_description.coSimulation.unwrap().modelIdentifier;
    // let instanceName = "instance";
    // let guid = &model_description.guid;
    // let logCalls = true;
    // let printCalls = true;
    // let logMessages = true;
    // let printMessages = true;
    // let provideMemoryManagementFunctions = true;

    // let fmu = FMU2::<CS>::new(unzipdir.path(), modelIdentifier, instanceName, guid, false, false, logCalls, printCalls, logMessages, printMessages, provideMemoryManagementFunctions).unwrap();

    // let mut fmu_state: fmi2FMUstate = std::ptr::null_mut();

    // fmu.getFMUstate(&mut fmu_state);
    // fmu.setFMUstate(fmu_state);

    test_set_fmu_state(&model_description, unzipdir.path());

    ExitCode::SUCCESS
}
