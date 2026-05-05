#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod euler;
pub mod fmi2;
pub mod fmi3;

use crate::model_description::ModelDescription;
use std::path::{Path, PathBuf};

pub struct SimulationSettings<'a> {
    pub unzipdir: &'a Path,
    pub model_description: &'a ModelDescription,
    pub start_time: f64,
    pub stop_time: f64,
    pub logging_on: bool,
    pub set_stop_time: bool,
    pub output_interval: f64,
    pub tolerance: Option<f64>,
    pub start_values: Vec<(String, String)>,
    pub output_file: Option<PathBuf>,
    pub log_fmi_calls: bool,
    pub input_file: Option<PathBuf>,
    pub early_return_allowed: bool,
    pub event_mode_used: bool,
}

type Error = Box<dyn std::error::Error>;

pub type SetTimeFn<'a> = Box<dyn Fn(f64) -> Result<(), Error> + 'a>;
pub type SetContinuousInputsFn<'a> = Box<dyn Fn(f64) -> Result<(), Error> + 'a>;
pub type GetEventIndicatorsFn<'a> = Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>;
pub type GetContinuousStatesFn<'a> = Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>;
pub type GetNominalsOfContinuousStatesFn<'a> = Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>;
pub type GetContinuousStateDerivativesFn<'a> = Box<dyn Fn(&mut [f64]) -> Result<(), Error> + 'a>;
pub type GetDirectionalDerivativeFn<'a> =
    Box<dyn Fn(&[u32], &[u32], &[f64], &mut [f64]) -> Result<(), Error> + 'a>;
pub type SetContinuousStatesFn<'a> = Box<dyn Fn(&[f64]) -> Result<(), Error> + 'a>;

pub trait Solver {
    fn reset(&mut self, time: f64) -> Result<(), Error>;
    fn step(&mut self, next_time: f64) -> Result<(f64, bool), Error>;
}
pub trait SolverFactory {
    fn create<'a>(
        &self,
        start_time: f64,
        nx: usize,
        nz: usize,
        rtol: f64,
        unknowns: Vec<u32>,
        knowns: Vec<u32>,
        set_time: SetTimeFn<'a>,
        set_continuous_inputs: SetContinuousInputsFn<'a>,
        get_event_indicators: GetEventIndicatorsFn<'a>,
        get_continuous_states: GetContinuousStatesFn<'a>,
        get_nominals_of_continuous_states: GetNominalsOfContinuousStatesFn<'a>,
        get_continuous_state_derivatives: GetContinuousStateDerivativesFn<'a>,
        get_directional_derivative: Option<GetDirectionalDerivativeFn<'a>>,
        set_continuous_states: SetContinuousStatesFn<'a>,
    ) -> Result<Box<dyn Solver + 'a>, Error>;
}
