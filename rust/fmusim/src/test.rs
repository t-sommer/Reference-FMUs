use std::process::ExitCode;

use fmi::model_description::FMIMajorVersion;

use crate::{
    TestArgs, prepare_fmu,
    test::{fmi2::smoke_test_fmi2, fmi3::smoke_test_fmi3},
    error
};

mod fmi2;
mod fmi3;

pub fn smoke_test(args: &TestArgs) -> ExitCode {
    let (_unzipdir, _xml_path, fmi_major_version) = match prepare_fmu(&args.fmu_file) {
        Ok(val) => val,
        Err(message) => { error!(message); },
    };

    match fmi_major_version {
        FMIMajorVersion::V2 => smoke_test_fmi2(args),
        FMIMajorVersion::V3 => smoke_test_fmi3(args),
    }
}
