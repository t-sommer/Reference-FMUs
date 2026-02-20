// Shared FMI3 implementation that can be included by multiple FMUs
// This file is meant to be included via include!() macro, not imported as a module
//
// Usage in your FMU crate:
// ```rust
// include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../fmi/src/fmi3/shared_impl.rs"));
// ```

// Note: This file assumes the including crate has:
// - use fmi::fmi3::types::*;
// - A ModelInstance type with appropriate methods
// - get_instance! and get_instance_mut! macros defined

macro_rules! error {
    ($inst:expr, $($arg:tt)+) => {{
        let message = format!($($arg)+);
        ($inst.logError)(&message);
        return fmi2Error;
    }};
}

macro_rules! get_instance {
    ($instance: expr) => {{

        if $instance.is_null() {
            return fmi2Error;
        }

        unsafe { &*($instance as *const ModelInstance) }
    }};
}

macro_rules! get_instance_mut {
    ($instance: expr) => {{

        if $instance.is_null() {
            return fmi2Error;
        }

        unsafe { &mut*($instance as *mut ModelInstance) }
    }};
}

macro_rules! assert_not_null {
    ($value:expr, $instance:expr) => {
        if $value.is_null() {
            let message = format!("Argument {} must not be NULL.", stringify!($value));
            ($instance.logError)(&message);
            return fmi2Error;
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
        fmi2Error
}};
}

/***************************************************
Types for Common Functions
****************************************************/

// typedef const char* fmi2GetTypesPlatformTYPE(void);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetTypesPlatform() -> fmi2String {
    b"default\0".as_ptr() as fmi2String
}

/* Inquire version numbers of header files and setting logging status */

// typedef const char* fmi2GetVersionTYPE(void);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetVersion() -> fmi2String {
    b"2.0\0".as_ptr() as fmi2String
}

// typedef fmi2Status  fmi2SetDebugLoggingTYPE(fmi2Component c,
//                                             fmi2Boolean loggingOn,
//                                             size_t nCategories,
//                                             const fmi2String categories[]);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2SetDebugLogging(
    c: fmi2Component,
    loggingOn: fmi2Boolean,
    nCategories: usize,
    categories: *const fmi2String,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

/* Creation and destruction of FMU instances and setting debug status */

// typedef fmi2Component fmi2InstantiateTYPE(fmi2String instanceName,
//                                             fmi2Type fmuType,
//                                             fmi2String fmuGUID,
//                                             fmi2String fmuResourceLocation,
//                                             const fmi2CallbackFunctions* functions,
//                                             fmi2Boolean visible,
//                                             fmi2Boolean loggingOn);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2Instantiate(
    instanceName: fmi2String,
    fmuType: fmi2Type,
    fmuGUID: fmi2String,
    fmuResourceLocation: fmi2String,
    functions: *const fmi2CallbackFunctions,
    visible: fmi2Boolean,
    loggingOn: fmi2Boolean,
) -> fmi2Component {
    todo!()
}

// typedef void          fmi2FreeInstanceTYPE(fmi2Component c);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2FreeInstance(c: fmi2Component) {
    todo!()
}

// /* Enter and exit initialization mode, terminate and reset */
// typedef fmi2Status fmi2SetupExperimentTYPE       (fmi2Component c,
//                                                     fmi2Boolean toleranceDefined,
//                                                     fmi2Real tolerance,
//                                                     fmi2Real startTime,
//                                                     fmi2Boolean stopTimeDefined,
//                                                     fmi2Real stopTime);
// typedef fmi2Status fmi2EnterInitializationModeTYPE(fmi2Component c);
// typedef fmi2Status fmi2ExitInitializationModeTYPE (fmi2Component c);
// typedef fmi2Status fmi2TerminateTYPE              (fmi2Component c);
// typedef fmi2Status fmi2ResetTYPE                  (fmi2Component c);

// /* Getting and setting variable values */
// typedef fmi2Status fmi2GetRealTYPE   (fmi2Component c, const fmi2ValueReference vr[], size_t nvr, fmi2Real    value[]);
// typedef fmi2Status fmi2GetIntegerTYPE(fmi2Component c, const fmi2ValueReference vr[], size_t nvr, fmi2Integer value[]);
// typedef fmi2Status fmi2GetBooleanTYPE(fmi2Component c, const fmi2ValueReference vr[], size_t nvr, fmi2Boolean value[]);
// typedef fmi2Status fmi2GetStringTYPE (fmi2Component c, const fmi2ValueReference vr[], size_t nvr, fmi2String  value[]);

// typedef fmi2Status fmi2SetRealTYPE   (fmi2Component c, const fmi2ValueReference vr[], size_t nvr, const fmi2Real    value[]);
// typedef fmi2Status fmi2SetIntegerTYPE(fmi2Component c, const fmi2ValueReference vr[], size_t nvr, const fmi2Integer value[]);
// typedef fmi2Status fmi2SetBooleanTYPE(fmi2Component c, const fmi2ValueReference vr[], size_t nvr, const fmi2Boolean value[]);
// typedef fmi2Status fmi2SetStringTYPE (fmi2Component c, const fmi2ValueReference vr[], size_t nvr, const fmi2String  value[]);

