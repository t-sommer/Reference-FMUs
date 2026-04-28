use plotly::{Configuration, Layout, Plot, Scatter, common::Line, layout::{Axis, GridPattern, LayoutGrid, Margin}};
use crate::sim::fmi3::{SimulationResult, Trajectory};


pub fn plot_result(sim_results: &SimulationResult<'_>) -> Plot {
    let mut plot = Plot::new();

    let plot_height = 250 * sim_results.variables.len().max(1);

    let mut layout = Layout::new()
        // .title("Simulation Result".to_owned())
        .x_axis(Axis::new().title("time"))
        .grid(
            LayoutGrid::new()
                .rows(sim_results.variables.len())
                .columns(1)
                .pattern(GridPattern::Coupled), // Link X axes in the same column
        )
        .height(plot_height)
        .auto_size(true)
        .show_legend(false)
        .margin(Margin::new().top(30).bottom(40).left(65).right(30));

    for (i, (variable, trajectory)) in sim_results
        .variables
        .iter()
        .zip(&sim_results.trajectories)
        .enumerate()
    {
        let axis_title = variable.name.clone();
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

        let time = sim_results.time.clone();
        let name = variable.name.clone();
        let row = i + 1;

        match trajectory {
            Trajectory::Float64(values) => {
                if values.is_empty() {
                    continue;
                }

                for j in 0..values[0].len() {
                    let scalar_values: Vec<f64> = values.iter().map(|v| v[j]).collect();
                    let name = if values[0].len() > 1 {
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
            _ => todo!(),
        }
    }

    plot.set_layout(layout);

    plot.set_configuration(Configuration::new().responsive(true));

    plot
}
