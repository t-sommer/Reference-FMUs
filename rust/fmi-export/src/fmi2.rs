// Shared FMI 2.0 implementation that can be included by multiple FMUs

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

macro_rules! assert_interface_type {
    ($instance:expr, $itype:pat) => {
        if !matches!($instance.data.interfaceType, $itype) {
            error!($instance, 
                "Function {} may only be called for interface type {:?} but current interface type is {:?}.",
                current_fn!(),
                stringify!($itype),
                $instance.data.interfaceType
            );
        }
    };
}

macro_rules! assert_mode {
    ($instance:expr, $mode:pat) => {
        if !matches!($instance.data.mode, $mode) {
            error!($instance, 
                "Function {} may only be called in mode {:?} but current mode is {:?}.",
                current_fn!(),
                stringify!($mode),
                $instance.data.mode
            );
        }
    };
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
    
    if instanceName.is_null() {
        return std::ptr::null_mut();
    }

    if fmuGUID.is_null() {
        return std::ptr::null_mut();
    }

    if functions.is_null() {
        return std::ptr::null_mut();
    }

    if unsafe { (*functions).logger as *const c_void }.is_null() {
        return std::ptr::null_mut();
    }

    let logger = unsafe { (*functions).logger };
    
    let instanceName = unsafe { std::ffi::CStr::from_ptr(instanceName) }.to_owned();

    let componentEnvironment = unsafe { (*functions).componentEnvironment };
    
    let log_error = move |message: &str| {
        if let Ok(message) = std::ffi::CString::new(message) {
            unsafe {
                logger(
                    componentEnvironment,
                    instanceName.as_ptr() as fmi2String,
                    fmi2Error,
                    b"error\0".as_ptr() as fmi2String,
                    message.as_ptr() as fmi2String,
                )    
            };    
        }    
    };

    let interfaceType = match fmuType {
        fmi::fmi2::types::fmi2Type::fmi2ModelExchange => InterfaceType::ModelExchange,
        fmi::fmi2::types::fmi2Type::fmi2CoSimulation => InterfaceType::CoSimulation(Some(Solver::new())),
        _ => return std::ptr::null_mut(),
    };

    let instance = ModelInstance::new(interfaceType, Box::new(log_error));
    let instance = Box::new(instance);
    Box::into_raw(instance) as fmi2Component
}

// typedef void          fmi2FreeInstanceTYPE(fmi2Component c);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2FreeInstance(c: fmi2Component) {
    if !c.is_null() {
        let _ = unsafe { Box::from_raw(c as *mut ModelInstance)};
    }
}

/* Enter and exit initialization mode, enter event mode, terminate and reset */

// /* Enter and exit initialization mode, terminate and reset */
// typedef fmi2Status fmi2SetupExperimentTYPE       (fmi2Component c,
//                                                     fmi2Boolean toleranceDefined,
//                                                     fmi2Real tolerance,
//                                                     fmi2Real startTime,
//                                                     fmi2Boolean stopTimeDefined,
//                                                     fmi2Real stopTime);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2SetupExperiment(
    c: fmi2Component,
    toleranceDefined: fmi2Boolean,
    tolerance: fmi2Real,
    startTime: fmi2Real,
    stopTimeDefined: fmi2Boolean,
    stopTime: fmi2Real,
) -> fmi2Status {
    let instance = get_instance_mut!(c);
    assert_mode!(instance, ModelMode::Instantiated);
    fmi2OK
}

// typedef fmi2Status fmi2EnterInitializationModeTYPE(fmi2Component c);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2EnterInitializationMode(c: fmi2Component) -> fmi2Status {
    let instance = get_instance_mut!(c);
    assert_mode!(instance, ModelMode::Instantiated);
    instance.data.mode = ModelMode::InitializationMode;
    fmi2OK
}