// /* Getting and setting the internal FMU state */
// typedef fmi2Status fmi2GetFMUstateTYPE           (fmi2Component c, fmi2FMUstate* FMUstate);
// typedef fmi2Status fmi2SetFMUstateTYPE           (fmi2Component c, fmi2FMUstate  FMUstate);
// typedef fmi2Status fmi2FreeFMUstateTYPE          (fmi2Component c, fmi2FMUstate* FMUstate);
// typedef fmi2Status fmi2SerializedFMUstateSizeTYPE(fmi2Component c, fmi2FMUstate  FMUstate, size_t* size);
// typedef fmi2Status fmi2SerializeFMUstateTYPE     (fmi2Component c, fmi2FMUstate  FMUstate, fmi2Byte[], size_t size);
// typedef fmi2Status fmi2DeSerializeFMUstateTYPE   (fmi2Component c, const fmi2Byte serializedState[], size_t size, fmi2FMUstate* FMUstate);

// /* Getting partial derivatives */
// typedef fmi2Status fmi2GetDirectionalDerivativeTYPE(fmi2Component c,
//                                                     const fmi2ValueReference vUnknown_ref[], size_t nUnknown,
//                                                     const fmi2ValueReference vKnown_ref[],   size_t nKnown,
//                                                     const fmi2Real dvKnown[],
//                                                     fmi2Real dvUnknown[]);

// /***************************************************
// Types for Functions for FMI2 for Model Exchange
// ****************************************************/

// /* Enter and exit the different modes */
// typedef fmi2Status fmi2EnterEventModeTYPE         (fmi2Component c);
// typedef fmi2Status fmi2NewDiscreteStatesTYPE      (fmi2Component c, fmi2EventInfo* fmi2eventInfo);
// typedef fmi2Status fmi2EnterContinuousTimeModeTYPE(fmi2Component c);
// typedef fmi2Status fmi2CompletedIntegratorStepTYPE(fmi2Component c,
//                                                     fmi2Boolean   noSetFMUStatePriorToCurrentPoint,
//                                                     fmi2Boolean*  enterEventMode,
//                                                     fmi2Boolean*  terminateSimulation);

// /* Providing independent variables and re-initialization of caching */
// typedef fmi2Status fmi2SetTimeTYPE            (fmi2Component c, fmi2Real time);
// typedef fmi2Status fmi2SetContinuousStatesTYPE(fmi2Component c, const fmi2Real x[], size_t nx);

// /* Evaluation of the model equations */
// typedef fmi2Status fmi2GetDerivativesTYPE               (fmi2Component c, fmi2Real derivatives[],     size_t nx);
// typedef fmi2Status fmi2GetEventIndicatorsTYPE           (fmi2Component c, fmi2Real eventIndicators[], size_t ni);
// typedef fmi2Status fmi2GetContinuousStatesTYPE          (fmi2Component c, fmi2Real x[],               size_t nx);
// typedef fmi2Status fmi2GetNominalsOfContinuousStatesTYPE(fmi2Component c, fmi2Real x_nominal[],       size_t nx);


// /***************************************************
// Types for Functions for FMI2 for Co-Simulation
// ****************************************************/

// /* Simulating the slave */
// typedef fmi2Status fmi2SetRealInputDerivativesTYPE (fmi2Component c,
//                                                     const fmi2ValueReference vr[], size_t nvr,
//                                                     const fmi2Integer order[],
//                                                     const fmi2Real value[]);
// typedef fmi2Status fmi2GetRealOutputDerivativesTYPE(fmi2Component c,
//                                                     const fmi2ValueReference vr[], size_t nvr,
//                                                     const fmi2Integer order[],
//                                                     fmi2Real value[]);
// typedef fmi2Status fmi2DoStepTYPE   (fmi2Component c,
//                                         fmi2Real      currentCommunicationPoint,
//                                         fmi2Real      communicationStepSize,
//                                         fmi2Boolean   noSetFMUStatePriorToCurrentPoint);
// typedef fmi2Status fmi2CancelStepTYPE(fmi2Component c);

// /* Inquire slave status */
// typedef fmi2Status fmi2GetStatusTYPE       (fmi2Component c, const fmi2StatusKind s, fmi2Status*  value);
// typedef fmi2Status fmi2GetRealStatusTYPE   (fmi2Component c, const fmi2StatusKind s, fmi2Real*    value);
// typedef fmi2Status fmi2GetIntegerStatusTYPE(fmi2Component c, const fmi2StatusKind s, fmi2Integer* value);
// typedef fmi2Status fmi2GetBooleanStatusTYPE(fmi2Component c, const fmi2StatusKind s, fmi2Boolean* value);
// typedef fmi2Status fmi2GetStringStatusTYPE (fmi2Component c, const fmi2StatusKind s, fmi2String*  value);

