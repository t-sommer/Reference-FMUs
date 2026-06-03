#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::too_many_arguments
)]

pub mod euler;
pub mod fmi2;
pub mod fmi3;

use approx::relative_eq;

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

/// Approximate equality using both the absolute difference and relative based comparisons.
pub fn relative_eq(lhs: f64, rhs: f64) -> bool {
    relative_eq!(lhs, rhs)
}

/// Greater or approximate equality using both the absolute difference and relative based comparisons.
pub fn relative_ge(lhs: f64, rhs: f64) -> bool {
    lhs > rhs || relative_eq(lhs, rhs)
}

/// Less or approximate equality using both the absolute difference and relative based comparisons.
pub fn relative_le(lhs: f64, rhs: f64) -> bool {
    lhs < rhs || relative_eq(lhs, rhs)
}

/// Less than and not approximate equality using both the absolute difference and relative based comparisons.
pub fn relative_lt(lhs: f64, rhs: f64) -> bool {
    lhs < rhs && !relative_eq(lhs, rhs)
}

/// Greater than and not approximate equality using both the absolute difference and relative based comparisons.
pub fn relative_gt(lhs: f64, rhs: f64) -> bool {
    lhs > rhs && !relative_eq(lhs, rhs)
}
