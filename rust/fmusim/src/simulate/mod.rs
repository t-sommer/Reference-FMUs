use std::{fs::read_to_string, process::ExitCode};

use fmi::model_description::FMIMajorVersion;

use crate::{SimulateArgs, SimulateConfigArgs, error, prepare_fmu};

pub mod fmi2;
pub mod fmi3;

pub fn simulate_fmu(args: &SimulateArgs) -> ExitCode {
    if args.fmu_file.is_empty() {
        error!("No FMU file specified.");
    }

    let (unzipdir, xml_path, fmi_major_version) = match prepare_fmu(&args.fmu_file) {
        Ok(val) => val,
        Err(message) => {
            error!(message);
        }
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
            error!(e);
        }
    }
}

pub fn simulate_config(args: &SimulateConfigArgs) -> ExitCode {
    match read_to_string(&args.config_path) {
        Ok(content) => match toml::from_str::<SimulateArgs>(&content) {
            Ok(toml_args) => simulate_fmu(&toml_args),
            Err(e) => {
                error!(format!(
                    "Failed to parse config file {}: {e}",
                    &args.config_path
                ));
            }
        },
        Err(e) => {
            error!(format!(
                "Failed to read config file {}: {e}",
                &args.config_path
            ));
        }
    }
}
