#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub mod input;
pub mod recorder;

use crate::{
    fmi2::{
        self, CS, FMU2, ME,
        types::{fmi2Boolean, fmi2False, fmi2Real, fmi2True},
    },
    model_description::{Causality, ModelDescription, ModelVariable, VariableType},
    sim::{
        SimulationSettings, SolverFactory,
        fmi2::{input::CSVInput, recorder::Recorder},
    },
    types::{
        fmiStatus::{self, fmiOK, fmiWarning},
        fmiValueReference,
    },
    util::VariableValue,
};

use std::{
    collections::HashMap,
    error::Error,
    fs::File,
};

#[derive(Debug, PartialEq)]
pub enum Trajectory {
    Real(Vec<f64>),
    Integer(Vec<i32>),
    Boolean(Vec<i32>),
    String(Vec<String>),
}

pub struct SimulationResult<'a> {
    pub variables: Vec<&'a ModelVariable>,
    pub time: Vec<f64>,
    pub trajectories: Vec<Trajectory>,
}

impl<'a> SimulationResult<'a> {
    pub fn new(variables: Vec<&'a ModelVariable>) -> Self {
        let trajectories = variables.iter().map(|variable| match variable.variableType {
            crate::model_description::VariableType::Float64 => Trajectory::Real(vec![]),
            crate::model_description::VariableType::Int32 | crate::model_description::VariableType::Enumeration => Trajectory::Integer(vec![]),
            crate::model_description::VariableType::Boolean => Trajectory::Boolean(vec![]),
            crate::model_description::VariableType::String => Trajectory::String(vec![]),
            _ => panic!("Unexpected variable type: {:?}", variable.variableType),
        }).collect();
        SimulationResult { variables, time: vec![], trajectories }
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
        VariableType::Float64 => {
            let value: Result<f64, _> = literal.parse();
            Ok(VariableValue::Float64(vec![value?]))
        }
        VariableType::Int32 | VariableType::Enumeration => {
            let value: Result<i32, _> = literal.parse();
            Ok(VariableValue::Int32(vec![value?]))
        }
        VariableType::Boolean => {
            let value: Result<bool, _> = literal.parse();
            Ok(VariableValue::Boolean(vec![value?]))
        }
        VariableType::String => {
            let values: Vec<String> = literal.split_whitespace().map(|v| v.to_string()).collect();
            Ok(VariableValue::String(values))
        }
        _ => Err(format!("Unsupported variable type {variable_type:?}.").into()),
    }
}

pub fn set_variable_value<T>(
    fmu: &FMU2<T>,
    value_reference: fmiValueReference,
    value: &VariableValue,
) -> Result<fmiStatus, Box<dyn Error>> {
    match value {
        VariableValue::Float64(values) => call(fmu.setReal(&[value_reference], values)),
        VariableValue::Int32(values) => call(fmu.setInteger(&[value_reference], values)),
        VariableValue::Boolean(values) => {
            let values: Vec<fmi2Boolean> = values.iter().map(|v| if *v { fmi2True } else { fmi2False }).collect();
            call(fmu.setBoolean(&[value_reference], &values))
        }
        VariableValue::String(values) => {
            let string_refs: Vec<&str> = values.iter().map(|x| x.as_str()).collect();
            call(fmu.setString(&[value_reference], &string_refs))
        }
        _ => Err("Unsupported variable type {value:?}.".into()),
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

pub fn simulate_cs(settings: &SimulationSettings, simulation_result: &mut SimulationResult) -> Result<(), Box<dyn Error>> {
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
        match File::open(&path) {
            Ok(file) => match CSVInput::new(&file, &settings.model_description) {
                Ok(input) => Some(input),
                Err(e) => {
                    return Err(format!("Failed to load input from {path:?}. {e}").into());
                }
            },
            Err(e) => {
                return Err(format!("Failed to open input file {path:?}. {e}").into());
            }
        }
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

    let mut n_steps = 0;

    loop {
        if time > stop_time || relative_eq!(time, stop_time) {
            break;
        }

        let next_regular_point = start_time + (n_steps + 1) as f64 * output_interval;

        let mut next_communication_point = next_regular_point;

        if can_handle_variable_communication_step_size {
            if let Some(input) = &input {
                if let Some(next_input_event_time) = input.next_event_time(time) {
                    if next_regular_point > next_input_event_time
                        && !relative_eq!(next_regular_point, next_input_event_time)
                    {
                        next_communication_point = next_input_event_time;
                    }
                }
            }
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
        match File::open(&path) {
            Ok(file) => match CSVInput::new(&file, &settings.model_description) {
                Ok(input) => Some(input),
                Err(e) => {
                    return Err(format!("Failed to load input from {path:?}. {e}").into());
                }
            },
            Err(e) => {
                return Err(format!("Failed to open input file {path:?}. {e}").into());
            }
        }
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
        Box::new(|unknowns, knowns, seed, sensitivity| {
            fmu.getDirectionalDerivative(unknowns, knowns, seed, sensitivity);
            Ok(())
        }),
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

        if let Some(next_input_event_time) = next_input_event_time {
            if next_regular_point > next_input_event_time
                && !relative_eq!(next_regular_point, next_input_event_time)
            {
                next_communication_point = next_input_event_time;
            }
        }

        if let Some(next_event_time) = nextEventTime {
            if next_communication_point > next_event_time
                && !relative_eq!(next_communication_point, next_event_time)
            {
                next_communication_point = next_event_time;
            }
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

        if is_input_event {
            if let Some(input) = &input {
                input.set_continuous_inputs(time, false, &fmu)?;
            }
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

            if is_input_event {
                if let Some(input) = &input {
                    input.set_discrete_inputs(time, true, &fmu)?;
                    input.set_continuous_inputs(time, true, &fmu)?;
                }
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

    // let mut plot = Plot::new();

    // for (variable, trajectory) in recorder.variables.iter().zip(recorder.trajectories) {

    //     let time = recorder.time.clone();

    //     match trajectory {
    //         Trajectory::Real(values) => {
    //             plot.add_trace(Scatter::new(time, values).name(variable.name.clone()));
    //         }
    //         Trajectory::Integer(values) => {
    //             plot.add_trace(Scatter::new(time, values).name(variable.name.clone()));
    //         }
    //         Trajectory::Boolean(values) => {
    //             plot.add_trace(Scatter::new(time, values).name(variable.name.clone()));
    //         }
    //         Trajectory::String(_) => {
    //             // Plotting string variables is not supported
    //         }
    //     }
    // }

    // plot.write_html("plot.html");

    Ok(())
}
