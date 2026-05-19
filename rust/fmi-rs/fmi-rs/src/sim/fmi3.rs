pub mod csv;
pub mod input;
pub mod recorder;

use std::{collections::HashMap, error::Error, fs::File};

use crate::{
    fmi3::{FMU3, types::*},
    model_description::fmi3::{ModelVariable, VariableType},
    sim::{
        SolverFactory,
        fmi3::{csv::read_csv, input::StaticInput, recorder::Recorder},
    },
    types::*,
};
use crate::{
    model_description::fmi3::{Causality, ModelDescription},
    types::fmiStatus::{self, fmiOK, fmiWarning},
};

use std::path::{Path, PathBuf};

pub struct SimulationSettings<'a> {
    pub unzipdir: &'a Path,
    pub model_description: &'a ModelDescription,
    pub start_time: f64,
    pub stop_time: f64,
    pub logging_on: bool,
    pub set_stop_time: bool,
    pub output_interval: f64,
    pub tolerance: Option<f64>,
    pub start_values: Vec<(String, String)>,
    pub log_fmi_calls: bool,
    pub input_file: Option<PathBuf>,
    pub early_return_allowed: bool,
    pub event_mode_used: bool,
}

#[derive(Debug, PartialEq)]
pub enum VariableValue {
    Float32(Vec<fmi3Float32>),
    Float64(Vec<fmi3Float64>),
    Int8(Vec<fmi3Int8>),
    UInt8(Vec<fmi3UInt8>),
    Int16(Vec<fmi3Int16>),
    UInt16(Vec<fmi3UInt16>),
    Int32(Vec<fmi3Int32>),
    UInt32(Vec<fmi3UInt32>),
    Int64(Vec<fmi3Int64>),
    UInt64(Vec<fmi3UInt64>),
    Boolean(Vec<fmi3Boolean>),
    String(Vec<String>),
    Binary(Vec<Vec<fmi3Byte>>),
    // Clock(fmiClock),
}

impl VariableValue {
    pub fn len(&self) -> usize {
        match self {
            VariableValue::Float32(v) => v.len(),
            VariableValue::Float64(v) => v.len(),
            VariableValue::Int8(v) => v.len(),
            VariableValue::UInt8(v) => v.len(),
            VariableValue::Int16(v) => v.len(),
            VariableValue::UInt16(v) => v.len(),
            VariableValue::Int32(v) => v.len(),
            VariableValue::UInt32(v) => v.len(),
            VariableValue::Int64(v) => v.len(),
            VariableValue::UInt64(v) => v.len(),
            VariableValue::Boolean(v) => v.len(),
            VariableValue::String(v) => v.len(),
            VariableValue::Binary(v) => v.len(),
        }
    }

