#![allow(non_camel_case_types, non_snake_case, unused_variables)]

use fmi::{fmi3::types::*, model_description::CoSimulation};
use std::any::type_name_of_val;
use std::os::raw::c_void;
use std::ptr::null_mut;

use fmi_export::{InterfaceType, ModelMode, ValueReference};
use std::{error::Error, f64, ffi::CString};
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

#[derive(Debug, ValueReference)]
#[repr(u32)]
enum ValueReference {
    time = 0,
    x = 1,
    der_x = 2,
    k = 3,
}

impl ModelInstance {

    fn new(interfaceType: InterfaceType, instanceEnvironment: fmi3InstanceEnvironment, logMessage: Option<fmi3LogMessageCallback>) -> Self {
  
        // convert raw pointer to thread-safe representation
        let instance_environment = instanceEnvironment as usize;

        let logMessage = logMessage.unwrap();

        let log_error = move |message: &str| {

            let message = CString::new(message).unwrap();
            
            unsafe { 
                logMessage(
                    instanceEnvironment,
                    fmi3Error,
                    b"error\0".as_ptr() as fmi3String,
                    message.as_ptr() as fmi3String,
                ) 
            };

        };

        ModelInstance {
            data: ModelData::default(interfaceType),
            logError: Box::new(log_error),
        }
    }

    fn doFixedStep(&mut self, stepSize: f64) {

        let mut der_x = [0.0];

        self.get_continuous_state_derivatives(&mut der_x);

        self.data.der_x = -self.data.k * self.data.x;

        self.data.x += self.data.der_x * stepSize;
    }

    fn get_event_indicators(&self, z: &mut [f64]) {
        // nothing to do
    }

    fn get_continuous_states(&self, x: &mut [f64]) {
        x[0] = self.data.x;
    }

    fn update_discrete_states(&self) {
        // nothing to do
    }

    fn set_continuous_states(&mut self, x: &[f64]) {
        self.data.x = x[0];
    }

    fn get_continuous_state_derivatives(&self, der_x: &mut [f64]) {
        der_x[0] = -self.data.k * self.data.x;
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