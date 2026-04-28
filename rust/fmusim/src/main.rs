#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

mod cvode;

use clap::{Parser, ValueEnum};
use colored::Colorize;
use fmi::{
    model_description::{Causality, MajorVersion, ModelVariable, read_model_description},
    sim::{self, SimulationSettings, euler::ForwardEulerFactory},
    util::extract_fmu,
};
use fmi_schema::validate_model_description_against_xsd;
use plotly::{
    Configuration, Layout, Plot, Scatter,
    common::Line,
    layout::{Axis, GridPattern, LayoutGrid, Margin},
};
use std::{
    collections::HashMap,
    path::PathBuf,
    process::ExitCode,
};

use crate::cvode::CVodeSolverFactory;

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

#[derive(Parser)]
#[command(name = "fmusim", version, about = "FMU simulation tool")]
struct Args {
    /// Path to the FMU file
    filename: String,

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
    let args = Args::parse();

    // Extract FMU to temporary directory
    let unzipdir = match extract_fmu(&args.filename) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Failed to extract FMU: {e}");
            return ExitCode::FAILURE;
        }
    };

    let xml_path = unzipdir.path().join("modelDescription.xml");

    let model_description = match read_model_description(xml_path.as_path()) {
        Ok(desc) => desc,
        Err(e) => {
            eprintln!("Failed to read model description: {e}");
            return ExitCode::FAILURE;
        }
    };

    let fmi_major_version = model_description.majorVersion.clone() as i32;

    if let Err(validation_errors) =
        validate_model_description_against_xsd(&xml_path, fmi_major_version)
    {
        for error in validation_errors {
            eprintln!("Validation error: {}", error);
        }
        eprintln!("modelDescription.xml failed XSD schema validation");
        return ExitCode::FAILURE;
    }

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

    let output_variables: Vec<&ModelVariable> = if args.output_variable.is_empty() {
        model_description
            .modelVariables
            .iter()
            .filter(|v| v.causality == Causality::Output)
            .collect()
    } else {
        let variable_map: HashMap<&str, &ModelVariable> = model_description
            .modelVariables
            .iter()
            .map(|var| (var.name.as_str(), var))
            .collect();

        let mut output_variables = vec![];

        for variable_name in args.output_variable {
            if let Some(&variable) = variable_map.get(variable_name.as_str()) {
                output_variables.push(variable);
            } else {
                eprintln!("The requested output variable {variable_name:?} does not exist.");
                return ExitCode::FAILURE;
            }
        }

        output_variables
    };

    let settings = SimulationSettings {
        unzipdir: unzipdir.path(),
        model_description: &model_description,
        start_time,
        stop_time,
        set_stop_time: args.set_stop_time,
        output_interval,
        tolerance,
        start_values: args.start_values.clone(),
        output_variables,
        output_file: args.output_file.as_ref().map(|f| PathBuf::from(f)),
        log_fmi_calls: args.log_fmi_calls,
        input_file: args.input_file.as_ref().map(|f| PathBuf::from(f)),
        early_return_allowed: args.early_return_allowed,
        event_mode_used: args.event_mode_used,
        logging_on: args.logging_on,
    };

    let interface_type = match args.interface_type {
        Some(t) => t,
        None => {
            if let Some(_) = &model_description.coSimulation {
                InterfaceType::CoSimulation
            } else {
                InterfaceType::ModelExchange
            }
        }
    };

    let fixes_step_size = args.fixed_step_size.unwrap_or(output_interval);
    let start_time = std::time::Instant::now();

    let result = match &model_description.majorVersion {
        MajorVersion::V2 => {
            let mut sim_results =
                sim::fmi2::SimulationResult::new(settings.output_variables.clone());

            let result = match interface_type {
                InterfaceType::ModelExchange => match args.solver {
                    SolverType::Euler => sim::fmi2::simulate_me(
                        &settings,
                        &ForwardEulerFactory { fixes_step_size },
                        &mut sim_results,
                    ),
                    SolverType::Cvode => {
                        sim::fmi2::simulate_me(&settings, &CVodeSolverFactory, &mut sim_results)
                    }
                },
                InterfaceType::CoSimulation => sim::fmi2::simulate_cs(&settings, &mut sim_results),
            };

            if let Some(output_file) = settings.output_file.as_ref() {
                if let Err(e) = sim::fmi2::csv::write_csv(&sim_results, output_file) {
                    eprintln!("Failed to write output CSV file: {e}");
                    return ExitCode::FAILURE;
                }
            }

            if args.show_plot {
                sim::fmi2::plot::plot_result(&sim_results).show();
            }

            result
        }
        MajorVersion::V3 => {
            let mut sim_results =
                sim::fmi3::SimulationResult::new(settings.output_variables.clone());

            let result = match interface_type {
                InterfaceType::ModelExchange => match args.solver {
                    SolverType::Euler => sim::fmi3::simulate_me(
                        &settings,
                        &ForwardEulerFactory { fixes_step_size },
                        &mut sim_results,
                    ),
                    SolverType::Cvode => {
                        sim::fmi3::simulate_me(&settings, &CVodeSolverFactory, &mut sim_results)
                    }
                },
                InterfaceType::CoSimulation => sim::fmi3::simulate_cs(&settings, &mut sim_results),
            };

            if let Some(output_file) = settings.output_file.as_ref() {
                if let Err(e) = sim::fmi3::csv::write_csv(&sim_results, output_file) {
                    eprintln!("Failed to write output CSV file: {e}");
                    return ExitCode::FAILURE;
                }
            }

            if args.show_plot {
                sim::fmi3::plot::plot_result(&sim_results).show();
            }

            result
        }
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