    pub fn to_literal(&self) -> String {
        match self {
            VariableValue::Float32(v) => v
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            VariableValue::Float64(v) => v
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            VariableValue::Int8(v) => v
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            VariableValue::UInt8(v) => v
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            VariableValue::Int16(v) => v
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            VariableValue::UInt16(v) => v
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            VariableValue::Int32(v) => v
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            VariableValue::UInt32(v) => v
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            VariableValue::Int64(v) => v
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            VariableValue::UInt64(v) => v
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            VariableValue::Boolean(v) => v
                .iter()
                .map(|&b| if b { "true" } else { "false" })
                .collect::<Vec<_>>()
                .join(" "),
            VariableValue::String(v) => v.join(" "),
            VariableValue::Binary(v) => v
                .iter()
                .map(|bytes| {
                    bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

    pub fn as_f64(&self) -> &Vec<f64> {
        match self {
            VariableValue::Float64(v) => v,
            _ => panic!("VariableValue is not a Float64"),
        }
    }
}

#[derive(Debug)]
pub struct Trajectories<'a> {
    pub model_description: &'a ModelDescription,
    pub variables: Vec<&'a ModelVariable>,
    pub time: Vec<f64>,
    pub rows: Vec<Vec<VariableValue>>,
}

impl<'a> Trajectories<'a> {
    pub fn new(model_description: &'a ModelDescription, variables: Vec<&'a ModelVariable>) -> Self {
        Trajectories {
            model_description,
            variables,
            time: vec![],
            rows: vec![],
        }
    }

    /// Return a list of all event times
    pub fn events(&self) -> Vec<f64> {
        let mut events = vec![];

        for t in self.time.windows(2).filter(|t| t[0] == t[1]) {
            if events.last() != Some(&t[0]) {
                events.push(t[0]);
            }
        }
        
        events
    }
}

pub fn parse_variable_value(
    variable_type: &VariableType,
    literal: &str,
) -> Result<VariableValue, Box<dyn Error>> {
    match variable_type {
        VariableType::Float32 { .. } => {
            let values: Result<Vec<fmiFloat32>, _> =
                literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Float32(values?))
        }
        VariableType::Float64 { .. } => {
            let values: Result<Vec<fmiFloat64>, _> =
                literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Float64(values?))
        }
        VariableType::Int8 { .. } => {
            let values: Result<Vec<fmiInt8>, _> =
                literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Int8(values?))
        }
        VariableType::UInt8 { .. } => {
            let values: Result<Vec<fmiUInt8>, _> =
                literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::UInt8(values?))
        }
        VariableType::Int16 { .. } => {
            let values: Result<Vec<fmiInt16>, _> =
                literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Int16(values?))
        }
        VariableType::UInt16 { .. } => {
            let values: Result<Vec<fmiUInt16>, _> =
                literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::UInt16(values?))
        }
        VariableType::Int32 { .. } => {
            let values: Result<Vec<fmiInt32>, _> =
                literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Int32(values?))
        }
        VariableType::UInt32 { .. } => {
            let values: Result<Vec<fmiUInt32>, _> =
                literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::UInt32(values?))
        }
        VariableType::Int64 { .. } | VariableType::Enumeration { .. } => {
            let values: Result<Vec<fmiInt64>, _> =
                literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Int64(values?))
        }
        VariableType::UInt64 { .. } => {
            let values: Result<Vec<fmiUInt64>, _> =
                literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::UInt64(values?))
        }
        VariableType::Boolean { .. } | VariableType::Clock { .. } => {
            let values: Result<Vec<fmiBoolean>, _> =
                literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Boolean(values?))
        }
        VariableType::String { .. } => {
            let values: Vec<String> = literal.split_whitespace().map(|v| v.to_string()).collect();
            Ok(VariableValue::String(values))
        }
        VariableType::Binary { .. } => {
            let values: Result<Vec<Vec<fmiByte>>, Box<dyn Error>> = literal
                .split_whitespace()
                .map(|hex_str| {
                    if hex_str.len() % 2 != 0 {
                        return Err(format!("Invalid hex string length: {}", hex_str).into());
                    }

                    let mut bytes = Vec::new();

                    for i in (0..hex_str.len()).step_by(2) {
                        let byte_str = &hex_str[i..i + 2];
                        match u8::from_str_radix(byte_str, 16) {
                            Ok(byte) => bytes.push(byte),
                            Err(e) => {
                                return Err(
                                    format!("Invalid hex byte '{}': {}", byte_str, e).into()
                                );
                            }
                        }
                    }

                    Ok(bytes)
                })
                .collect();
            Ok(VariableValue::Binary(values?))
        }
    }
}

pub fn set_variable_value(
    fmu: &FMU3,
    value_reference: fmiValueReference,
    value: &VariableValue,
) -> fmiStatus {
    match value {
        VariableValue::Float32(values) => fmu.setFloat32(&[value_reference], values),
        VariableValue::Float64(values) => fmu.setFloat64(&[value_reference], values),
        VariableValue::Int8(values) => fmu.setInt8(&[value_reference], values),
        VariableValue::UInt8(values) => fmu.setUInt8(&[value_reference], values),
        VariableValue::Int16(values) => fmu.setInt16(&[value_reference], values),
        VariableValue::UInt16(values) => fmu.setUInt16(&[value_reference], values),
        VariableValue::Int32(values) => fmu.setInt32(&[value_reference], values),
        VariableValue::UInt32(values) => fmu.setUInt32(&[value_reference], values),
        VariableValue::Int64(values) => fmu.setInt64(&[value_reference], values),
        VariableValue::UInt64(values) => fmu.setUInt64(&[value_reference], values),
        VariableValue::Boolean(values) => fmu.setBoolean(&[value_reference], values),
        VariableValue::String(values) => {
            let string_refs: Vec<&str> = values.iter().map(|x| x.as_str()).collect();
            fmu.setString(&[value_reference], &string_refs)
        }
        VariableValue::Binary(values) => fmu.setBinary(&[value_reference], &values),
    }
}

