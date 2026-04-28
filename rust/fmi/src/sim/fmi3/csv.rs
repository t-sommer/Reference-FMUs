use std::path::Path;
use crate::sim::fmi3::{SimulationResult, Trajectory};


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
                Trajectory::Float32(values) => {
                    record.push(
                        (&values[i])
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(" "),
                    );
                }
                Trajectory::Float64(values) => {
                    record.push(
                        (&values[i])
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(" "),
                    );
                }
                _ => todo!(),
            }
        }

        writer.write_record(&record)?;
    }

    writer.flush()?;

    Ok(())
}
