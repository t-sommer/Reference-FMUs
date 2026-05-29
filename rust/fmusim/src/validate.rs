use std::process::ExitCode;

use colored::Colorize;
use fmi::model_description::FMIMajorVersion;
use fmi_rs_xsd::validate_model_description_against_xsd;

use crate::{ValidateArgs, prepare_fmu};

pub fn validate_fmu(args: &ValidateArgs) -> ExitCode {
    println!("{}", "    Validating model description against XML schema".green().bold());
    
    let (_unzipdir, xml_path, fmi_major_version) = match prepare_fmu(&args.fmu_file) {
        Ok(val) => val,
        Err(code) => return code,
    };
    
    let problems = validate_model_description_against_xsd(&xml_path, fmi_major_version as i32);
    
    for problem in problems.iter() {
        println!("{}: {}", "error".red().bold(), problem);
    }
    
    println!("{}", "    Validating model description".green().bold());

    let mut problems = vec![]; // validate_model_description_against_xsd(&xml_path, fmi_major_version as i32);

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

    let text = std::fs::read_to_string(xml_path).unwrap();

    let opt = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..roxmltree::ParsingOptions::default()
    };

    let doc = roxmltree::Document::parse_with_options(&text, opt).unwrap();

    let terminal_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(120);

    let max_width = terminal_width - 8;

    for problem in problems.iter() {
        println!("{}: {}", "error".red().bold(), problem.message.bold());

        if let Some(range) = problem.range.last() {
            let start_pos = doc.text_pos_at(range.start);
            println!("     {} modelDescription.xml:{}:{}", "-->".cyan().bold(), start_pos.row, start_pos.col);            
        }

        for (j, range) in problem.range.iter().enumerate() {
            let start_pos = doc.text_pos_at(range.start);
            let end_pos = doc.text_pos_at(range.end);
            let start_line = (start_pos.row - 1) as usize;
            let end_line = (end_pos.row - 1) as usize;

            if j == 0 {
                println!("      {}", "|".cyan().bold());
            } else {
                println!("  {}", "...".cyan().bold());
            }

            for (i, line) in text.lines().enumerate() {

                if i >= start_line && i <= end_line {

                    let text = if line.len() > max_width {
                        let limit = max_width.saturating_sub(3);
                        format!("{line:.limit$}{}", "...".cyan().bold())
                    } else {
                        line.to_string()
                    };

                    let prefix = if i == start_line {
                        format!("{:>5} {} ", (i + 1).to_string().cyan().bold(), "|".cyan().bold())
                    } else {
                        format!("      {} ", "|".cyan().bold())
                    };
                    println!("{}{}", prefix, text);
                }
            }
            println!("      {}", "|".cyan().bold());
        }
    }

    if problems.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