pub fn call(status: fmiStatus) -> Result<fmiStatus, Box<dyn Error>> {
    if matches!(status, fmiOK | fmiWarning) {
        Ok(status)
    } else {
        Err(format!("FMI call failed with status: {:?}", status).into())
    }
}

fn set_start_values(
    start_values: &Vec<(String, String)>,
    model_description: &ModelDescription,
    fmu: &FMU3,
) -> Result<fmiStatus, Box<dyn Error>> {
    let mut configuration_mode = false;

    // Create a map for quick lookup of variables by name
    let variable_map: HashMap<&str, &ModelVariable> = model_description
        .modelVariables
        .iter()
        .map(|var| (var.name.as_str(), var))
        .collect();

    // set structural parameters first
    for (var_name, value) in start_values {
        if let Some(variable) = variable_map.get(var_name.as_str())
            && variable.causality == Causality::StructuralParameter
        {
            if !configuration_mode {
                call(fmu.enterConfigurationMode())?;
                configuration_mode = true;
            }

            let value_references = [variable.valueReference];
            let values: Result<Vec<u64>, _> = value.split_whitespace().map(|v| v.parse()).collect();
            match values {
                Ok(vals) => {
                    fmu.setUInt64(&value_references, &vals);
                }
                Err(_) => {
                    return Err(format!(
                        "Invalid integer value {value:?} for variable {var_name:?}."
                    )
                    .into());
                }
            }
        }
    }

    if configuration_mode {
        call(fmu.exitConfigurationMode())?;
    }

    // then the remaining start values
    for (var_name, literal) in start_values {
        if let Some(variable) = variable_map.get(var_name.as_str()) {
            if variable.causality == Causality::StructuralParameter {
                continue;
            }

            match parse_variable_value(&variable.variableType, literal) {
                Ok(value) => {
                    set_variable_value(fmu, variable.valueReference, &value);
                }
                Err(e) => {
                    return Err(format!(
                        "Invalid value {literal:?} for variable {var_name:?}. {e}"
                    )
                    .into());
                }
            }
        }
    }

    Ok(fmiOK)
}

