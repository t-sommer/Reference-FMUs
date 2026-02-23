#![allow(non_camel_case_types, non_snake_case, unused_variables)]

#[cfg(feature = "fmi2")]
use fmi::fmi2::types::*;
// #[cfg(not(feature = "fmi2"))]
use fmi::{fmi3::types::*, types::fmiStatus};
use fmi::types::fmiInstanceEnvironment;
use std::any::type_name_of_val;
use std::os::raw::c_void;
use std::ptr::null_mut;
use fmi_export::{BaseModel, InterfaceType, ModelMode, ValueReference};
use std::{error::Error, f64};
use serde::{Deserialize, Serialize};


type LogError = dyn Fn(&str);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ModelData {
    interfaceType: InterfaceType,
    mode: ModelMode,
    eventModeUsed: bool,
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

    fn new(interfaceType: InterfaceType, instanceEnvironment: fmiInstanceEnvironment, logMessage: Box<LogError>) -> Self {
        ModelInstance {
            data: ModelData::default(interfaceType),
            logError: logMessage,
        }
    }

    fn doFixedStep(&mut self, stepSize: f64) {
        self.data.v += self.data.g * stepSize;
        self.data.h += self.data.v * stepSize;

        if self.data.h <= 0.0 {
            self.data.h = 0.0;
            self.data.v = -self.data.e * self.data.v;
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

    fn update_discrete_states(&mut self) {
        if self.data.h <= 0.0 && self.data.v < 0.0 {
            self.data.h = 0.0;
            self.data.v = -self.data.e * self.data.v;
            if self.data.v.abs() < self.data.v_min {
                self.data.v = 0.0;
            }
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

    fn getFloat64(&self, value_reference: &ValueReference) -> Result<&f64, Box<dyn Error>> {
        match value_reference {
            ValueReference::time => Ok(&self.data.time),
            ValueReference::h => Ok(&self.data.h),
            ValueReference::der_h => Ok(&self.data.v),
            ValueReference::v => Ok(&self.data.v),
            ValueReference::der_v => Ok(&self.data.g),
            ValueReference::g => Ok(&self.data.g),
            ValueReference::e => Ok(&self.data.e),
            ValueReference::v_min => Ok(&self.data.v_min),
        }
    }

}

// Include shared FMI3 implementation
// This compiles common functions directly into this DLL
#[cfg(feature = "fmi2")]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../fmi-export/src/fmi2.rs"));

#[cfg(not(feature = "fmi2"))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../fmi-export/src/fmi3.rs"));