// typedef fmi2Status fmi2ExitInitializationModeTYPE (fmi2Component c);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2ExitInitializationMode(c: fmi2Component) -> fmi2Status {
    let instance = get_instance_mut!(c);
    
    assert_mode!(instance, ModelMode::InitializationMode);

    instance.exit_initialization_mode()
    

    // instance.data.mode = match instance.data.interfaceType {
    //     InterfaceType::ModelExchange => ModelMode::EventMode,
    //     InterfaceType::CoSimulation(_) => {
    //         ModelMode::StepMode
    //     },
    // };
    
    // fmi2OK
}

// typedef fmi2Status fmi2TerminateTYPE              (fmi2Component c);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2Terminate(c: fmi2Component) -> fmi2Status {
    fmi2OK
}

// typedef fmi2Status fmi2ResetTYPE                  (fmi2Component c);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2Reset(c: fmi2Component) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

/* Getting and setting variable values */

// typedef fmi2Status fmi2GetRealTYPE   (fmi2Component c, const fmi2ValueReference vr[], size_t nvr, fmi2Real    value[]);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetReal(
    c: fmi2Component,
    vr: *const fmi2ValueReference,
    nvr: usize,
    value: *mut fmi2Real,
) -> fmi2Status {
    let instance = get_instance!(c);

    assert_not_null!(vr, instance);
    assert_not_null!(value, instance);

    let valueReferences =
        unsafe { std::slice::from_raw_parts(vr, nvr) };

    let values = unsafe { std::slice::from_raw_parts_mut(value, nvr) };

    for (i, &vr) in valueReferences.iter().enumerate() {
        let status = instance.get_Float64(vr, &mut values[i]);
        if status != fmi2OK {
            return status;
        }
    }

    fmi2OK
}