pub fn simulate_cs(
    settings: &SimulationSettings,
    input: Option<&StaticInput>,
    recorder: &mut Recorder,
) -> Result<(), Box<dyn Error>> {
    let start_time = settings.start_time;
    let stop_time = settings.stop_time;
    let set_stop_time = settings.set_stop_time;
    let output_interval = settings.output_interval;
    let event_mode_used = settings.event_mode_used;

    let mut time = start_time;

    let co_simulation = match &settings.model_description.coSimulation {
        Some(cs) => cs,
        None => {
            return Err("The FMU does not support Co-Simulation.".into());
        }
    };

    let can_handle_variable_communication_step_size =
        co_simulation.canHandleVariableCommunicationStepSize.clone();

    let fmu = FMU3::instantiateCoSimulation(
        settings.unzipdir.as_ref(),
        &co_simulation.modelIdentifier,
        &settings.model_description.modelName,
        &settings.model_description.instantiationToken,
        false,
        settings.logging_on,
        settings.event_mode_used,
        settings.early_return_allowed,
        &[],
        settings.log_fmi_calls,
        true,
        true,
        true,
    )?;

    set_start_values(&settings.start_values, &settings.model_description, &fmu)?;

    call(fmu.enterInitializationMode(
        settings.tolerance,
        start_time,
        if set_stop_time { Some(stop_time) } else { None },
    ))?;

    if let Some(input) = &input {
        input.set_discrete_inputs(time, true, &fmu)?;
        input.set_continuous_inputs(time, true, &fmu)?;
    }

    call(fmu.exitInitializationMode())?;

    if event_mode_used {
        loop {
            let mut discreteStatesNeedUpdate = false;
            let mut terminateSimulation = false;
            let mut nominalsOfContinuousStatesChanged = false;
            let mut valuesOfContinuousStatesChanged = false;
            let mut nextEventTimeDefined = false;
            let mut nextEventTime = 0.0;

            call(fmu.updateDiscreteStates(
                &mut discreteStatesNeedUpdate,
                &mut terminateSimulation,
                &mut nominalsOfContinuousStatesChanged,
                &mut valuesOfContinuousStatesChanged,
                &mut nextEventTimeDefined,
                &mut nextEventTime,
            ))?;

            if terminateSimulation {
                call(fmu.terminate())?;
                return Ok(());
            }

            if !discreteStatesNeedUpdate {
                break;
            }
        }

        call(fmu.enterStepMode())?;
    }

    recorder.sample(time, &fmu)?;

    let mut n_steps = 0;

    let mut input_applied = false;

    loop {
        if time > stop_time || relative_eq!(time, stop_time) {
            break;
        }

        let next_regular_point = start_time + (n_steps + 1) as f64 * output_interval;

        let mut next_communication_point = next_regular_point;

        let next_input_event_time = if let Some(input) = &input {
            input.next_event_time(time)
        } else {
            None
        };

        if let Some(next_input_event_time) = next_input_event_time
            && can_handle_variable_communication_step_size
            && next_communication_point > next_input_event_time
            && !relative_eq!(next_regular_point, next_input_event_time)
        {
            next_communication_point = next_input_event_time;
        }

        if next_communication_point > stop_time
            && !relative_eq!(next_communication_point, stop_time)
        {
            if can_handle_variable_communication_step_size {
                next_communication_point = stop_time;
            } else {
                break;
            }
        }

        if !input_applied && let Some(input) = &input {
            input.set_discrete_inputs(time, !event_mode_used, &fmu)?;
            input.set_continuous_inputs(time, !event_mode_used, &fmu)?;
        }

        let communication_step_size = next_communication_point - time;
        let mut event_handling_needed = false;
        let mut terminate_simulation = false;
        let mut early_return = false;
        let mut last_successful_time = 0.0;

        call(fmu.doStep(
            time,
            communication_step_size,
            true,
            &mut event_handling_needed,
            &mut terminate_simulation,
            &mut early_return,
            &mut last_successful_time,
        ))?;

        if early_return && !settings.early_return_allowed {
            return Err(
                "The FMU returned early from fmi3DoStep() but early return is not allowed.".into(),
            );
        }

        time = if early_return && last_successful_time < next_communication_point {
            last_successful_time
        } else {
            next_communication_point
        };

        if relative_eq!(time, next_regular_point) {
            n_steps += 1;
        }

        recorder.sample(time, &fmu)?;

        if terminate_simulation {
            call(fmu.terminate())?;
            return Ok(());
        }

        let input_event = if let Some(next_input_event_time) = next_input_event_time {
            relative_eq!(next_communication_point, next_input_event_time)
        } else {
            false
        };

        input_applied = if event_mode_used && (input_event || event_handling_needed) {
            call(fmu.enterEventMode())?;

            if input_event && let Some(input) = &input {
                input.set_discrete_inputs(time, true, &fmu)?;
                input.set_continuous_inputs(time, true, &fmu)?;
            }

            loop {
                let mut discreteStatesNeedUpdate = false;
                let mut terminateSimulation = false;
                let mut nominalsOfContinuousStatesChanged = false;
                let mut valuesOfContinuousStatesChanged = false;
                let mut nextEventTimeDefined = false;
                let mut nextEventTime = 0.0;

                call(fmu.updateDiscreteStates(
                    &mut discreteStatesNeedUpdate,
                    &mut terminateSimulation,
                    &mut nominalsOfContinuousStatesChanged,
                    &mut valuesOfContinuousStatesChanged,
                    &mut nextEventTimeDefined,
                    &mut nextEventTime,
                ))?;

                if terminateSimulation {
                    call(fmu.terminate())?;
                    return Ok(());
                }

                if !discreteStatesNeedUpdate {
                    break;
                }
            }

            call(fmu.enterStepMode())?;

            recorder.sample(time, &fmu)?;

            true
        } else {
            false
        };
    }

    call(fmu.terminate())?;

    Ok(())
}

