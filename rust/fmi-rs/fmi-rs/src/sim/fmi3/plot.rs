use crate::{model_description::fmi3::VariableType, sim::fmi3::Trajectories};
use plotly::{
    Configuration, Layout, Plot, Scatter,
    common::Line,
    layout::{Axis, GridPattern, LayoutGrid, Margin},
};

pub fn plot_result(trajectories: &Trajectories<'_>) -> Plot {
    let mut plot = Plot::new();

    let plot_height = 250 * trajectories.variables.len().max(1);

    let mut layout = Layout::new()
        // .title("Simulation Result".to_owned())
        .x_axis(Axis::new().title("time"))
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
        let y_axis = Axis::new().title(axis_title.as_str());

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

        if matches!(
            variable.variableType,
            VariableType::String { .. } | VariableType::Binary { .. }
        ) {
            continue;
        }

        let size = trajectories.rows[0][i].len();

        for j in 0..size {
            let scalar_values: Vec<f64> = trajectories
                .rows
                .iter()
                .map(|row| row[i].as_f64()[j])
                .collect();
            let name = if size > 1 {
                format!("{}[{}]", name, j)
            } else {
                name.clone()
            };
            let mut trace = Scatter::new(time.clone(), scalar_values).name(name);
            // Use the shared x-axis ("x") for all subplots
            trace = trace
                .x_axis("x")
                .y_axis(format!("y{row}"))
                .line(Line::new().width(1.5).color("#229AEB"));
            plot.add_trace(trace);
        }
    }

    plot.set_layout(layout);

    plot.set_configuration(Configuration::new().responsive(true));

    plot
}
