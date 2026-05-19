use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

use fmi::model_description::fmi2::VariableType;
use fmi::sim::euler::ForwardEulerFactory;
use fmi::sim::fmi2::Trajectories;
use plotly::{
    Configuration, Layout, Plot, Scatter, Trace, color::NamedColor, common::{Line, LineShape}, layout::{Axis, GridPattern, LayoutGrid, Margin}
};

use crate::{InterfaceType, SimulateArgs, SolverType, cvode};

pub fn simulate_fmu(
    args: &SimulateArgs,
    unzipdir: &tempfile::TempDir,
    xml_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let model_description = fmi::model_description::fmi2::ModelDescription::read(&xml_path)?;

    let output_variables: Vec<&fmi::model_description::fmi2::ScalarVariable> =
        if args.output_variable.is_empty() {
            model_description
                .modelVariables
                .iter()
                .filter(|v| v.causality == fmi::model_description::fmi2::Causality::Output)
                .collect()
        } else {
            let variable_map: HashMap<&str, &fmi::model_description::fmi2::ScalarVariable> =
                model_description
                    .modelVariables
                    .iter()
                    .map(|var| (var.name.as_str(), var))
                    .collect();

            let mut output_variables = vec![];

            for variable_name in &args.output_variable {
                if let Some(&variable) = variable_map.get(variable_name.as_str()) {
                    output_variables.push(variable);
                } else {
                    return Err(format!(
                        "The requested output variable {variable_name:?} does not exist."
                    )
                    .into());
                }
            }

            output_variables
        };

    let (start_time, stop_time, _tolerance) =
        if let Some(default_experiment) = &model_description.defaultExperiment {
            let start_time: f64 = if let Some(v) = &default_experiment.startTime {
                v.parse().unwrap()
            } else {
                0.0
            };
            let stop_time: f64 = if let Some(v) = &default_experiment.stopTime {
                v.parse().unwrap()
            } else {
                start_time + 1.0
            };
            let tolerance: Option<f64> = default_experiment
                .tolerance
                .as_ref()
                .map(|v| v.parse().unwrap());
            (start_time, stop_time, tolerance)
        } else {
            (0.0, 1.0, None)
        };

    let internal_step_size: Option<f64> = model_description
        .coSimulation
        .as_ref()
        .unwrap()
        .fixedInternalStepSize
        .as_ref()
        .map(|v| v.parse().unwrap());

    let start_time = args.start_time.unwrap_or(start_time);
    let stop_time = args.stop_time.unwrap_or(stop_time);
    let tolerance = args.tolerance;

    let output_interval = if let Some(v) = args.output_interval {
        v
    } else {
        if let Some(v) = internal_step_size {
            v
        } else {
            (stop_time - start_time) / 500.0
        }
    };

    let settings = fmi::sim::fmi2::SimulationSettings {
        unzipdir: unzipdir.path(),
        model_description: &model_description,
        start_time,
        stop_time,
        set_stop_time: args.set_stop_time,
        output_interval,
        tolerance,
        start_values: args.start_values.clone(),
        log_fmi_calls: args.log_fmi_calls,
        input_file: args.input_file.as_ref().map(|f| PathBuf::from(f)),
        early_return_allowed: args.early_return_allowed,
        event_mode_used: args.event_mode_used,
        logging_on: args.logging_on,
    };

    let interface_type = match &args.interface_type {
        Some(t) => t.clone(),
        None => {
            if let Some(_) = &model_description.coSimulation {
                InterfaceType::CoSimulation
            } else {
                InterfaceType::ModelExchange
            }
        }
    };

    let input = if let Some(path) = &args.input_file {
        let file = File::open(&path).expect("Failed to open input file");
        let trajectories = fmi::sim::fmi2::csv::read_csv(&file, &settings.model_description)
            .expect("Failed to read CSV");
        Some(fmi::sim::fmi2::input::StaticInput::new(trajectories))
    } else {
        None
    };

    let mut simulation_result =
        fmi::sim::fmi2::Trajectories::new(&model_description, output_variables.clone());

    let mut recorder = fmi::sim::fmi2::recorder::Recorder::new(&mut simulation_result);

    let fixes_step_size = args.fixed_step_size.unwrap_or(output_interval);

    let result = match interface_type {
        InterfaceType::ModelExchange => match args.solver {
            SolverType::Euler => fmi::sim::fmi2::simulate_me(
                &settings,
                &ForwardEulerFactory { fixes_step_size },
                input.as_ref(),
                &mut recorder,
            ),
            SolverType::Cvode => fmi::sim::fmi2::simulate_me(
                &settings,
                &cvode::CVodeSolverFactory,
                input.as_ref(),
                &mut recorder,
            ),
        },
        InterfaceType::CoSimulation => {
            fmi::sim::fmi2::simulate_cs(&settings, input.as_ref(), &mut recorder)
        }
    };

    if let Some(output_file) = args.output_file.as_ref() {
        fmi::sim::fmi2::csv::write_csv(&simulation_result, output_file)?;
    }

    if args.show_plot {
        let plot = crate::simulate::fmi2::plot_result(&simulation_result);

        // Generate a unique path in the temp directory starting with the model name
        let temp_path = tempfile::Builder::new()
            .prefix(&format!("{}_", model_description.modelName))
            .suffix(".html")
            .tempfile()?
            .into_temp_path();

        let path = temp_path.to_path_buf();
        // Prevent the file from being deleted immediately so the browser can read it
        let _ = temp_path.persist(&path);

        plot.show_html(path);
    }

    result
}

