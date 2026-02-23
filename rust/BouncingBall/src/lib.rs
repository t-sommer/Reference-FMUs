#![allow(non_camel_case_types, non_snake_case, unused_variables)]

#[cfg(feature = "fmi2")]
use fmi::fmi2::types::*;
// #[cfg(not(feature = "fmi2"))]
use fmi::{fmi3::types::*, types::fmiStatus};
use std::any::type_name_of_val;
use std::os::raw::c_void;
use std::ptr::null_mut;
use fmi_export::{BaseModel, InterfaceType, ModelMode, ValueReference};
use serde::{Deserialize, Serialize};

const FIXED_STEP_SIZE: f64 = 1e-3;

type LogError = dyn Fn(&str);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ModelData {
    interfaceType: InterfaceType,
    mode: ModelMode,
    eventModeUsed: bool,
    n_steps: u64,
    time: f64,
    h: f64,
    v: f64,
    e: f64,
    g: f64,
    v_min: f64,
}

impl ModelData {

    fn default(interfaceType: InterfaceType) -> Self {
        ModelData {
            interfaceType: interfaceType,
            mode: ModelMode::Instantiated,
            eventModeUsed: false,
            n_steps: 0,
            time: 0.0,
            h: 1.0,  // initial height
            v: 0.0,  // initial velocity
            e: 0.8,  // coefficient of restitution
            g: -9.81, // gravity
            v_min: 0.01, // minimum velocity threshold
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
}

#[derive(Debug, ValueReference)]
#[repr(u32)]
enum ValueReference {
    time = 0,
    h = 1,
    der_h = 2,
    v = 3,
    der_v = 4,
    g = 5,
    e = 6,
    v_min = 7,
}

impl ModelInstance {

    fn new(interfaceType: InterfaceType, logMessage: Box<LogError>) -> Self {
        ModelInstance {
            data: ModelData::default(interfaceType),
            logError: logMessage,
        }
    }

    fn do_fixed_step(&mut self) {

        let mut x = [0.0; 2];
        let mut der_x = [0.0; 2];
        let mut z = [0.0];
        let mut pre_z = [0.0];
        
        self.get_event_indicators(&mut pre_z);
        self.get_continuous_states(&mut x);
        self.get_continuous_state_derivatives(&mut der_x);

        for i in 0..x.len() {
            x[i] += der_x[i] * FIXED_STEP_SIZE;
        }

        self.data.n_steps += 1;
        self.data.time = self.data.n_steps as f64 * FIXED_STEP_SIZE;

        self.set_continuous_states(&x);

        self.get_event_indicators(&mut z);

        if pre_z[0] < 0.0 && z[0] >= 0.0 || pre_z[0] > 0.0 && z[0] <= 0.0 {
            self.update_discrete_states();
        }
    }

    fn get_event_indicators(&self, z: &mut [f64]) -> fmiStatus {
        if z.len() != 1 {
            self.log_error("Event indicators array must have length 1");
            return fmiStatus::fmiError;
        }
        z[0] = self.data.h;
        fmiStatus::fmiOK
    }

    fn get_continuous_states(&self, x: &mut [f64]) -> fmiStatus {
        if x.len() != 2 {
            self.log_error("Continuous states array must have length 2");
            return fmiStatus::fmiError;
        }
        x[0] = self.data.h;
        x[1] = self.data.v;
        fmiStatus::fmiOK
    }

    fn get_nominals_of_continuous_states(&self, nominals: &mut [f64]) -> fmiStatus {
        if nominals.len() != 2 {
            self.log_error("Nominals array must have length 2");
            return fmiStatus::fmiError;
        }
        nominals[0] = 1.0;
        nominals[1] = 1.0;
        fmiStatus::fmiOK
    }

    fn update_discrete_states(&mut self) -> bool {
        if self.data.h <= 0.0 && self.data.v < 0.0 {
            self.data.h = 0.0;
            self.data.v = -self.data.e * self.data.v;
            if self.data.v.abs() < self.data.v_min {
                self.data.v = 0.0;
            }
            true
        } else {
            false
        }
    }

    fn set_continuous_states(&mut self, x: &[f64]) -> fmiStatus {
        if x.len() != 2 {
            self.log_error("Continuous states array must have length 2");
            return fmiStatus::fmiError;
        }
        self.data.h = x[0];
        self.data.v = x[1];
        fmiStatus::fmiOK
    }

    fn get_continuous_state_derivatives(&self, der_x: &mut [f64]) -> fmiStatus {
        if der_x.len() != 2 {
            self.log_error("Continuous state derivatives array must have length 2");
            return fmiStatus::fmiError;
        }
        der_x[0] = self.data.v;
        der_x[1] = self.data.g;
        fmiStatus::fmiOK
    }

    fn get_Float64(&self, value_reference: u32, value: &mut f64) -> fmiStatus {
        match ValueReference::try_from(value_reference) {
            Ok(ValueReference::time) => *value = self.data.time,
            Ok(ValueReference::h) => *value = self.data.h,
            Ok(ValueReference::der_h) => *value = self.data.v,
            Ok(ValueReference::v) => *value = self.data.v,
            Ok(ValueReference::der_v) => *value = self.data.g,
            Ok(ValueReference::g) => *value = self.data.g,
            Ok(ValueReference::e) => *value = self.data.e,
            Ok(ValueReference::v_min) => *value = self.data.v_min,
            _ => {
                let message = format!("Unknown value reference for type Float64: {}", value_reference);
                self.log_error(&message);
                return fmiStatus::fmiError;
            }
        }
        fmiStatus::fmiOK
    }

}

// Include shared FMI3 implementation
// This compiles common functions directly into this DLL
#[cfg(feature = "fmi2")]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../fmi-export/src/fmi2.rs"));

#[cfg(not(feature = "fmi2"))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../fmi-export/src/fmi3.rs"));
