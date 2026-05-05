#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod csv;
pub mod input;
pub mod plot;
pub mod recorder;

use crate::{
    fmi2::{
        self, CS, FMU2, ME,
        types::{fmi2Boolean, fmi2False, fmi2Integer, fmi2Real, fmi2True},
    },
    model_description::{Causality, ModelDescription, ModelVariable, VariableType},
    sim::{
        SimulationSettings, SolverFactory,
        fmi2::{csv::read_csv, input::CSVInput, recorder::Recorder},
    },
    types::{
        fmiStatus::{self, fmiOK, fmiWarning},
        fmiValueReference,
    },
};

use std::{collections::HashMap, error::Error, fs::File};

#[derive(Debug, PartialEq)]
pub enum VariableValue {
    Real(fmi2Real),
    Integer(fmi2Integer),
    Boolean(fmi2Boolean),
    String(String),
}

impl VariableValue {
    pub fn to_f64(&self) -> f64 {
        if let VariableValue::Real(value) = self {
            *value
        } else {
            panic!("Expected a Real variable value, but got {:?}", self);
        }
    }

    pub fn to_literal(&self) -> String {
        match self {
            VariableValue::Real(v) => v.to_string(),
            VariableValue::Integer(v) => v.to_string(),
            VariableValue::Boolean(v) => v.to_string(),
            VariableValue::String(v) => v.clone(),
        }
    }
}

#[derive(Debug)]
pub struct SimulationResult<'a> {
    pub variables: Vec<&'a ModelVariable>,
    pub time: Vec<f64>,
    pub rows: Vec<Vec<VariableValue>>,
}

impl<'a> SimulationResult<'a> {
    pub fn new(variables: Vec<&'a ModelVariable>) -> Self {
        SimulationResult {
            variables,
            time: vec![],
            rows: vec![],
        }
    }
}

fn call(status: fmiStatus) -> Result<fmiStatus, Box<dyn Error>> {
    if matches!(status, fmiOK | fmiWarning) {
        Ok(status)
    } else {
        Err(format!("FMI call failed with status: {:?}", status).into())
    }
}

pub fn parse_variable_value(
    variable_type: &VariableType,
    literal: &str,
) -> Result<VariableValue, Box<dyn Error>> {
    match variable_type {
        VariableType::Float64 => Ok(VariableValue::Real(literal.parse()?)),
        VariableType::Int32 | VariableType::Enumeration => {
            Ok(VariableValue::Integer(literal.parse()?))
        }
        VariableType::Boolean => {
            let value: bool = literal.parse()?;
            Ok(VariableValue::Boolean(if value {
                fmi2True
            } else {
                fmi2False
            }))
        }
        VariableType::String => Ok(VariableValue::String(literal.to_string())),
        _ => Err(format!("Unsupported variable type {variable_type:?}.").into()),
    }
}

pub fn set_variable_value<T>(
    fmu: &FMU2<T>,
    value_reference: fmiValueReference,
    value: &VariableValue,
) -> Result<fmiStatus, Box<dyn Error>> {
    match value {
        VariableValue::Real(value) => call(fmu.setReal(&[value_reference], &[*value])),
        VariableValue::Integer(value) => call(fmu.setInteger(&[value_reference], &[*value])),
        VariableValue::Boolean(value) => call(fmu.setBoolean(&[value_reference], &[*value])),
        VariableValue::String(value) => call(fmu.setString(&[value_reference], &[value.as_str()])),
    }
}

