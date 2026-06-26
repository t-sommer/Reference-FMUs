#![allow(non_camel_case_types, non_snake_case, unused_variables)]

#[cfg(feature = "fmi2")]
use fmi_rs::fmi2::types::*;
// #[cfg(not(feature = "fmi2"))]
use fmi_rs::{fmi3::types::*, types::fmiStatus};
use std::any::type_name_of_val;
use std::os::raw::c_void;
use std::ptr::null_mut;

use fmi_export::{BaseModel, ModelMode, Solver, ValueReference};
use serde::{Deserialize, Serialize};

const FIXED_STEP_SIZE: f64 = 0.1;

type LogError = dyn Fn(&str);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ModelData {
    solver: Option<Solver>,
    mode: ModelMode,
    eventModeUsed: bool,
    n_steps: u64,
    time: f64,
    x: f64,
    der_x: f64,
    k: f64,
}

impl ModelData {
    fn default(solver: Option<Solver>) -> Self {
        ModelData {
            solver: solver,
            mode: ModelMode::Instantiated,
            eventModeUsed: false,
            n_steps: 0,
            time: 0.0,
            x: 1.0,
            der_x: 0.0,
            k: 1.0,
        }
    }
}

struct ModelInstance {
    data: ModelData,
    logError: Box<LogError>,
}

impl BaseModel for ModelInstance {
    fn log_error(&self, message: &str) {
        (self.logError)(message);
    }

    fn time(&self) -> f64 {
        self.data.time
    }

    fn set_time(&mut self, time: f64) -> fmiStatus {
        self.data.time = time;
        fmiStatus::fmiOK
    }

    fn solver(&mut self) -> Option<Solver> {
        self.data.solver.take()
    }

    fn set_solver(&mut self, solver: Solver) {
        self.data.solver = Some(solver);
    }

    fn set_mode(&mut self, mode: ModelMode) {
        self.data.mode = mode;
    }

    fn get_event_indicators(&self, z: &mut [f64]) -> fmiStatus {
        if z.len() != 0 {
            (self.logError)("Event indicators array must have length 0");
            return fmiStatus::fmiError;
        }
        fmiStatus::fmiOK
    }

    fn get_continuous_states(&self, x: &mut [f64]) -> fmiStatus {
        if x.len() != 1 {
            (self.logError)("Continuous states array must have length 1");
            return fmiStatus::fmiError;
        }
        x[0] = self.data.x;
        fmiStatus::fmiOK
    }

    fn get_nominals_of_continuous_states(&self, nominals: &mut [f64]) -> fmiStatus {
        if nominals.len() != 1 {
            (self.logError)("Nominals array must have length 1");
            return fmiStatus::fmiError;
        }
        nominals[0] = 1.0;
        fmiStatus::fmiOK
    }

    fn get_number_of_continuous_states(&self) -> usize {
        1
    }

    fn set_continuous_states(&mut self, x: &[f64]) -> fmiStatus {
        if x.len() != 1 {
            (self.logError)("Continuous states array must have length 1");
            return fmiStatus::fmiError;
        }
        self.data.x = x[0];
        fmiStatus::fmiOK
    }

    fn get_continuous_state_derivatives(&self, der_x: &mut [f64]) -> fmiStatus {
        if der_x.len() != 1 {
            (self.logError)("Continuous state derivatives array must have length 1");
            return fmiStatus::fmiError;
        }
        der_x[0] = -self.data.k * self.data.x;
        fmiStatus::fmiOK
    }

    fn get_Float64(&self, value_reference: u32, value: &mut f64) -> fmiStatus {
        match ValueReference::try_from(value_reference) {
            Ok(ValueReference::time) => *value = self.data.time,
            Ok(ValueReference::x) => *value = self.data.x,
            Ok(ValueReference::der_x) => *value = self.data.der_x,
            Ok(ValueReference::k) => *value = self.data.k,
            Err(_) => {
                let message = format!(
                    "Unknown value reference for type Float64: {}",
                    value_reference
                );
                (self.logError)(&message);
                return fmiStatus::fmiError;
            }
        }
        fmiStatus::fmiOK
    }
}

#[derive(Debug, ValueReference)]
#[repr(u32)]
enum ValueReference {
    time = 0,
    x = 1,
    der_x = 2,
    k = 3,
}

impl ModelInstance {
    fn new(solver: Option<Solver>, logMessage: Box<LogError>) -> Self {
        ModelInstance {
            data: ModelData::default(solver),
            logError: logMessage,
        }
    }
}

// Include shared FMI3 implementation
// This compiles common functions directly into this DLL
#[cfg(feature = "fmi2")]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../fmi-export/src/fmi2.rs"
));

#[cfg(not(feature = "fmi2"))]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../fmi-export/src/fmi3.rs"
));