pub fn plot_result(trajectories: &Trajectories<'_>) -> Plot {
    let mut plot = Plot::new();

    let plot_height = 250 * trajectories.variables.len().max(1);

    let mut layout = Layout::new()
        .title(trajectories.model_description.modelName.clone())
        .x_axis(Axis::new().title("time [s]"))
        .grid(
            LayoutGrid::new()
                .rows(trajectories.variables.len())
                .columns(1)
                .pattern(GridPattern::Coupled), // Link X axes in the same column
        )
        .height(plot_height)
        .auto_size(true)
        .show_legend(false)
        .margin(Margin::new().top(30).bottom(40).left(65).right(30));

    for (i, variable) in trajectories.variables.iter().enumerate() {
        let mut axis_title = variable.name.clone();

        if let Some(unit) = trajectories.model_description.get_unit(variable) {
            axis_title.push_str(format!(" [{unit}]").as_str());
        }

        let mut y_axis = Axis::new()
            .title(axis_title.as_str())
            .zero_line_color(NamedColor::LightGrey);

        if matches!(variable.variableType, VariableType::Boolean { .. }) {
            y_axis = y_axis
                .tick_values(vec![0.0, 1.0])
                .tick_text(vec!["false", "true"]);
        }

        // Set y-axis titles for subplots (Plotly uses y1, y2, y3... internally)
        layout = match i {
            0 => layout.y_axis(y_axis),
            1 => layout.y_axis2(y_axis),
            2 => layout.y_axis3(y_axis),
            3 => layout.y_axis4(y_axis),
            4 => layout.y_axis5(y_axis),
            5 => layout.y_axis6(y_axis),
            6 => layout.y_axis7(y_axis),
            7 => layout.y_axis8(y_axis),
            _ => layout, // The plotly crate typed API typically supports up to y_axis8
        };

        let time = trajectories.time.clone();
        let name = variable.name.clone();
        let row = i + 1;

        if matches!(variable.variableType, VariableType::String { .. }) {
            continue;
        }

        let trace: Box<dyn Trace> = match variable.variableType {
            VariableType::Real { .. } => {
                let values: Vec<f64> = trajectories
                    .rows
                    .iter()
                    .map(|row| row[i].to_f64())
                    .collect();
                let mut trace = Scatter::new(time, values).name(name);
                // Use the shared x-axis ("x") for all subplots
                trace = trace
                    .x_axis("x")
                    .y_axis(format!("y{row}"))
                    .line(Line::new().width(1.5).color("#229AEB"));
                trace
            }
            VariableType::Integer { .. } => {
                let values: Vec<i32> = trajectories
                    .rows
                    .iter()
                    .map(|row| row[i].to_i32())
                    .collect();
                let mut trace = Scatter::new(time, values).name(name);
                // Use the shared x-axis ("x") for all subplots
                trace = trace
                    .x_axis("x")
                    .y_axis(format!("y{row}"))
                    .line(Line::new().width(1.5).color("#229AEB").shape(LineShape::Hv));
                trace
            }
            VariableType::Boolean { .. } => {
                let values: Vec<i32> = trajectories
                    .rows
                    .iter()
                    .map(|row| row[i].to_bool())
                    .collect();
                let mut trace = Scatter::new(time, values).name(name);
                // Use the shared x-axis ("x") for all subplots
                trace = trace
                    .x_axis("x")
                    .y_axis(format!("y{row}"))
                    .line(Line::new().width(1.5).color("#229AEB").shape(LineShape::Hv));
                trace
            }
            VariableType::Enumeration { .. } => {
                todo!()
            }
            VariableType::String { .. } => {
                continue
            }
        };


        // let values: Vec<f64> = trajectories
        //     .rows
        //     .iter()
        //     .map(|row| row[i].to_f64())
        //     .collect();

        // let mut trace = Scatter::new(time, values).name(name);

        // // Use the shared x-axis ("x") for all subplots
        // trace = trace
        //     .x_axis("x")
        //     .y_axis(format!("y{row}"))
        //     .line(Line::new().width(1.5).color("#229AEB"));
        
        plot.add_trace(trace);
    }

    plot.set_layout(layout);

    plot.set_configuration(Configuration::new().responsive(true));

    plot
}