// typedef fmi2Status fmi2GetIntegerTYPE(fmi2Component c, const fmi2ValueReference vr[], size_t nvr, fmi2Integer value[]);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetInteger(
    c: fmi2Component,
    vr: *const fmi2ValueReference,
    nvr: usize,
    value: *mut fmi2Integer,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2GetBooleanTYPE(fmi2Component c, const fmi2ValueReference vr[], size_t nvr, fmi2Boolean value[]);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetBoolean(
    c: fmi2Component,
    vr: *const fmi2ValueReference,
    nvr: usize,
    value: *mut fmi2Boolean,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2GetStringTYPE (fmi2Component c, const fmi2ValueReference vr[], size_t nvr, fmi2String  value[]);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetString(
    c: fmi2Component,
    vr: *const fmi2ValueReference,
    nvr: usize,
    value: *mut fmi2String,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2SetRealTYPE   (fmi2Component c, const fmi2ValueReference vr[], size_t nvr, const fmi2Real    value[]);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2SetReal(
    c: fmi2Component,
    vr: *const fmi2ValueReference,
    nvr: usize,
    value: *const fmi2Real,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2SetIntegerTYPE(fmi2Component c, const fmi2ValueReference vr[], size_t nvr, const fmi2Integer value[]);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2SetInteger(
    c: fmi2Component,
    vr: *const fmi2ValueReference,
    nvr: usize,
    value: *const fmi2Integer,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}



// typedef fmi2Status fmi2SetBooleanTYPE(fmi2Component c, const fmi2ValueReference vr[], size_t nvr, const fmi2Boolean value[]);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2SetBoolean(
    c: fmi2Component,
    vr: *const fmi2ValueReference,
    nvr: usize,
    value: *const fmi2Boolean,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}


// typedef fmi2Status fmi2SetStringTYPE (fmi2Component c, const fmi2ValueReference vr[], size_t nvr, const fmi2String  value[]);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2SetString(
    c: fmi2Component,
    vr: *const fmi2ValueReference,
    nvr: usize,
    value: *const fmi2String,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

/* Getting and setting the internal FMU state */

// typedef fmi2Status fmi2GetFMUstateTYPE           (fmi2Component c, fmi2FMUstate* FMUstate);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetFMUstate(c: fmi2Component, FMUstate: *mut fmi2FMUstate) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2SetFMUstateTYPE           (fmi2Component c, fmi2FMUstate  FMUstate);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2SetFMUstate(c: fmi2Component, FMUstate: fmi2FMUstate) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2FreeFMUstateTYPE          (fmi2Component c, fmi2FMUstate* FMUstate);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2FreeFMUstate(c: fmi2Component, FMUstate: *mut fmi2FMUstate) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2SerializedFMUstateSizeTYPE(fmi2Component c, fmi2FMUstate  FMUstate, size_t* size);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2SerializedFMUstateSize(
    c: fmi2Component,
    FMUstate: fmi2FMUstate,
    size: *mut usize,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2SerializeFMUstateTYPE     (fmi2Component c, fmi2FMUstate  FMUstate, fmi2Byte[], size_t size);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2SerializeFMUstate(
    c: fmi2Component,
    FMUstate: fmi2FMUstate,
    serializedState: *mut fmi2Byte,
    size: usize,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2DeSerializeFMUstateTYPE   (fmi2Component c, const fmi2Byte serializedState[], size_t size, fmi2FMUstate* FMUstate);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2DeSerializeFMUstate(
    c: fmi2Component,
    serializedState: *const fmi2Byte,
    size: usize,
    FMUstate: *mut fmi2FMUstate,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}


/* Getting partial derivatives */

// typedef fmi2Status fmi2GetDirectionalDerivativeTYPE(fmi2Component c,
//                                                     const fmi2ValueReference vUnknown_ref[], size_t nUnknown,
//                                                     const fmi2ValueReference vKnown_ref[],   size_t nKnown,
//                                                     const fmi2Real dvKnown[],
//                                                     fmi2Real dvUnknown[]);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetDirectionalDerivative(
    c: fmi2Component,
    vUnknown_ref: *const fmi2ValueReference,
    nUnknown: usize,
    vKnown_ref: *const fmi2ValueReference,
    nKnown: usize,
    dvKnown: *const fmi2Real,
    dvUnknown: *mut fmi2Real,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

/***************************************************
Types for Functions for FMI2 for Model Exchange
****************************************************/

/* Enter and exit the different modes */

// typedef fmi2Status fmi2EnterEventModeTYPE         (fmi2Component c);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2EnterEventMode(c: fmi2Component) -> fmi2Status {
    let instance = get_instance_mut!(c);
    assert_interface_type!(instance, InterfaceType::ModelExchange);
    assert_mode!(instance, ModelMode::ContinuousTimeMode);
    instance.data.mode = ModelMode::EventMode;
    fmi2OK
}

// typedef fmi2Status fmi2NewDiscreteStatesTYPE      (fmi2Component c, fmi2EventInfo* eventInfo);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2NewDiscreteStates(
    c: fmi2Component,
    fmi2eventInfo: *mut fmi2EventInfo,
) -> fmi2Status {
    let instance = get_instance_mut!(c);
    
    assert_interface_type!(instance, InterfaceType::ModelExchange);
    assert_mode!(instance, ModelMode::EventMode);
    
    instance.update_discrete_states();

    let event_info = unsafe { &mut *fmi2eventInfo };
    
    event_info.newDiscreteStatesNeeded = fmi2False;
    event_info.terminateSimulation = fmi2False;
    event_info.nominalsOfContinuousStatesChanged = fmi2False;
    event_info.valuesOfContinuousStatesChanged = fmi2True;
    event_info.nextEventTimeDefined = fmi2False;
    event_info.nextEventTime = 0.0;
    
    fmi2OK
}

// typedef fmi2Status fmi2EnterContinuousTimeModeTYPE(fmi2Component c);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2EnterContinuousTimeMode(c: fmi2Component) -> fmi2Status {
    let instance = get_instance_mut!(c);
    assert_interface_type!(instance, InterfaceType::ModelExchange);
    assert_mode!(instance, ModelMode::EventMode);
    instance.data.mode = ModelMode::ContinuousTimeMode;
    fmi2OK
}

