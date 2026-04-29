use std::path::Path;
use crate::sim::fmi3::SimulationResult;


pub fn write_csv<P: AsRef<Path>>(
    sim_result: &SimulationResult<'_>,
    output_file: P,
) -> std::io::Result<()> {
    let mut writer = csv::Writer::from_path(output_file)?;

    let mut header = vec!["time".to_string()];

    for variable in sim_result.variables.iter() {
        header.push(variable.name.clone());
    }

    writer.write_record(&header)?;

    for i in 0..sim_result.time.len() {
        
        let mut record = vec![sim_result.time[i].to_string()];

        for variable_value in (&sim_result.rows[i]).iter() {
            record.push(variable_value.to_literal());
        }

        writer.write_record(&record)?;
    }

    writer.flush()?;

    Ok(())
}
