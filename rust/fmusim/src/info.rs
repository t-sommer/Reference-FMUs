use std::process::ExitCode;

use colored::Colorize;
use fmi::model_description::FMIMajorVersion;

use crate::{InfoArgs, prepare_fmu};

pub fn info_fmu(args: &InfoArgs) -> ExitCode {
    let (unzipdir, xml_path, fmi_major_version) = match prepare_fmu(&args.fmu_file) {
        Ok(val) => val,
        Err(code) => return code,
    };

    let entries = std::fs::read_dir(unzipdir.path().join("binaries")).unwrap();

    let mut platform_dirs: Vec<String> = vec![];

    if unzipdir.path().join("sources").is_dir() {
        platform_dirs.push("c-code".to_string());
    }

    platform_dirs.extend(
        entries
            .filter_map(|res| res.ok()) // Filter out entries that failed to read
            .filter(|entry| {
                // Use file_type() to check if the entry is a directory
                entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
            })
            .filter_map(|entry| {
                // Convert the OsString name into a String
                // This returns None if the directory name is not valid UTF-8
                entry.file_name().into_string().ok()
            }),
    );

    match fmi_major_version {
        FMIMajorVersion::V2 => {
            let model_description =
                match fmi::model_description::fmi2::ModelDescription::read(&xml_path) {
                    Ok(md) => md,
                    Err(e) => {
                        eprintln!("Failed to parse modelDescription.xml: {e}");
                        return ExitCode::FAILURE;
                    }
                };

            println!("{}", "Model Information".bold());
            println!();
            println!("FMI Version:       2.0");
            println!("Model Name:        {}", model_description.modelName);
            println!("Platforms:         {}", platform_dirs.join(", "));
            println!("Continuous States: {}", model_description.derivatives.len());
            println!(
                "Event Indicators:  {}",
                model_description.numberOfEventIndicators
            );
            println!(
                "Model Variables:   {}",
                model_description.modelVariables.len()
            );
            println!(
                "Generation Date:   {}",
                model_description.generationDateAndTime.unwrap_or_default()
            );
            println!(
                "Generation Tool:   {}",
                model_description.generationTool.unwrap_or_default()
            );
            println!(
                "Description:       {}",
                model_description.description.unwrap_or_default()
            );
            println!();
            println!("{}", "Model Variables".bold());
            println!();

            let terminal_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(120);

            let name_width = model_description
                .modelVariables
                .iter()
                .map(|v| v.name.len())
                .max() // Get the maximum length
                .unwrap_or(4); // Default to 4 if no variables or names are empty

            let description_width = terminal_width.saturating_sub(name_width + 3);

            let header = format!(
                "{:<nw$} │ {:<dw$}",
                "Name".bold(),
                "Description".bold(),
                nw = name_width,
                dw = description_width
            );
            println!("{}", header);

            println!(
                "{}─┼─{}",
                "─".repeat(name_width),
                "─".repeat(description_width)
            );

            for variable in &model_description.modelVariables {
                println!(
                    "{:<nw$} │ {:<dw$}",
                    variable.name,
                    variable.description.as_deref().unwrap_or_default(),
                    nw = name_width,
                    dw = description_width
                );
            }
        }
        FMIMajorVersion::V3 => {
            let model_description =
                match fmi::model_description::fmi3::ModelDescription::read(&xml_path) {
                    Ok(md) => md,
                    Err(e) => {
                        eprintln!("Failed to parse modelDescription.xml: {e}");
                        return ExitCode::FAILURE;
                    }
                };

            println!("{}", "Model Information".bold());
            println!();
            println!("FMI Version:       {}", model_description.fmiVersion);
            println!("Model Name:        {}", model_description.modelName);
            println!("Platforms:         {}", platform_dirs.join(", "));
            println!("Continuous States: {}", model_description.derivatives.len());
            println!(
                "Event Indicators:  {}",
                model_description.eventIndicators.len()
            );
            println!(
                "Model Variables:   {}",
                model_description.modelVariables.len()
            );
            println!(
                "Generation Date:   {}",
                model_description.generationDateAndTime.unwrap_or_default()
            );
            println!(
                "Generation Tool:   {}",
                model_description.generationTool.unwrap_or_default()
            );
            println!(
                "Description:       {}",
                model_description.description.unwrap_or_default()
            );
            println!();
            println!("{}", "Model Variables".bold());
            println!();

            let terminal_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(120);

            let name_width = model_description
                .modelVariables
                .iter()
                .map(|v| v.name.len())
                .max() // Get the maximum length
                .unwrap_or(4); // Default to 4 if no variables or names are empty

            let description_width = terminal_width.saturating_sub(name_width + 3);

            let header = format!(
                "{:<nw$} │ {:<dw$}",
                "Name".bold(),
                "Description".bold(),
                nw = name_width,
                dw = description_width
            );
            println!("{}", header);

            println!(
                "{}─┼─{}",
                "─".repeat(name_width),
                "─".repeat(description_width)
            );

            for variable in &model_description.modelVariables {
                println!(
                    "{:<nw$} │ {:<dw$}",
                    variable.name,
                    variable.description.as_deref().unwrap_or_default(),
                    nw = name_width,
                    dw = description_width
                );
            }
        }
    }

    ExitCode::SUCCESS
}
