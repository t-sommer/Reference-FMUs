#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod cvode;
mod info;
mod simulate;
mod validate;

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
        Commands::Info(args) => info::info_fmu(args),
        Commands::Validate(args) => validate::validate_fmu(args),
        Commands::Simulate(args) => simulate::simulate_fmu(args),
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
