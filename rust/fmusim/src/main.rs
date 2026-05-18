#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod cvode;

use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::Colorize;
use fmi::{
    model_description::{self, FMIMajorVersion, peak_fmi_major_version},
    sim::{self, euler::ForwardEulerFactory},
    util::extract_fmu,
};
use fmi_rs_xsd::validate_model_description_against_xsd;
use std::{collections::HashMap, fs::File, path::PathBuf, process::ExitCode};

#[derive(ValueEnum, Clone, Debug)]
enum InterfaceType {
    /// Model Exchange
    #[value(name = "me")]
    ModelExchange,
    /// Co-Simulation
    #[value(name = "cs")]
    CoSimulation,
}

#[derive(ValueEnum, Clone, Debug)]
enum SolverType {
    #[value(name = "euler")]
    Euler,
    #[value(name = "cvode")]
    Cvode,
}

fn parse_start_value(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();

    if parts.len() != 2 {
        return Err(format!(
            "Invalid format {s:?}. Expected \"variable_name=value\"."
        ));
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

#[derive(Parser, Debug)]
#[command(name = "fmusim", version, about = "FMU simulation tool")]
#[command(propagate_version = true)]
struct Cli {
    /// Path to the FMU file
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Display information about an FMU
    Info(InfoArgs),
    /// Validate an FMU
    Validate(ValidateArgs),
    /// Simulate an FMU
    Simulate(SimulateArgs),
}

#[derive(Debug, Args)]
struct InfoArgs {
    /// Path to the FMU file
    fmu_file: String,
}

#[derive(Debug, Args)]
struct ValidateArgs {
    /// Path to the FMU file
    fmu_file: String,
}

#[derive(Debug, Args)]
struct SimulateArgs {
    /// Path to the FMU file
    fmu_file: String,

    /// Enable logging of FMI function calls
    #[arg(long)]
    log_fmi_calls: bool,

    /// Interval for sampling the output variables
    #[arg(long)]
    output_interval: Option<f64>,

    /// Start time for the simulation
    #[arg(long)]
    start_time: Option<f64>,

    /// Stop time for the simulation
    #[arg(long)]
    stop_time: Option<f64>,

    /// Set stop time explicitly
    #[arg(long)]
    set_stop_time: bool,

    /// Relative tolerance for the simulation
    #[arg(long)]
    tolerance: Option<f64>,

    /// CSV file to read the input from
    #[arg(long)]
    input_file: Option<String>,

    /// CSV file to store the output
    #[arg(long)]
    output_file: Option<String>,

    /// Plot of up to 8 output variables
    #[arg(long)]
    show_plot: bool,

    /// Set start values for variables (format: variable_name=value)
    #[arg(long = "start-value", value_parser = parse_start_value)]
    start_values: Vec<(String, String)>,

    /// Record a specific variable
    #[arg(long)]
    output_variable: Vec<String>,

    /// Allow early return
    #[arg(long)]
    early_return_allowed: bool,

    /// Use event mode
    #[arg(long)]
    event_mode_used: bool,

    /// Enable FMU looging
    #[arg(long)]
    logging_on: bool,

    /// Show statisics
    #[arg(long)]
    show_stats: bool,

    /// The interface type to use
    #[arg(long, value_enum)]
    interface_type: Option<InterfaceType>,

    /// Step size for fixed step solver (default: output interval)
    #[arg(long)]
    fixed_step_size: Option<f64>,

    /// The solver to integrate Model Exchange FMUs
    #[arg(long, value_enum, default_value_t = SolverType::Cvode)]
    solver: SolverType,
}

fn main() -> ExitCode {
    // Parse command line arguments
    let cli = Cli::parse();

    match &cli.command {
        Commands::Info(args) => info_fmu(args),
        Commands::Validate(args) => validate_fmu(args),
        Commands::Simulate(args) => simulate_fmu(args),
    }
}

/// Common logic to extract, detect version, and validate an FMU
fn prepare_fmu(
    fmu_path: &str,
) -> Result<(tempfile::TempDir, std::path::PathBuf, FMIMajorVersion), ExitCode> {
    let unzipdir = match extract_fmu(fmu_path) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Failed to extract FMU: {e}");
            return Err(ExitCode::FAILURE);
        }
    };

    let xml_path = unzipdir.path().join("modelDescription.xml");

    let fmi_major_version = match peak_fmi_major_version(&xml_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "{}: Failed to determine FMI version: {e}",
                "error".red().bold()
            );
            return Err(ExitCode::FAILURE);
        }
    };

    Ok((unzipdir, xml_path, fmi_major_version))
}

