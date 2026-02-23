#![allow(non_camel_case_types, non_snake_case, unused_variables)]

#[cfg(feature = "fmi2")]
use fmi::fmi2::types::*;
// #[cfg(not(feature = "fmi2"))]
use fmi::{fmi3::types::*, types::fmiStatus};
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
    x: f64,
    der_x: f64,
    k: f64,
}

impl ModelData {

    fn default(interfaceType: InterfaceType) -> Self {
        ModelData {
            interfaceType: interfaceType,
            mode: ModelMode::Instantiated,
            eventModeUsed: false,
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

    fn new(interfaceType: InterfaceType, instanceEnvironment: fmi3InstanceEnvironment, logMessage: Box<LogError>) -> Self {
  
        // convert raw pointer to thread-safe representation
        let instance_environment = instanceEnvironment as usize;

        // let logMessage = logMessage.unwrap();

        // let log_error = move |message: &str| {

        //     let message = CString::new(message).unwrap();
            
        //     unsafe { 
        //         logMessage(
        //             instanceEnvironment,
        //             fmi3Error,
        //             b"error\0".as_ptr() as fmi3String,
        //             message.as_ptr() as fmi3String,
        //         ) 
        //     };

        // };

        ModelInstance {
            data: ModelData::default(interfaceType),
            logError: logMessage,
        }
    }

    fn doFixedStep(&mut self, stepSize: f64) {

        let mut der_x = [0.0];

        self.get_continuous_state_derivatives(&mut der_x);

        self.data.der_x = -self.data.k * self.data.x;

        self.data.x += self.data.der_x * stepSize;
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

    fn getFloat64(&self, value_reference: &ValueReference) -> Result<&f64, Box<dyn Error>> {
        match value_reference {
            ValueReference::time => Ok(&self.data.time),
            ValueReference::x => Ok(&self.data.x),
            ValueReference::der_x => Ok(&self.data.der_x),
            ValueReference::k => Ok(&self.data.k),
        }
    }

}

// Include shared FMI3 implementation
// This compiles common functions directly into this DLL
#[cfg(feature = "fmi2")]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../fmi-export/src/fmi2.rs"));

#[cfg(not(feature = "fmi2"))]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../fmi-export/src/fmi3.rs"));