fn set_start_values<T>(
    start_values: &Vec<(String, String)>,
    model_description: &ModelDescription,
    fmu: &FMU2<T>,
) -> Result<fmiStatus, Box<dyn Error>> {
    // Create a map for quick lookup of variables by name
    let variable_map: HashMap<&str, &ModelVariable> = model_description
        .modelVariables
        .iter()
        .map(|var| (var.name.as_str(), var))
        .collect();

    // then the remaining start values
    for (var_name, literal) in start_values {
        if let Some(variable) = variable_map.get(var_name.as_str()) {
            if variable.causality == Causality::StructuralParameter {
                continue;
            }

            match parse_variable_value(&variable.variableType, literal) {
                Ok(value) => {
                    set_variable_value(fmu, variable.valueReference, &value)?;
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
    simulation_result: &mut SimulationResult,
) -> Result<(), Box<dyn Error>> {
    let start_time = settings.start_time;
    let stop_time = settings.stop_time;
    let set_stop_time = settings.set_stop_time;
    let output_interval = settings.output_interval;

    let mut time = start_time;

    let co_simulation = match &settings.model_description.coSimulation {
        Some(cs) => cs,
        None => {
            return Err("The FMU does not support Co-Simulation.".into());
        }
    };

    let can_handle_variable_communication_step_size =
        co_simulation.canHandleVariableCommunicationStepSize.clone();

    let input = if let Some(path) = &settings.input_file {
        let file = File::open(&path)?;
        let trajectories = read_csv(&file, &settings.model_description)?;
        Some(CSVInput::new(trajectories))
    } else {
        None
    };

    let fmu = FMU2::<CS>::new(
        settings.unzipdir.as_ref(),
        &co_simulation.modelIdentifier,
        &settings.model_description.modelName,
        &settings.model_description.instantiationToken,
        false,
        settings.logging_on,
        settings.log_fmi_calls,
        true,
        true,
        true,
        !co_simulation.canNotUseMemoryManagementFunctions,
    )?;

    set_start_values(&settings.start_values, &settings.model_description, &fmu)?;

    call(fmu.setupExperiment(
        settings.tolerance,
        time,
        if set_stop_time { Some(stop_time) } else { None },
    ))?;

    call(fmu.enterInitializationMode())?;

    if let Some(input) = &input {
        input.set_discrete_inputs(time, true, &fmu)?;
        input.set_continuous_inputs(time, true, &fmu)?;
    }

    call(fmu.exitInitializationMode())?;

    let mut recorder = Recorder::new(&fmu, simulation_result);

    recorder.sample(time)?;

    let mut n_steps = 0;

    loop {
        if time > stop_time || relative_eq!(time, stop_time) {
            break;
        }

        let next_regular_point = start_time + (n_steps + 1) as f64 * output_interval;

        let mut next_communication_point = next_regular_point;

        if can_handle_variable_communication_step_size
            && let Some(input) = &input
            && let Some(next_input_event_time) = input.next_event_time(time)
            && next_regular_point > next_input_event_time
            && !relative_eq!(next_regular_point, next_input_event_time)
        {
            next_communication_point = next_input_event_time;
        };

        if next_communication_point > stop_time
            && !relative_eq!(next_communication_point, stop_time)
        {
            if can_handle_variable_communication_step_size {
                next_communication_point = stop_time;
            } else {
                break;
            }
        }

        let communication_step_size = next_communication_point - time;

        if let Some(input) = &input {
            input.set_discrete_inputs(time, true, &fmu)?;
            input.set_continuous_inputs(time, true, &fmu)?;
        }

        let do_step_status = fmu.doStep(time, communication_step_size, 0);

        let mut terminate_simulation = 0;

        if do_step_status == fmiStatus::fmiDiscard {
            call(fmu.getRealStatus(
                &fmi2::types::fmi2StatusKind::fmi2LastSuccessfulTime,
                &mut time,
            ))?;
            call(fmu.getBooleanStatus(
                &fmi2::types::fmi2StatusKind::fmi2Terminated,
                &mut terminate_simulation,
            ))?;
        } else {
            call(do_step_status)?;
            time = next_communication_point;
        }

        if relative_eq!(time, next_communication_point) {
            n_steps += 1;
        }

        recorder.sample(time)?;

        if terminate_simulation != 0 {
            break;
        }
    }

    call(fmu.terminate())?;

    Ok(())
}

pub fn simulate_me<S: SolverFactory>(
    settings: &SimulationSettings,
    solver_factory: &S,
    simulation_result: &mut SimulationResult,
) -> Result<(), Box<dyn Error>> {
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

    let input = if let Some(path) = &settings.input_file {
        let file = File::open(&path)?;
        let trajectories = read_csv(&file, &settings.model_description)?;
        Some(CSVInput::new(trajectories))
    } else {
        None
    };

    let fmu = FMU2::<ME>::new(
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
        !model_exchange.canNotUseMemoryManagementFunctions,
    )?;

    set_start_values(&settings.start_values, &settings.model_description, &fmu)?;

    call(fmu.setupExperiment(
        settings.tolerance,
        time,
        if set_stop_time { Some(stop_time) } else { None },
    ))?;

    call(fmu.enterInitializationMode())?;

    if let Some(input) = &input {
        input.set_discrete_inputs(time, true, &fmu)?;
        input.set_continuous_inputs(time, true, &fmu)?;
    }

    call(fmu.exitInitializationMode())?;

    let mut nextEventTime: Option<fmi2Real> = None;

    loop {
        let mut newDiscreteStatesNeeded: bool = false;
        let mut terminateSimulation: bool = false;
        let mut _nominalsOfContinuousStatesChanged: bool = false;
        let mut _valuesOfContinuousStatesChanged: bool = false;

        call(fmu.newDiscreteStates(
            &mut newDiscreteStatesNeeded,
            &mut terminateSimulation,
            &mut _nominalsOfContinuousStatesChanged,
            &mut _valuesOfContinuousStatesChanged,
            &mut nextEventTime,
        ))?;

        if terminateSimulation {
            call(fmu.terminate())?;
            return Ok(());
        }

        if !newDiscreteStatesNeeded {
            break;
        }
    }

    call(fmu.enterContinuousTimeMode())?;

    let mut recorder = Recorder::new(&fmu, simulation_result);
    let mut solver = solver_factory.create(
        time,
        settings.model_description.derivatives.len(),
        settings.model_description.numberOfEventIndicators,
        settings.tolerance.unwrap_or(1e-4),
        vec![0],
        vec![0],
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
            fmu.getDerivatives(state_derivatives);
            Ok(())
        }),
        if model_exchange.providesDirectionalDerivatives {
            Some(Box::new(|unknowns, knowns, seed, sensitivity| {
                fmu.getDirectionalDerivative(unknowns, knowns, seed, sensitivity);
                Ok(())
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
        recorder.sample(time)?;

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

        if let Some(next_event_time) = nextEventTime
            && next_communication_point > next_event_time
            && !relative_eq!(next_communication_point, next_event_time)
        {
            next_communication_point = next_event_time;
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
            nextEventTime.is_some_and(|t| relative_eq!(t, next_communication_point));

        let (time_reached, is_state_event) = solver.step(next_communication_point)?;

        time = time_reached;

        if is_input_event && let Some(input) = &input {
            input.set_continuous_inputs(time, false, &fmu)?;
        }

        if relative_eq!(time, next_regular_point) {
            n_steps += 1;
        }

        let is_step_event = if needs_completed_integrator_step {
            let mut is_step_event = fmi2False;
            let mut terminate_simulation = fmi2False;

            call(fmu.completedIntegratorStep(
                fmi2False,
                &mut is_step_event,
                &mut terminate_simulation,
            ))?;

            if terminate_simulation != fmi2False {
                call(fmu.terminate())?;
                return Ok(());
            }

            is_step_event != fmi2False
        } else {
            false
        };

        if is_input_event || is_time_event || is_state_event || is_step_event {
            recorder.sample(time)?;

            call(fmu.enterEventMode())?;

            if is_input_event && let Some(input) = &input {
                input.set_discrete_inputs(time, true, &fmu)?;
                input.set_continuous_inputs(time, true, &fmu)?;
            }

            loop {
                let mut newDiscreteStatesNeeded: bool = false;
                let mut terminateSimulation: bool = false;
                let mut _nominalsOfContinuousStatesChanged: bool = false;
                let mut _valuesOfContinuousStatesChanged: bool = false;

                call(fmu.newDiscreteStates(
                    &mut newDiscreteStatesNeeded,
                    &mut terminateSimulation,
                    &mut _nominalsOfContinuousStatesChanged,
                    &mut _valuesOfContinuousStatesChanged,
                    &mut nextEventTime,
                ))?;

                if terminateSimulation {
                    call(fmu.terminate())?;
                    return Ok(());
                }

                if !newDiscreteStatesNeeded {
                    break;
                }
            }

            call(fmu.enterContinuousTimeMode())?;

            solver.reset(time)?;
        }
    }

    call(fmu.terminate())?;

    Ok(())
}
