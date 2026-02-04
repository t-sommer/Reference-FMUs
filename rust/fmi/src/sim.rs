#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
pub mod fmi2;
pub mod fmi3;

use crate::{fmi2::FMU2, input::CSVInput, model_description::{Causality, ModelDescription, ModelVariable, Variability, VariableType, read_model_description}, recorder::{FMI2Recorder, Recorder}, sim::fmi3::{parse_variable_value, set_variable_value}, types::fmiStatus::{self, fmiOK, fmiWarning}, util::extract_fmu};
use std::{collections::HashMap, error::Error, fs::File, io::{Write, stdout}, path::PathBuf, process::ExitCode};
use crate::{SHARED_LIBRARY_EXTENSION, fmi3::{FMU3, PLATFORM_TUPLE}};
use libloading::Library;

pub struct SimulationSettings {
    pub start_time: f64,
    pub stop_time: f64,
    pub output_interval: f64,
    pub tolerance: Option<f64>,
    pub start_values: Vec<(String, String)>,
    pub output_file: Option<PathBuf>,
}