#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use std::path::PathBuf;
pub mod fmi2;
pub mod fmi3;


pub struct SimulationSettings {
    pub instance_name: String,
    pub start_time: f64,
    pub stop_time: f64,
    pub output_interval: f64,
    pub tolerance: Option<f64>,
    pub start_values: Vec<(String, String)>,
    pub output_file: Option<PathBuf>,
    pub log_fmi_calls: bool,
    pub input_file: Option<PathBuf>,
}