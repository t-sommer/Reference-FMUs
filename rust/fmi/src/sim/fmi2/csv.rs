use std::path::Path;
use crate::sim::fmi2::{SimulationResult, Trajectory};

pub fn write_csv<P: AsRef<Path>>(
    sim_results: &SimulationResult<'_>,
    output_file: P,
) -> std::io::Result<()> {
    let mut writer = csv::Writer::from_path(output_file)?;

    let mut header = vec!["time".to_string()];

    for variable in sim_results.variables.iter() {
        header.push(variable.name.clone());
    }

    writer.write_record(&header)?;

    for i in 0..sim_results.time.len() {
        let mut record = vec![sim_results.time[i].to_string()];

        for trajectory in sim_results.trajectories.iter() {
            match trajectory {
                Trajectory::Real(values) => record.push(values[i].to_string()),
                Trajectory::Integer(values) => record.push(values[i].to_string()),
                Trajectory::Boolean(values) => record.push(values[i].to_string()),
                Trajectory::String(values) => record.push(values[i].clone()),
            }
        }

        writer.write_record(&record)?;
    }

    writer.flush()?;

    Ok(())
}
