#![allow(non_camel_case_types, non_snake_case, unused_variables)]

#[cfg(feature = "fmi2")]
use fmi::fmi2::types::*;
// #[cfg(not(feature = "fmi2"))]
use fmi::fmi3::types::*;
use fmi::types::fmiInstanceEnvironment;
use std::any::type_name_of_val;
use std::os::raw::c_void;
use std::ptr::null_mut;
use fmi_export::{InterfaceType, ModelMode, ValueReference};
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
  
        // convert raw pointer to thread-safe representation
        let instance_environment = instanceEnvironment as usize;

        // let logMessage = logMessage.unwrap();

        // let log_error = move |message: &str| {

        //     let message = CString::new(message).unwrap();
            
        //     todo!("Handle FMI 2 / 3 logging callback properly");
        //     // unsafe { 
        //     //     logMessage(
        //     //         instanceEnvironment,
        //     //         fmi3Error,
        //     //         b"error\0".as_ptr() as fmi3String,
        //     //         message.as_ptr() as fmi3String,
        //     //     ) 
        //     // };

        // };

        ModelInstance {
            data: ModelData::default(interfaceType),
            logError: logMessage,
        }
    }

    fn doFixedStep(&mut self, stepSize: f64) {
        // self.data.v += self.data.g * stepSize;
        // self.data.h += self.data.v * stepSize;

        // if self.data.h <= 0.0 {
        //     self.data.h = 0.0;
        //     self.data.v = -self.data.e * self.data.v;
        // }
    }

    fn get_event_indicators(&self, z: &mut [f64]) {
        todo!()
    }

    fn get_continuous_states(&self, x: &mut [f64]) {
        todo!()
    }

    fn update_discrete_states(&self) {
        todo!()
    }

    fn set_continuous_states(&self, x: &[f64]) {
        todo!()
    }

    fn get_continuous_state_derivatives(&self, der_x: &mut [f64]) {
        todo!()
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
