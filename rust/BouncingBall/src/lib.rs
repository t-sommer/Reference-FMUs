#![allow(non_camel_case_types, non_snake_case, unused_variables)]

use fmi::fmi3::types::*;
use std::{f64, ffi::CString};
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::any::type_name_of_val;
use serde::{Deserialize, Serialize};

type LogError = dyn Fn(&str);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum ModelMode {
    Instantiated,
    InitializationMode,
    EventMode,
    ContinuousTimeMode,
    StepMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ModelData {
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

    fn default() -> Self {
        ModelData {
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

    fn new(instanceEnvironment: fmi3InstanceEnvironment, logMessage: Option<fmi3LogMessageCallback>) -> Self {
  
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
            data: ModelData::default(),
            logError: Box::new(log_error),
        }
    }

    fn doFixedStep(&mut self, stepSize: fmi3Float64) {
        self.data.v += self.data.g * stepSize;
        self.data.h += self.data.v * stepSize;

        if self.data.h <= 0.0 {
            self.data.h = 0.0;
            self.data.v = -self.data.e * self.data.v;
        }
    }

}

macro_rules! error {
    ($inst:expr, $($arg:tt)+) => {{
        let message = format!($($arg)+);
        ($inst.logError)(&message);
        return fmi3Error;
    }};
}

macro_rules! get_instance {
    ($instance: expr) => {{

        if $instance.is_null() {
            return fmi3Error;
        }

        unsafe { &*($instance as *const ModelInstance) }
    }};
}

macro_rules! get_instance_mut {
    ($instance: expr) => {{

        if $instance.is_null() {
            return fmi3Error;
        }

        unsafe { &mut*($instance as *mut ModelInstance) }
    }};
}

macro_rules! assert_not_null {
    ($value:expr, $instance:expr) => {
        if $value.is_null() {
            let message = format!("Argument {} must not be NULL.", stringify!($value));
            ($instance.logError)(&message);
            return fmi3Error;
        }
    };
}


macro_rules! current_fn {
    () => {{
        fn __current_fn_marker() {}
        let name = type_name_of_val(&__current_fn_marker);
        // strip the trailing "::__current_fn_marker"
        match name.rfind("::") {
            Some(idx) => &name[..idx],
            None => name,
        }
    }};
}

macro_rules! NOT_IMPLEMENTED {
    ($instance:expr) => {{
        let instance = get_instance!($instance);
        let function_name = current_fn!();
        let message = format!("Function {} is not implemented.", function_name);
        (instance.logError)(&message);
        fmi3Error
}};
}

/* Inquire version numbers and setting logging status */

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetVersion() -> fmi3String {
    b"3.0\0".as_ptr() as fmi3String
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3SetDebugLogging(
    instance: fmi3Instance,
    loggingOn: fmi3Boolean,
    nCategories: usize,
    categories: *const fmi3String,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

/* Creation and destruction of FMU instances and setting debug status */

#[unsafe(no_mangle)]
pub extern "C" fn fmi3InstantiateModelExchange(
    instanceName: fmi3String,
    instantiationToken: fmi3String,
    resourcePath: fmi3String,
    visible: fmi3Boolean,
    loggingOn: fmi3Boolean,
    instanceEnvironment: fmi3InstanceEnvironment,
    logMessage: Option<fmi3LogMessageCallback>,
) -> fmi3Instance {
    let instance = ModelInstance::new(instanceEnvironment, logMessage);
    let instance = Box::new(instance);
    Box::into_raw(instance) as fmi3Instance
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3InstantiateCoSimulation(
    instanceName: fmi3String,
    instantiationToken: fmi3String,
    resourcePath: fmi3String,
    visible: fmi3Boolean,
    loggingOn: fmi3Boolean,
    eventModeUsed: fmi3Boolean,
    earlyReturnAllowed: fmi3Boolean,
    requiredIntermediateVariables: *const fmi3ValueReference,
    nRequiredIntermediateVariables: usize,
    instanceEnvironment: fmi3InstanceEnvironment,
    logMessage: Option<fmi3LogMessageCallback>,
    intermediateUpdate: fmi3IntermediateUpdateCallback,
) -> fmi3Instance {
    let instance = ModelInstance::new(instanceEnvironment, logMessage);
    let instance = Box::new(instance);
    Box::into_raw(instance) as fmi3Instance
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3InstantiateScheduledExecution(
    instanceName: fmi3String,
    instantiationToken: fmi3String,
    resourcePath: fmi3String,
    visible: fmi3Boolean,
    loggingOn: fmi3Boolean,
    instanceEnvironment: fmi3InstanceEnvironment,
    logMessage: Option<fmi3LogMessageCallback>,
    clockUpdate: Option<fmi3ClockUpdateCallback>,
    lockPreemption: Option<fmi3LockPreemptionCallback>,
    unlockPreemption: Option<fmi3UnlockPreemptionCallback>,
) -> fmi3Instance {
    null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3FreeInstance(instance: fmi3Instance) {
    
    if instance.is_null() {
        return;
    }

    unsafe {
        let _ = Box::from_raw(instance as *mut ModelInstance);
    }
}

/* Enter and exit initialization mode, enter event mode, terminate and reset */

#[unsafe(no_mangle)]
pub extern "C" fn fmi3EnterInitializationMode(
    instance: fmi3Instance,
    toleranceDefined: bool,
    tolerance: fmi3Float64,
    startTime: fmi3Float64,
    stopTimeDefined: bool,
    stopTime: fmi3Float64,
) -> fmi3Status {

    let instance = get_instance_mut!(instance);

    if instance.data.mode != ModelMode::Instantiated {
        error!(instance, "Cannot enter initialization mode from mode {:?}.", instance.data.mode);
    }
    
    instance.data.mode = ModelMode::InitializationMode;

    fmi3OK
}

macro_rules! assert_mode {
    ($mode:expr, $instance:expr) => {
        if $instance.data.mode != $mode {
            error!($instance, 
                "Function {} may only be called in mode {:?} but current mode is {:?}.",
                current_fn!(),
                $mode,
                $instance.data.mode
            );
        }
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3ExitInitializationMode(instance: fmi3Instance) -> fmi3Status {

    let instance = get_instance_mut!(instance);

    assert_mode!(ModelMode::InitializationMode, instance);

    instance.data.mode = if instance.data.eventModeUsed {
        ModelMode::EventMode
    } else {
        ModelMode::StepMode
    };

    instance.data.mode = ModelMode::InitializationMode;

    fmi3OK
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3EnterEventMode(instance: fmi3Instance) -> fmi3Status {
    fmi3OK
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3Terminate(instance: fmi3Instance) -> fmi3Status {
    fmi3OK
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3Reset(instance: fmi3Instance) -> fmi3Status {
    let instance = get_instance_mut!(instance);
    instance.data = ModelData::default();
    fmi3OK
}

/* Getting and setting variable values */

macro_rules! make_getter {
    ($name:ident, $type:ident, $getter:ident) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(
            instance: fmi3Instance,
            valueReferences: *const fmi3ValueReference,
            nValueReferences: usize,
            values: *mut $type,
            nValues: usize,
        ) -> fmi3Status {
            NOT_IMPLEMENTED!(instance)
        }
    };
}

make_getter!(fmi3GetFloat32, fmi3Float32, getFloat32);

// make_getter!(fmi3GetFloat64, fmi3Float64, getFloat64);
#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetFloat64(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    values: *mut fmi3Float64,
    nValues: usize,
) -> fmi3Status {

    let instance = get_instance!(instance);

    assert_not_null!(valueReferences, instance);
    assert_not_null!(values, instance);

    let valueReferences =
        unsafe { std::slice::from_raw_parts(valueReferences, nValueReferences) };

    let values = unsafe { std::slice::from_raw_parts_mut(values, nValues) };

    for (i, &vr) in valueReferences.iter().enumerate() {

        let data = &instance.data;

        values[i] = data.time;
                
        match ValueReference::try_from(vr) {
            Ok(ValueReference::time) => values[i] = data.time,
            Ok(ValueReference::h) => values[i] = data.h,
            Ok(ValueReference::der_h) => values[i] = data.v,
            Ok(ValueReference::v) => values[i] = data.v,
            Ok(ValueReference::der_v) => values[i] = -data.g,
            Ok(ValueReference::g) => values[i] = data.g,
            Ok(ValueReference::e) => values[i] = data.e,
            Ok(ValueReference::v_min) => values[i] = data.v_min,
            _ => {
                error!(instance, "Unknown value reference: {}", vr);
            }
        }
    }

    fmi3OK
}

make_getter!(fmi3GetInt8, fmi3Int8, getInt8);
make_getter!(fmi3GetUInt8, fmi3UInt8, getUInt8);
make_getter!(fmi3GetInt16, fmi3Int16, getInt16);
make_getter!(fmi3GetUInt16, fmi3UInt16, getUInt16);
make_getter!(fmi3GetInt32, fmi3Int32, getInt32);
make_getter!(fmi3GetUInt32, fmi3UInt32, getUInt32);
make_getter!(fmi3GetInt64, fmi3Int64, getInt64);
make_getter!(fmi3GetUInt64, fmi3UInt64, getUInt64);
make_getter!(fmi3GetBoolean, fmi3Boolean, getBoolean);
// make_getter!(fmi3GetString, fmi3String, getString);

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetString(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    values: *mut fmi3String,
    nValues: usize,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetBinary(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    valueSizes: *mut usize,
    values: *mut *const fmi3Binary,
    nValues: usize,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetClock(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    values: *mut fmi3Clock,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

macro_rules! make_setter {
    ($name:ident, $type:ident, $setter:ident) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(
            instance: fmi3Instance,
            valueReferences: *const fmi3ValueReference,
            nValueReferences: usize,
            values: *const $type,
            nValues: usize,
        ) -> fmi3Status {
            NOT_IMPLEMENTED!(instance)
        }
    };
}

make_setter!(fmi3SetFloat32, fmi3Float32, setFloat32);
make_setter!(fmi3SetFloat64, fmi3Float64, setFloat64);
make_setter!(fmi3SetInt8, fmi3Int8, setInt8);
make_setter!(fmi3SetUInt8, fmi3UInt8, setUInt8);
make_setter!(fmi3SetInt16, fmi3Int16, setInt16);
make_setter!(fmi3SetUInt16, fmi3UInt16, setUInt16);
make_setter!(fmi3SetInt32, fmi3Int32, setInt32);
make_setter!(fmi3SetUInt32, fmi3UInt32, setUInt32);
make_setter!(fmi3SetInt64, fmi3Int64, setInt64);
make_setter!(fmi3SetUInt64, fmi3UInt64, setUInt64);
make_setter!(fmi3SetBoolean, fmi3Boolean, setBoolean);
// make_setter!(fmi3SetString, fmi3String, setString);

#[unsafe(no_mangle)]
pub extern "C" fn fmi3SetString(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    values: *const fmi3String,
    nValues: usize,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3SetBinary(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    sizes: *const usize,
    values: *const *const u8,
    nValues: usize,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3SetClock(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    values: *const u32,
    nValues: usize,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

/* Getting Variable Dependency Information */

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetNumberOfVariableDependencies(
    instance: fmi3Instance,
    valueReference: fmi3ValueReference,
    nDependencies: *mut usize,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetVariableDependencies(
    instance: fmi3Instance,
    valueReference: fmi3ValueReference,
    elementIndicesOfDependent: *mut usize,
    independentVariables: *mut fmi3ValueReference,
    elementIndicesOfIndependents: *mut usize,
    dependencyKinds: *mut i32,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

/* Getting and setting the internal FMU state */

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetFMUState(
    instance: fmi3Instance,
    FMUState: *mut fmi3FMUState,
) -> fmi3Status {
    let instance = get_instance!(instance);
    let fmu_state = Box::new(instance.data.clone());
    unsafe { *FMUState = Box::into_raw(fmu_state) as *mut c_void };
    fmi3OK
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3SetFMUState(instance: fmi3Instance, FMUState: fmi3FMUState) -> fmi3Status {
    let instance = get_instance_mut!(instance);
    let fmu_state = unsafe { &*(FMUState as *const ModelData) };
    instance.data = fmu_state.clone();
    fmi3OK
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3FreeFMUState(
    instance: fmi3Instance,
    FMUState: *mut fmi3FMUState,
) -> fmi3Status {

    let instance: &mut ModelInstance = get_instance_mut!(instance);

    assert_not_null!(FMUState, instance);

    unsafe {
        let _ = Box::from_raw(FMUState as *mut ModelData);
    }

    unsafe { *FMUState = null_mut() };

    fmi3OK
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3SerializedFMUStateSize(
    instance: fmi3Instance,
    FMUState: *mut c_void,
    size: *mut usize,
) -> fmi3Status {

    let instance: &mut ModelInstance = get_instance_mut!(instance);

    assert_not_null!(FMUState, instance);
    assert_not_null!(size, instance);

    let data =  unsafe { &*(FMUState as *const ModelData) };

    let serialized = serde_json::to_vec_pretty(data).unwrap();

    unsafe { *size = serialized.len() };

    fmi3OK
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3SerializeFMUState(
    instance: fmi3Instance,
    FMUState: *mut c_void,
    serializedState: *mut u8,
    size: usize,
) -> fmi3Status {
    let instance = get_instance_mut!(instance);

    assert_not_null!(FMUState, instance);
    assert_not_null!(serializedState, instance);
    
    let data =  unsafe { &*(FMUState as *const ModelData) };
    
    if let Ok(serialized) = serde_json::to_vec_pretty(data) {

        let required_size = serialized.len();
        
        if size != required_size {
            error!(instance,
                "Provided buffer size {} does not match the size of the serialized FMU state {}.",
                size, required_size
            );
        }

        unsafe { std::ptr::copy_nonoverlapping(serialized.as_ptr(), serializedState, size) };
    } else {
        error!(instance, "Failed to serialize FMU state.");
    }
        
    fmi3OK
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3DeserializeFMUState(
    instance: fmi3Instance,
    serializedState: *const u8,
    size: usize,
    FMUState: *mut *mut c_void,
) -> fmi3Status {
    let instance= get_instance!(instance);

    assert_not_null!(serializedState, instance);
    assert_not_null!(FMUState, instance);

    let serialized_slice = unsafe { std::slice::from_raw_parts(serializedState, size) };

    if let Ok(data) = serde_json::from_slice::<ModelData>(serialized_slice) {
        let fmu_state = Box::new(data);
        unsafe { *FMUState = Box::into_raw(fmu_state) as *mut c_void };
    } else {
        error!(instance, "Failed to deserialize FMU state.");
    }
    
    fmi3OK
}

/* Getting partial derivatives */

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetDirectionalDerivative(
    instance: fmi3Instance,
    unknowns: *const fmi3ValueReference,
    nUnknowns: usize,
    knowns: *const fmi3ValueReference,
    nKnowns: usize,
    seed: *const fmi3Float64,
    nSeed: usize,
    sensitivity: *mut fmi3Float64,
    nSensitivity: usize,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetAdjointDerivative(
    instance: fmi3Instance,
    unknowns: *const fmi3ValueReference,
    nUnknowns: usize,
    knowns: *const fmi3ValueReference,
    nKnowns: usize,
    seed: *const fmi3Float64,
    nSeed: usize,
    sensitivity: *mut fmi3Float64,
    nSensitivity: usize,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

/* Entering and exiting the Configuration or Reconfiguration Mode */

#[unsafe(no_mangle)]
pub extern "C" fn fmi3EnterConfigurationMode(instance: fmi3Instance) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3ExitConfigurationMode(instance: fmi3Instance) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetIntervalDecimal(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    intervals: *mut fmi3Float64,
    qualifiers: *mut fmi3IntervalQualifier,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetIntervalFraction(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    counters: *mut fmi3UInt64,
    resolutions: *mut fmi3UInt64,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetShiftDecimal(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    shifts: *mut fmi3Float64,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetShiftFraction(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    counters: *mut fmi3UInt64,
    resolutions: *mut fmi3UInt64,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3SetIntervalDecimal(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    intervals: *const fmi3Float64,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3SetIntervalFraction(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    counters: *const fmi3UInt64,
    resolutions: *const fmi3UInt64,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3SetShiftDecimal(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    shifts: *const fmi3Float64,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3SetShiftFraction(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    counters: *const fmi3UInt64,
    resolutions: *const fmi3UInt64,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3EvaluateDiscreteStates(instance: fmi3Instance) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3UpdateDiscreteStates(
    instance: fmi3Instance,
    discreteStatesNeedUpdate: *mut fmi3Boolean,
    terminateSimulation: *mut fmi3Boolean,
    nominalsOfContinuousStatesChanged: *mut fmi3Boolean,
    valuesOfContinuousStatesChanged: *mut fmi3Boolean,
    nextEventTime: *mut fmi3Float64,
) -> fmi3Status {

    let instance = get_instance_mut!(instance);

    assert_not_null!(discreteStatesNeedUpdate, instance);
    assert_not_null!(terminateSimulation, instance);
    assert_not_null!(nominalsOfContinuousStatesChanged, instance);
    assert_not_null!(valuesOfContinuousStatesChanged, instance);
    assert_not_null!(nextEventTime, instance);

    let data = &mut instance.data;

    if data.h <= 0.0 {

        data.h = f64::MIN_POSITIVE;  // slightly above 0 to avoid zero-crossing
        data.v = -data.e * data.v;

        if data.v.abs() < data.v_min {
            data.v = 0.0;
            data.g = 0.0;  // stop bouncing
        }

        unsafe { *valuesOfContinuousStatesChanged = fmi3True };
    } else {
        unsafe { *valuesOfContinuousStatesChanged = fmi3False };
    }

    unsafe { 
        *discreteStatesNeedUpdate = fmi3False;
        *terminateSimulation = fmi3False;
        *nominalsOfContinuousStatesChanged = fmi3False;
        *nextEventTime = f64::INFINITY;
    }

    fmi3OK
}

/***************************************************
Types for Functions for Model Exchange
****************************************************/

#[unsafe(no_mangle)]
pub extern "C" fn fmi3EnterContinuousTimeMode(instance: fmi3Instance) -> fmi3Status {
    fmi3OK
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3CompletedIntegratorStep(
    instance: fmi3Instance,
    noSetFMUStatePriorToCurrentPoint: fmi3Boolean,
    enterEventMode: *mut fmi3Boolean,
    terminateSimulation: *mut fmi3Boolean,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

/* Providing independent variables and re-initialization of caching */

#[unsafe(no_mangle)]
pub extern "C" fn fmi3SetTime(instance: fmi3Instance, time: fmi3Float64) -> fmi3Status {
    let instance = get_instance_mut!(instance);
    instance.data.time = time;
    fmi3OK
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3SetContinuousStates(
    instance: fmi3Instance,
    continuousStates: *const fmi3Float64,
    nContinuousStates: usize,
) -> fmi3Status {
    let instance = get_instance_mut!(instance);

    assert_not_null!(continuousStates, instance);

    if nContinuousStates != 2 {
        error!(instance, "Number of continuous states ({}) does not match the model (2).", nContinuousStates);
    }

    let continuous_states = unsafe { std::slice::from_raw_parts(continuousStates, nContinuousStates) };

    instance.data.h = continuous_states[0];
    instance.data.v = continuous_states[1]; 

    fmi3OK
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetContinuousStateDerivatives(
    instance: fmi3Instance,
    derivatives: *mut fmi3Float64,
    nContinuousStates: usize,
) -> fmi3Status {
    let instance = get_instance!(instance);

    assert_not_null!(derivatives, instance);

    if nContinuousStates != 2 {
        error!(instance, "Number of continuous state derivatives requested ({}) does not match the model (2).", nContinuousStates);
    }

    let derivatives = unsafe { std::slice::from_raw_parts_mut(derivatives, nContinuousStates) };

    derivatives[0] = instance.data.v;
    derivatives[1] = instance.data.g; 

    fmi3OK
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetEventIndicators(
    instance: fmi3Instance,
    eventIndicators: *mut fmi3Float64,
    nEventIndicators: usize,
) -> fmi3Status {
    let instance = get_instance!(instance);

    assert_not_null!(eventIndicators, instance);

    if nEventIndicators != 1 {
        error!(instance, "Number of event indicators requested ({}) does not match the model (1).", nEventIndicators);
    }

    let event_indicators = unsafe { std::slice::from_raw_parts_mut(eventIndicators, nEventIndicators) };

    event_indicators[0] = instance.data.h;

    fmi3OK}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetContinuousStates(
    instance: fmi3Instance,
    continuousStates: *mut fmi3Float64,
    nContinuousStates: usize,
) -> fmi3Status {

    let instance = get_instance!(instance);

    assert_not_null!(continuousStates, instance);

    if nContinuousStates != 2 {
        error!(instance, "Number of continuous states requested ({}) does not match the model (2).", nContinuousStates);
    }

    let continuous_states = unsafe { std::slice::from_raw_parts_mut(continuousStates, nContinuousStates) };

    continuous_states[0] = instance.data.h;
    continuous_states[1] = instance.data.v; 

    fmi3OK
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetNominalsOfContinuousStates(
    instance: fmi3Instance,
    nominals: *mut fmi3Float64,
    nNominals: usize,
) -> fmi3Status {
    let instance = get_instance!(instance);

    assert_not_null!(nominals, instance);

    if nNominals != 2 {
        error!(instance, "Number of nominals of continuous states requested ({nNominals}) does not match the model (2).");
    }

    let nominals = unsafe { std::slice::from_raw_parts_mut(nominals, nNominals) };

    nominals[0] = 1.0;
    nominals[1] = 1.0; 

    fmi3OK
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetNumberOfEventIndicators(
    instance: fmi3Instance,
    nEventIndicators: *mut usize,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetNumberOfContinuousStates(
    instance: fmi3Instance,
    nContinuousStates: *mut usize,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

/***************************************************
Types for Functions for Co-Simulation
****************************************************/

#[unsafe(no_mangle)]
pub extern "C" fn fmi3EnterStepMode(instance: fmi3Instance) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3GetOutputDerivatives(
    instance: fmi3Instance,
    valueReferences: *const fmi3ValueReference,
    nValueReferences: usize,
    orders: *const fmi3Int32,
    values: *mut fmi3Float64,
    nValues: usize,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn fmi3DoStep(
    instance: fmi3Instance,
    currentCommunicationPoint: fmi3Float64,
    communicationStepSize: fmi3Float64,
    noSetFMUStatePriorToCurrentPoint: fmi3Boolean,
    eventHandlingNeeded: *mut fmi3Boolean,
    terminateSimulation: *mut fmi3Boolean,
    earlyReturn: *mut fmi3Boolean,
    lastSuccessfulTime: *mut fmi3Float64,
) -> fmi3Status {

    let instance = get_instance_mut!(instance);

    assert_not_null!(eventHandlingNeeded, instance);
    assert_not_null!(terminateSimulation, instance);
    assert_not_null!(earlyReturn, instance);
    assert_not_null!(lastSuccessfulTime, instance);

    assert_mode!(ModelMode::StepMode, instance);

    instance.doFixedStep(communicationStepSize);

    instance.data.time = currentCommunicationPoint + communicationStepSize;

    fmi3OK
}

/***************************************************
Types for Functions for Scheduled Execution
****************************************************/

#[unsafe(no_mangle)]
pub extern "C" fn fmi3ActivateModelPartition(
    instance: fmi3Instance,
    clockReference: fmi3ValueReference,
    activationTime: fmi3Float64,
) -> fmi3Status {
    NOT_IMPLEMENTED!(instance)
}
