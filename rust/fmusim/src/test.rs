use colored::Colorize;
use std::process::ExitCode;

use fmi::{
    fmi2::{CS, FMU2, types::fmi2FMUstate},
    model_description::{FMIMajorVersion, fmi2::ModelDescription},
    util::extract_fmu,
};

use crate::{
    TestArgs, prepare_fmu,
    test::{
        fmi2::{
            smoke_test_fmi2, test_get_all_variables, test_serialize_fmu_state, test_set_fmu_state,
        },
        fmi3::{FMUFactory, smoke_test_fmi3},
    },
};

mod fmi2;
mod fmi3;

pub fn smoke_test(args: &TestArgs) -> ExitCode {
    let (unzipdir, xml_path, fmi_major_version) = match prepare_fmu(&args.fmu_file) {
        Ok(val) => val,
        Err(code) => return code,
    };

    match fmi_major_version {
        FMIMajorVersion::V2 => smoke_test_fmi2(args),
        FMIMajorVersion::V3 => smoke_test_fmi3(args),
    }
}
