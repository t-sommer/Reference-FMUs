use crate::{
    model_description::fmi3::{ModelDescription, ModelVariable},
    sim::fmi3::{SimulationResult, parse_variable_value},
};
use std::{collections::HashMap, io::Read, path::Path};

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

pub fn read_csv<'a, R: Read>(
    reader: R,
    model_description: &'a ModelDescription,
) -> Result<SimulationResult<'a>, Box<dyn std::error::Error>> {
    // Create a map for quick lookup of variables by name
    let variable_map: HashMap<&str, &ModelVariable> = model_description
        .modelVariables
        .iter()
        .map(|var| (var.name.as_str(), var))
        .collect();

    let mut reader = csv::Reader::from_reader(reader);

    let headers = match reader.headers() {
        Ok(record) => record,
        Err(e) => {
            return Err(format!("Failed to read headers. {e}").into());
        }
    };

    let mut variables: Vec<&ModelVariable> = vec![];

    for name in headers.iter().skip(1) {
        if let Some(variable) = variable_map.get(name) {
            variables.push(variable);
        } else {
            return Err(format!("Variable {name:?} does not exist in the FMU.").into());
        }
    }

    let mut time = vec![];
    let mut rows = vec![];

    for (i, result) in reader.records().enumerate() {
        match result {
            Ok(record) => {
                let mut row = vec![];

                let mut it = record.iter();

                time.push(it.next().unwrap().parse().unwrap());

                for (j, literal) in it.enumerate() {
                    let variable: &ModelVariable = variables[j];

                    match parse_variable_value(&variable.variableType, literal) {
                        Ok(v) => row.push(v),
                        Err(e) => {
                            return Err(format!(
                                "Failed to parse {literal:?} (row {i}, column {}). {e}",
                                j + 1
                            )
                            .into());
                        }
                    }
                }

                rows.push(row);
            }
            Err(e) => {
                return Err(format!("Error reading input. {e}").into());
            }
        }
    }

    Ok(SimulationResult {
        time,
        variables,
        rows,
    })
}
