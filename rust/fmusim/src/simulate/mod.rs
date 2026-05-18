use std::process::ExitCode;

use colored::Colorize;
use fmi::model_description::FMIMajorVersion;

use crate::{SimulateArgs, prepare_fmu};

pub mod fmi2;
pub mod fmi3;

pub fn simulate_fmu(args: &SimulateArgs) -> ExitCode {
    let (unzipdir, xml_path, fmi_major_version) = match prepare_fmu(&args.fmu_file) {
        Ok(val) => val,
        Err(code) => return code,
    };

    let start_time = std::time::Instant::now();

    let result = match fmi_major_version {
        FMIMajorVersion::V2 => crate::simulate::fmi2::simulate_fmu(args, &unzipdir, &xml_path),
        FMIMajorVersion::V3 => crate::simulate::fmi3::simulate_fmu(args, &unzipdir, &xml_path),
    };

    let elapsed_time = start_time.elapsed();

    if args.show_stats {
        eprintln!("Simulation took {:.2?}.", elapsed_time);
    }

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}: {}", "error".red().bold(), e);
            ExitCode::FAILURE
        }
    }
}