// typedef fmi2Status fmi2CompletedIntegratorStepTYPE(fmi2Component c,
//                                                     fmi2Boolean   noSetFMUStatePriorToCurrentPoint,
//                                                     fmi2Boolean*  enterEventMode,
//                                                     fmi2Boolean*  terminateSimulation);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2CompletedIntegratorStep(
    c: fmi2Component,
    noSetFMUStatePriorToCurrentPoint: fmi2Boolean,
    enterEventMode: *mut fmi2Boolean,
    terminateSimulation: *mut fmi2Boolean,
) -> fmi2Status {
    let instance = get_instance_mut!(c);
    assert_interface_type!(instance, InterfaceType::ModelExchange);
    assert_mode!(instance, ModelMode::ContinuousTimeMode);
    fmi2OK
}

/* Providing independent variables and re-initialization of caching */

// typedef fmi2Status fmi2SetTimeTYPE            (fmi2Component c, fmi2Real time);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2SetTime(c: fmi2Component, time: fmi2Real) -> fmi2Status {
    let instance = get_instance_mut!(c);
    assert_interface_type!(instance, InterfaceType::ModelExchange);
    assert_mode!(instance, ModelMode::ContinuousTimeMode);
    instance.data.time = time;
    fmi2OK
}

// typedef fmi2Status fmi2SetContinuousStatesTYPE(fmi2Component c, const fmi2Real x[], size_t nx);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2SetContinuousStates(
    c: fmi2Component,
    x: *const fmi2Real,
    nx: usize,
) -> fmi2Status {
    let instance = get_instance_mut!(c);
    assert_not_null!(x, instance);
    let continuous_states = unsafe { std::slice::from_raw_parts(x, nx) };
    instance.set_continuous_states(continuous_states)
}

/* Evaluation of the model equations */

// typedef fmi2Status fmi2GetDerivativesTYPE               (fmi2Component c, fmi2Real derivatives[],     size_t nx);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetDerivatives(
    c: fmi2Component,
    derivatives: *mut fmi2Real,
    nx: usize,
) -> fmi2Status {
    let instance = get_instance!(c);
    assert_not_null!(derivatives, instance);
    let derivatives = unsafe { std::slice::from_raw_parts_mut(derivatives, nx) };
    instance.get_continuous_state_derivatives(derivatives)
}

// typedef fmi2Status fmi2GetEventIndicatorsTYPE           (fmi2Component c, fmi2Real eventIndicators[], size_t ni);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetEventIndicators(
    c: fmi2Component,
    eventIndicators: *mut fmi2Real,
    ni: usize,
) -> fmi2Status {
    let instance = get_instance!(c);
    assert_not_null!(eventIndicators, instance);
    let event_indicators = unsafe { std::slice::from_raw_parts_mut(eventIndicators, ni) };
    instance.get_event_indicators(event_indicators)
}   

// typedef fmi2Status fmi2GetContinuousStatesTYPE          (fmi2Component c, fmi2Real x[],               size_t nx);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetContinuousStates(
    c: fmi2Component,
    x: *mut fmi2Real,
    nx: usize,
) -> fmi2Status {
    let instance = get_instance!(c);
    assert_not_null!(x, instance);
    let continuous_states = unsafe { std::slice::from_raw_parts_mut(x, nx) };
    instance.get_continuous_states(continuous_states)
}   

// typedef fmi2Status fmi2GetNominalsOfContinuousStatesTYPE(fmi2Component c, fmi2Real x_nominal[],       size_t nx);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetNominalsOfContinuousStates(
    c: fmi2Component,
    x_nominal: *mut fmi2Real,
    nx: usize,
) -> fmi2Status {
    let instance = get_instance!(c);
    assert_not_null!(x_nominal, instance);
    let nominals = unsafe { std::slice::from_raw_parts_mut(x_nominal, nx) };
    instance.get_nominals_of_continuous_states(nominals)
}      