pub fn simulate_me<S: SolverFactory>(
    settings: &SimulationSettings,
    solver_factory: &S,
    input: Option<&StaticInput>,
    recorder: &mut Recorder,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = settings.start_time;
    let stop_time = settings.stop_time;
    let set_stop_time = settings.set_stop_time;
    let output_interval = settings.output_interval;

    let mut time = start_time;

    let model_exchange = match &settings.model_description.modelExchange {
        Some(me) => me,
        None => {
            return Err("The FMU does not support Model Exchange.".into());
        }
    };

    let needs_completed_integrator_step = model_exchange.needsCompletedIntegratorStep;

    let fmu = FMU3::instantiateModelExchange(
        settings.unzipdir.as_ref(),
        &model_exchange.modelIdentifier,
        &settings.model_description.modelName,
        &settings.model_description.instantiationToken,
        false,
        settings.logging_on,
        settings.log_fmi_calls,
        true,
        true,
        true,
    )?;

    set_start_values(&settings.start_values, &settings.model_description, &fmu)?;

    call(fmu.enterInitializationMode(
        settings.tolerance,
        time,
        if set_stop_time { Some(stop_time) } else { None },
    ))?;

    if let Some(input) = &input {
        input.set_discrete_inputs(time, false, &fmu)?;
        input.set_continuous_inputs(time, false, &fmu)?;
    }

    call(fmu.exitInitializationMode())?;

    let mut discreteStatesNeedUpdate = false;
    let mut terminateSimulation = false;
    let mut nominalsOfContinuousStatesChanged = false;
    let mut valuesOfContinuousStatesChanged = false;
    let mut nextEventTimeDefined = false;
    let mut nextEventTime = 0.0;

    // initial event iteration
    loop {
        call(fmu.updateDiscreteStates(
            &mut discreteStatesNeedUpdate,
            &mut terminateSimulation,
            &mut nominalsOfContinuousStatesChanged,
            &mut valuesOfContinuousStatesChanged,
            &mut nextEventTimeDefined,
            &mut nextEventTime,
        ))?;

        if terminateSimulation {
            call(fmu.terminate())?;
            return Ok(());
        }

        if !discreteStatesNeedUpdate {
            break;
        }
    }

    call(fmu.enterContinuousTimeMode())?;

    // create a HashMap value reference -> variable
    let variables_map: HashMap<u32, &ModelVariable> = settings
        .model_description
        .modelVariables
        .iter()
        .map(|v| (v.valueReference, v))
        .collect();

    // Get Continuous States and Derivatives dynamically to ensure correct order
    let derivative_vrs: Vec<u32> = settings
        .model_description
        .derivatives
        .iter()
        .map(|d| d.valueReference)
        .collect();

    let state_vrs: Vec<u32> = derivative_vrs
        .iter()
        .map(|s| {
            let derivative_variable = variables_map[s];
            if let VariableType::Float64 {
                derivative: Some(vr),
                ..
            }
            | VariableType::Float32 {
                derivative: Some(vr),
                ..
            } = derivative_variable.variableType
            {
                vr
            } else {
                panic!(
                    "Derivative variable with value reference {} is not of type Float32 or Float64",
                    s
                );
            }
        })
        .collect();

    let mut solver = solver_factory.create(
        time,
        settings.model_description.derivatives.len(),
        settings.model_description.eventIndicators.len(),
        settings.tolerance.unwrap_or(1e-6),
        derivative_vrs,
        state_vrs,
        Box::new(|time| {
            fmu.setTime(time);
            Ok(())
        }),
        Box::new(|time| {
            if let Some(input) = &input {
                input.set_continuous_inputs(time, false, &fmu)?;
            }
            Ok(())
        }),
        Box::new(|event_indicators| {
            fmu.getEventIndicators(event_indicators);
            Ok(())
        }),
        Box::new(|continuous_states| {
            fmu.getContinuousStates(continuous_states);
            Ok(())
        }),
        Box::new(|nominals| {
            fmu.getNominalsOfContinuousStates(nominals);
            Ok(())
        }),
        Box::new(|state_derivatives| {
            fmu.getContinuousStateDerivatives(state_derivatives);
            Ok(())
        }),
        if model_exchange.providesDirectionalDerivatives {
            Some(Box::new(|unknowns, knowns, seed, sensitivity| {
                let status = fmu.getDirectionalDerivative(unknowns, knowns, seed, sensitivity);
                if status == fmiOK {
                    Ok(())
                } else {
                    Err("Failed to get directional derivative".into())
                }
            }))
        } else {
            None
        },
        Box::new(|continuous_states| {
            fmu.setContinuousStates(continuous_states);
            Ok(())
        }),
    )?;

    let mut n_steps = 0;

    loop {
        recorder.sample(time, &fmu)?;

        if time > stop_time || relative_eq!(time, stop_time) {
            break;
        }

        let next_regular_point = start_time + (n_steps + 1) as f64 * output_interval;

        let mut next_communication_point = next_regular_point;

        let next_input_event_time = if let Some(input) = &input {
            input.next_event_time(time)
        } else {
            None
        };

        if let Some(next_input_event_time) = next_input_event_time
            && next_regular_point > next_input_event_time
            && !relative_eq!(next_regular_point, next_input_event_time)
        {
            next_communication_point = next_input_event_time;
        }

        if nextEventTimeDefined
            && next_communication_point > nextEventTime
            && !relative_eq!(next_communication_point, nextEventTime)
        {
            next_communication_point = nextEventTime;
        }

        if next_communication_point > stop_time
            && !relative_eq!(next_communication_point, stop_time)
        {
            next_communication_point = stop_time;
        }

        let is_input_event = if let Some(input_event_time) = next_input_event_time {
            relative_eq!(input_event_time, next_communication_point)
        } else {
            false
        };

        let is_time_event =
            nextEventTimeDefined && relative_eq!(nextEventTime, next_communication_point);

        let (time_reached, is_state_event) = solver.step(next_communication_point)?;

        time = time_reached;

        if is_input_event && let Some(input) = &input {
            input.set_continuous_inputs(time, false, &fmu)?;
        }

        if relative_eq!(time, next_regular_point) {
            n_steps += 1;
        }

        let mut is_step_event = false;

        if needs_completed_integrator_step {
            let mut terminate_simulation = false;

            call(fmu.completedIntegratorStep(
                false,
                &mut is_step_event,
                &mut terminate_simulation,
            ))?;

            if terminate_simulation {
                call(fmu.terminate())?;
                return Ok(());
            }
        }

        if is_input_event || is_time_event || is_state_event || is_step_event {
            recorder.sample(time, &fmu)?;

            call(fmu.enterEventMode())?;

            if is_input_event && let Some(input) = &input {
                input.set_discrete_inputs(time, true, &fmu)?;
                input.set_continuous_inputs(time, true, &fmu)?;
            }

            let mut discreteStatesNeedUpdate = true;
            let mut terminateSimulation = false;
            let mut _nominalsOfContinuousStatesChanged = false;
            let mut _valuesOfContinuousStatesChanged = false;

            while discreteStatesNeedUpdate {
                call(fmu.updateDiscreteStates(
                    &mut discreteStatesNeedUpdate,
                    &mut terminateSimulation,
                    &mut nominalsOfContinuousStatesChanged,
                    &mut valuesOfContinuousStatesChanged,
                    &mut nextEventTimeDefined,
                    &mut nextEventTime,
                ))?;

                if terminateSimulation {
                    call(fmu.terminate())?;
                    return Ok(());
                }
            }

            call(fmu.enterContinuousTimeMode())?;

            solver.reset(time)?;
        }
    }

    call(fmu.terminate())?;

    Ok(())
}
