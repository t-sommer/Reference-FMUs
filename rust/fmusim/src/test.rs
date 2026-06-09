use colored::Colorize;
use std::process::ExitCode;

use fmi::{
    fmi2::{CS, FMU2, types::fmi2FMUstate},
    model_description::fmi2::ModelDescription,
    util::extract_fmu,
};

use crate::{
    TestArgs,
    test::fmi2::{test_serialize_fmu_state, test_set_fmu_state},
};

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

    let factory = fmi2::FMU2Factory {
        model_description,
        unzipdir: unzipdir.path().to_path_buf(),
        visible: false,
        loggingOn: false,
        logCalls: args.log_fmi_calls,
        printCalls: true,
        logMessages: true,
        printMessages: true,
        provideMemoryManagementFunctions: true,
    };

    let _ = test_set_fmu_state(&factory);

    let _ = test_serialize_fmu_state(&factory);

    ExitCode::SUCCESS
}