fn simulate_fmu(args: &SimulateArgs) -> ExitCode {
    let (unzipdir, xml_path, fmi_major_version) = match prepare_fmu(&args.fmu_file) {
        Ok(val) => val,
        Err(code) => return code,
    };

    let start_time = std::time::Instant::now();

    let result = if fmi_major_version == FMIMajorVersion::V2 {
        let model_description = match model_description::fmi2::ModelDescription::read(&xml_path) {
            Ok(md) => md,
            Err(e) => {
                eprintln!("Failed to parse modelDescription.xml: {e}");
                return ExitCode::FAILURE;
            }
        };

        let output_variables: Vec<&model_description::fmi2::ScalarVariable> = if args
            .output_variable
            .is_empty()
        {
            model_description
                .modelVariables
                .iter()
                .filter(|v| v.causality == model_description::fmi2::Causality::Output)
                .collect()
        } else {
            let variable_map: HashMap<&str, &model_description::fmi2::ScalarVariable> =
                model_description
                    .modelVariables
                    .iter()
                    .map(|var| (var.name.as_str(), var))
                    .collect();

            let mut output_variables = vec![];

            for variable_name in &args.output_variable {
                if let Some(&variable) = variable_map.get(variable_name.as_str()) {
                    output_variables.push(variable);
                } else {
                    eprintln!("The requested output variable {variable_name:?} does not exist.");
                    return ExitCode::FAILURE;
                }
            }

            output_variables
        };

        let (start_time, stop_time, _tolerance) =
            if let Some(default_experiment) = &model_description.defaultExperiment {
                let start_time: f64 = if let Some(v) = &default_experiment.startTime {
                    v.parse().unwrap()
                } else {
                    0.0
                };
                let stop_time: f64 = if let Some(v) = &default_experiment.stopTime {
                    v.parse().unwrap()
                } else {
                    start_time + 1.0
                };
                let tolerance: Option<f64> = default_experiment
                    .tolerance
                    .as_ref()
                    .map(|v| v.parse().unwrap());
                (start_time, stop_time, tolerance)
            } else {
                (0.0, 1.0, None)
            };

        let internal_step_size: Option<f64> = model_description
            .coSimulation
            .as_ref()
            .unwrap()
            .fixedInternalStepSize
            .as_ref()
            .map(|v| v.parse().unwrap());

        let start_time = args.start_time.unwrap_or(start_time);
        let stop_time = args.stop_time.unwrap_or(stop_time);
        let tolerance = args.tolerance;

        let output_interval = if let Some(v) = args.output_interval {
            v
        } else {
            if let Some(v) = internal_step_size {
                v
            } else {
                (stop_time - start_time) / 500.0
            }
        };

        let settings = sim::fmi2::SimulationSettings {
            unzipdir: unzipdir.path(),
            model_description: &model_description,
            start_time,
            stop_time,
            set_stop_time: args.set_stop_time,
            output_interval,
            tolerance,
            start_values: args.start_values.clone(),
            log_fmi_calls: args.log_fmi_calls,
            input_file: args.input_file.as_ref().map(|f| PathBuf::from(f)),
            early_return_allowed: args.early_return_allowed,
            event_mode_used: args.event_mode_used,
            logging_on: args.logging_on,
        };

        let interface_type = match &args.interface_type {
            Some(t) => t.clone(),
            None => {
                if let Some(_) = &model_description.coSimulation {
                    InterfaceType::CoSimulation
                } else {
                    InterfaceType::ModelExchange
                }
            }
        };

        let input = if let Some(path) = &args.input_file {
            let file = File::open(&path).expect("Failed to open input file");
            let trajectories = sim::fmi2::csv::read_csv(&file, &settings.model_description)
                .expect("Failed to read CSV");
            Some(sim::fmi2::input::StaticInput::new(trajectories))
        } else {
            None
        };

        let mut simulation_result = sim::fmi2::SimulationResult::new(output_variables.clone());

        let mut recorder = sim::fmi2::recorder::Recorder::new(&mut simulation_result);

        let fixes_step_size = args.fixed_step_size.unwrap_or(output_interval);

        let result = match interface_type {
            InterfaceType::ModelExchange => match args.solver {
                SolverType::Euler => sim::fmi2::simulate_me(
                    &settings,
                    &ForwardEulerFactory { fixes_step_size },
                    input.as_ref(),
                    &mut recorder,
                ),
                SolverType::Cvode => sim::fmi2::simulate_me(
                    &settings,
                    &cvode::CVodeSolverFactory,
                    input.as_ref(),
                    &mut recorder,
                ),
            },
            InterfaceType::CoSimulation => {
                sim::fmi2::simulate_cs(&settings, input.as_ref(), &mut recorder)
            }
        };

        if let Some(output_file) = args.output_file.as_ref() {
            if let Err(e) = sim::fmi2::csv::write_csv(&simulation_result, output_file) {
                eprintln!("Failed to write output CSV file: {e}");
                return ExitCode::FAILURE;
            }
        }

        if args.show_plot {
            sim::fmi2::plot::plot_result(&simulation_result).show();
        }

        result
    } else {
        let model_description = match model_description::fmi3::ModelDescription::read(&xml_path) {
            Ok(md) => md,
            Err(e) => {
                eprintln!("Failed to parse modelDescription.xml: {e}");
                return ExitCode::FAILURE;
            }
        };

        let output_variables: Vec<&model_description::fmi3::ModelVariable> = if args
            .output_variable
            .is_empty()
        {
            model_description
                .modelVariables
                .iter()
                .filter(|v| v.causality == model_description::fmi3::Causality::Output)
                .collect()
        } else {
            let variable_map: HashMap<&str, &model_description::fmi3::ModelVariable> =
                model_description
                    .modelVariables
                    .iter()
                    .map(|var| (var.name.as_str(), var))
                    .collect();

            let mut output_variables = vec![];

            for variable_name in &args.output_variable {
                if let Some(&variable) = variable_map.get(variable_name.as_str()) {
                    output_variables.push(variable);
                } else {
                    eprintln!("The requested output variable {variable_name:?} does not exist.");
                    return ExitCode::FAILURE;
                }
            }

            output_variables
        };

        let (start_time, stop_time, _tolerance) =
            if let Some(default_experiment) = &model_description.defaultExperiment {
                let start_time: f64 = if let Some(v) = &default_experiment.startTime {
                    v.parse().unwrap()
                } else {
                    0.0
                };
                let stop_time: f64 = if let Some(v) = &default_experiment.stopTime {
                    v.parse().unwrap()
                } else {
                    start_time + 1.0
                };
                let tolerance: Option<f64> = default_experiment
                    .tolerance
                    .as_ref()
                    .map(|v| v.parse().unwrap());
                (start_time, stop_time, tolerance)
            } else {
                (0.0, 1.0, None)
            };

        let internal_step_size: Option<f64> = model_description
            .coSimulation
            .as_ref()
            .unwrap()
            .fixedInternalStepSize
            .as_ref()
            .map(|v| v.parse().unwrap());

        let start_time = args.start_time.unwrap_or(start_time);
        let stop_time = args.stop_time.unwrap_or(stop_time);
        let tolerance = args.tolerance;

        let output_interval = if let Some(v) = args.output_interval {
            v
        } else {
            if let Some(v) = internal_step_size {
                v
            } else {
                (stop_time - start_time) / 500.0
            }
        };

        let settings = sim::fmi3::SimulationSettings {
            unzipdir: unzipdir.path(),
            model_description: &model_description,
            start_time,
            stop_time,
            set_stop_time: args.set_stop_time,
            output_interval,
            tolerance,
            start_values: args.start_values.clone(),
            log_fmi_calls: args.log_fmi_calls,
            input_file: args.input_file.as_ref().map(|f| PathBuf::from(f)),
            early_return_allowed: args.early_return_allowed,
            event_mode_used: args.event_mode_used,
            logging_on: args.logging_on,
        };

        let interface_type = match &args.interface_type {
            Some(t) => t.clone(),
            None => {
                if let Some(_) = &model_description.coSimulation {
                    InterfaceType::CoSimulation
                } else {
                    InterfaceType::ModelExchange
                }
            }
        };

        let input = if let Some(path) = &args.input_file {
            let file = File::open(&path).expect("Failed to open input file");
            let trajectories = sim::fmi3::csv::read_csv(&file, &settings.model_description)
                .expect("Failed to read CSV");
            Some(sim::fmi3::input::StaticInput::new(trajectories))
        } else {
            None
        };

        let mut simulation_result = sim::fmi3::SimulationResult::new(output_variables.clone());

        let mut recorder = sim::fmi3::recorder::Recorder::new(&mut simulation_result);

        let fixes_step_size = args.fixed_step_size.unwrap_or(output_interval);

        let result = match interface_type {
            InterfaceType::ModelExchange => match args.solver {
                SolverType::Euler => sim::fmi3::simulate_me(
                    &settings,
                    &ForwardEulerFactory { fixes_step_size },
                    input.as_ref(),
                    &mut recorder,
                ),
                SolverType::Cvode => sim::fmi3::simulate_me(
                    &settings,
                    &cvode::CVodeSolverFactory,
                    input.as_ref(),
                    &mut recorder,
                ),
            },
            InterfaceType::CoSimulation => {
                sim::fmi3::simulate_cs(&settings, input.as_ref(), &mut recorder)
            }
        };

        if let Some(output_file) = args.output_file.as_ref() {
            if let Err(e) = sim::fmi3::csv::write_csv(&simulation_result, output_file) {
                eprintln!("Failed to write output CSV file: {e}");
                return ExitCode::FAILURE;
            }
        }

        if args.show_plot {
            sim::fmi3::plot::plot_result(&simulation_result).show();
        }

        result
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

fn info_fmu(args: &InfoArgs) -> ExitCode {
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
            let model_description = match model_description::fmi2::ModelDescription::read(&xml_path)
            {
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
            let model_description = match model_description::fmi3::ModelDescription::read(&xml_path)
            {
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

fn validate_fmu(args: &ValidateArgs) -> ExitCode {
    println!("{} {}", "Validating".green().bold(), args.fmu_file);

    let (_unzipdir, xml_path, fmi_major_version) = match prepare_fmu(&args.fmu_file) {
        Ok(val) => val,
        Err(code) => return code,
    };

    let mut problems = validate_model_description_against_xsd(&xml_path, fmi_major_version as i32);

    match &fmi_major_version {
        FMIMajorVersion::V2 => {
            let model_description = match model_description::fmi2::ModelDescription::read(&xml_path)
            {
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
            println!("{:#?}", model_description.typeDefinitions);
        }
        FMIMajorVersion::V3 => {
            let model_description = match model_description::fmi3::ModelDescription::read(&xml_path)
            {
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
            println!("{:#?}", model_description.typeDefinitions);
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
