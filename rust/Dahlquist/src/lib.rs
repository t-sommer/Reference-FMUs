#![allow(non_camel_case_types, non_snake_case, unused_variables)]

use fmi::fmi3::types::*;
use std::any::type_name_of_val;
use std::os::raw::c_void;
use std::ptr::null_mut;

use fmi_export::{InterfaceType, ModelMode};
use std::{error::Error, f64, ffi::CString};
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

#[derive(Debug)]
enum ValueReference {
    time,
    h,
    der_h,
    v,
    der_v,
    g,
    e,
    v_min,
}

impl TryFrom<u32> for ValueReference {
    
    type Error = ();
    
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            x if x == ValueReference::time as u32 => Ok(ValueReference::time),
            x if x == ValueReference::h as u32 => Ok(ValueReference::h),
            x if x == ValueReference::der_h as u32 => Ok(ValueReference::der_h),
            x if x == ValueReference::v as u32 => Ok(ValueReference::v),
            x if x == ValueReference::der_v as u32 => Ok(ValueReference::der_v),
            x if x == ValueReference::g as u32 => Ok(ValueReference::g),
            x if x == ValueReference::e as u32 => Ok(ValueReference::e),
            x if x == ValueReference::v_min as u32 => Ok(ValueReference::v_min),
            _ => Err(()),
        }
    }

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
        self.data.v += self.data.g * stepSize;
        self.data.h += self.data.v * stepSize;

        if self.data.h <= 0.0 {
            self.data.h = 0.0;
            self.data.v = -self.data.e * self.data.v;
        }
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

        // match ValueReference::try_from(value_reference) {
        //     Ok(ValueReference::time) => Ok(&self.data.time),
        //     Ok(ValueReference::h) => Ok(&self.data.h),
        //     Ok(ValueReference::der_h) => Ok(&self.data.v),
        //     Ok(ValueReference::v) => Ok(&self.data.v),
        //     Ok(ValueReference::der_v) => Ok(&self.data.g),
        //     Ok(ValueReference::g) => Ok(&self.data.g),
        //     Ok(ValueReference::e) => Ok(&self.data.e),
        //     Ok(ValueReference::v_min) => Ok(&self.data.v_min),
        //     _ => {
        //         Err(format!("Unknown value reference for type Float64: {value_reference:?}.").into())
        //     }
        //     Err(_) => todo!(),
        // }
    }

}

// Include shared FMI3 implementation
// This compiles common functions directly into this DLL
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../fmi-export/src/shared_impl.rs"));