/***************************************************
Types for Functions for FMI2 for Co-Simulation
****************************************************/

/* Simulating the slave */

// typedef fmi2Status fmi2SetRealInputDerivativesTYPE (fmi2Component c,
//                                                     const fmi2ValueReference vr[], size_t nvr,
//                                                     const fmi2Integer order[],
//                                                     const fmi2Real value[]);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2SetRealInputDerivatives(
    c: fmi2Component,
    vr: *const fmi2ValueReference,
    nvr: usize,
    order: *const fmi2Integer,
    value: *const fmi2Real,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2GetRealOutputDerivativesTYPE(fmi2Component c,
//                                                     const fmi2ValueReference vr[], size_t nvr,
//                                                     const fmi2Integer order[],
//                                                     fmi2Real value[]);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetRealOutputDerivatives(
    c: fmi2Component,
    vr: *const fmi2ValueReference,
    nvr: usize,
    order: *const fmi2Integer,
    value: *mut fmi2Real,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2DoStepTYPE   (fmi2Component c,
//                                         fmi2Real      currentCommunicationPoint,
//                                         fmi2Real      communicationStepSize,
//                                         fmi2Boolean   noSetFMUStatePriorToCurrentPoint);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2DoStep(
    c: fmi2Component,
    currentCommunicationPoint: fmi2Real,
    communicationStepSize: fmi2Real,
    noSetFMUStatePriorToCurrentPoint: fmi2Boolean,
) -> fmi2Status {
    let instance = get_instance_mut!(c);
    loop {
        if instance.data.time + FIXED_STEP_SIZE >= currentCommunicationPoint + communicationStepSize {
            break;
        }
        instance.do_fixed_step(instance.data.time, FIXED_STEP_SIZE);
        instance.data.time += FIXED_STEP_SIZE;
        // instance.do_fixed_step();
        // if let Some(mut solver) = instance.data.solver.take() {
        //     let (update, status) = solver.step(instance, FIXED_STEP_SIZE);
        //     if status != fmi2OK {
        //         return status;
        //     }
        //     if update {
        //         instance.update_discrete_states();
        //         solver.reset(instance.get_number_of_continuous_states(), instance.get_number_of_event_indicators());
        //     }
        //     instance.data.solver = Some(solver);
        //     instance.data.time += FIXED_STEP_SIZE;
        // }
    }

    fmi2OK
}

// typedef fmi2Status fmi2CancelStepTYPE(fmi2Component c);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2CancelStep(c: fmi2Component) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

/* Inquire slave status */

// typedef fmi2Status fmi2GetStatusTYPE       (fmi2Component c, const fmi2StatusKind s, fmi2Status*  value);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetStatus(
    c: fmi2Component,
    s: fmi2StatusKind,
    value: *mut fmi2Status,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2GetRealStatusTYPE   (fmi2Component c, const fmi2StatusKind s, fmi2Real*    value);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetRealStatus(
    c: fmi2Component,
    s: fmi2StatusKind,
    value: *mut fmi2Real,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2GetIntegerStatusTYPE(fmi2Component c, const fmi2StatusKind s, fmi2Integer* value);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetIntegerStatus(
    c: fmi2Component,
    s: fmi2StatusKind,
    value: *mut fmi2Integer,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2GetBooleanStatusTYPE(fmi2Component c, const fmi2StatusKind s, fmi2Boolean* value);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetBooleanStatus(
    c: fmi2Component,
    s: fmi2StatusKind,
    value: *mut fmi2Boolean,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}

// typedef fmi2Status fmi2GetStringStatusTYPE (fmi2Component c, const fmi2StatusKind s, fmi2String*  value);
#[unsafe(no_mangle)]
pub extern "C" fn fmi2GetStringStatus(
    c: fmi2Component,
    s: fmi2StatusKind,
    value: *mut fmi2String,
) -> fmi2Status {
    NOT_IMPLEMENTED!(c)
}
