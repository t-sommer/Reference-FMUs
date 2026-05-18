use std::process::ExitCode;

use colored::Colorize;
use fmi::model_description::FMIMajorVersion;
use fmi_rs_xsd::validate_model_description_against_xsd;

use crate::{ValidateArgs, prepare_fmu};

pub fn validate_fmu(args: &ValidateArgs) -> ExitCode {
    println!("{} {}", "Validating".green().bold(), args.fmu_file);

    let (_unzipdir, xml_path, fmi_major_version) = match prepare_fmu(&args.fmu_file) {
        Ok(val) => val,
        Err(code) => return code,
    };

    let mut problems = validate_model_description_against_xsd(&xml_path, fmi_major_version as i32);

    match &fmi_major_version {
        FMIMajorVersion::V2 => {
            let model_description =
                match fmi::model_description::fmi2::ModelDescription::read(&xml_path) {
                    Ok(md) => md,
                    Err(e) => {
                        eprintln!(
                            "{}: Failed to parse modelDescription.xml: {e}",
                            "error".red().bold()
                        );
                        return ExitCode::FAILURE;
                    }
                };
            problems.extend(model_description.validate());
        }
        FMIMajorVersion::V3 => {
            let model_description =
                match fmi::model_description::fmi3::ModelDescription::read(&xml_path) {
                    Ok(md) => md,
                    Err(e) => {
                        eprintln!(
                            "{}: Failed to parse modelDescription.xml: {e}",
                            "error".red().bold()
                        );
                        return ExitCode::FAILURE;
                    }
                };
            problems.extend(model_description.validate());
        }
    };

    for problem in problems.iter() {
        println!("{}: {}", "error".red().bold(), problem);
    }

    if problems.is_empty() {
        println!("{}", "Validation successful".green().bold());
        ExitCode::SUCCESS
    } else {
        println!(
            "{}: {} problems have been found.",
            "Validation failed".red().bold(),
            problems.len()
        );
        ExitCode::FAILURE
    }
}
