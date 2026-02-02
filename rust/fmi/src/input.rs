use std::{collections::HashMap, error::Error, fs::File};

use crate::{fmi3::FMU3, model_description::{ModelDescription, ModelVariable, Variability, VariableType}, types::fmiStatus};


fn call(status: fmiStatus) -> Result<fmiStatus, Box<dyn Error>> {
    if matches!(status, fmiStatus::fmiOK | fmiStatus::fmiWarning) {
        Ok(status)
    } else {
        Err(format!("FMI call failed with status: {:?}", status).into())
    }
}

#[derive(Debug, PartialEq)]
enum VariableValue {
    Float32(Vec<f32>),
    Float64(Vec<f64>),
    UInt64(Vec<u64>),
}

fn parse_variable_value(variable_type: &VariableType, literal: &str) -> Result<VariableValue, Box<dyn Error>> {
    match variable_type {
        VariableType::Float64 => {
            let values: Result<Vec<f64>, _> = literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::Float64(values?))
        },
        VariableType::UInt64 => {
            let values: Result<Vec<u64>, _> = literal.split_whitespace().map(|v| v.parse()).collect();
            Ok(VariableValue::UInt64(values?))
        },
        _ => todo!()
    }
}

#[derive(Debug)]
pub struct CSVInput<'a> {
    variables: Vec<&'a ModelVariable>,
    time: Vec<f64>,
    rows: Vec<Vec<VariableValue>>,
}

fn approx_eq(a: f64, b: f64) -> bool {

    let rel_tol = 1e-12;
    let abs_tol = 1e-15;

    // exact equality handles infinities and signed zero quickly
    if a == b { return true; }

    // NaNs are never approximately equal
    if a.is_nan() || b.is_nan() { return false; }

    let diff = (a - b).abs();

    if diff <= abs_tol {
        return true;
    }
    
    diff <= rel_tol * a.abs().max(b.abs())
}

impl<'a> CSVInput<'a> {

    pub fn new(file: &File, model_description: &'a ModelDescription) -> Result<CSVInput<'a>, Box<dyn Error>> {

        // Create a map for quick lookup of variables by name
        let variable_map: HashMap<&str, &ModelVariable> = model_description.modelVariables
            .iter()
            .map(|var| (var.name.as_str(), var))
            .collect();
        
        let mut reader = csv::Reader::from_reader(file);
        
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
                                return Err(format!("Failed to parse {literal:?} (row {i}, column {}). {e}", j + 1).into());
                            },
                        }
                    }
                    
                    rows.push(row);
                },
                Err(e) => {
                    return Err(format!("Error reading input. {e}").into());
                }
            }
                
        }

        Ok(CSVInput { variables, time, rows })
    }

    pub fn next_event_time(&self, time: f64) -> Option<f64> {

        for i in 0..self.time.len() - 1 {
            
            let t0 = self.time[i];
            let t1 = self.time[i + 1];

            if time >= t1 { // TODO: use is_close()
                continue;
            }

            if t0 == t1 {
                return Some(t0)  // discrete change of a continuous variable
            }

            let row0 = &self.rows[i];
            let row1 = &self.rows[i + 1];

            for (j, variable) in self.variables.iter().enumerate() {
                
                if variable.variability == Variability::Continuous {
                    continue;  // skip continuous variables
                }

                let value0 = &row0[j];
                let value1 = &row1[j];

                if value0 != value1 {
                    return Some(t1);
                }
            }

        }

        None
    }

    pub fn set_discrete_inputs(&self, time: f64, after_event: bool, fmu: &FMU3) -> Result<(), Box<dyn Error>> {

        let mut index = 0;

        for (i, t) in self.time.iter().enumerate() {
            if *t > time {
                break;
            }
            index = i;
        }

        let row = &self.rows[index];

        for (variable, value) in self.variables.iter().zip(row.iter()) {

            if variable.variability != Variability::Continuous {
                match value {
                    VariableValue::Float64(values) => {
                        fmu.setFloat64(&[variable.valueReference], values);
                    },
                    VariableValue::UInt64(values) => {
                        fmu.setUInt64(&[variable.valueReference], values);
                    },
                    _ => todo!(),
                }
            }
        }

        Ok(())
    }

    pub fn set_continuous_inputs(&self, time: f64, after_event: bool, fmu: &FMU3) -> Result<(), Box<dyn Error>> {

        let mut row_index = 0;

        // find the index
        while row_index < self.time.len() - 1 {

            let next_time = self.time[row_index + 1];
            
            if !after_event && (approx_eq(next_time, time) || next_time > time) {
                break
            }
            
            if after_event && (next_time > time && !approx_eq(next_time, time)) {
                break
            }
            
            row_index += 1;
        }

        let time_s = self.time[0];
        let time_e = self.time[self.time.len() - 1];

        let interpolate = time > time_s && !approx_eq(time, time_s) && time < time_e && !approx_eq(time, time_e);

        if interpolate {

            let row0 = &self.rows[row_index];
            let row1 = &self.rows[row_index + 1];

            for (i, variable) in self.variables.iter().enumerate() {

                if variable.variability != Variability::Continuous {
                    continue
                }

                let t0 = self.time[row_index];
                let t1 = self.time[row_index + 1];
                let t = (time - t0) / (t1 - t0);

                let value0 = &row0[i];
                let value1 = &row1[i];

                println!("TODO: interpolate {variable:?} for time {time} {t0}-{t1}  {value0:?}-{value1:?}");

                match value0 {
                    VariableValue::Float32(values0) => {

                        if let VariableValue::Float32(values1) = value1 {

                            let mut interpolated_values = vec![0.0; values0.len()];

                            for j in 0..interpolated_values.len() {
                                let x0 = values0[j];
                                let x1 = values1[j];
                                interpolated_values[j] = x0 + t as f32 * (x1 - x0);
                                println!("{x0} {x1}")
                            }
        
                            call(fmu.setFloat32(&[variable.valueReference], &interpolated_values))?;
                        }
                    },
                    VariableValue::Float64(values0) => {

                        if let VariableValue::Float64(values1) = value1 {

                            let mut interpolated_values = vec![0.0; values0.len()];

                            for j in 0..interpolated_values.len() {
                                let x0 = values0[j];
                                let x1 = values1[j];
                                interpolated_values[j] = x0 + t * (x1 - x0);
                                println!("{x0} {x1}")
                            }
    
                            call(fmu.setFloat64(&[variable.valueReference], &interpolated_values))?;
                        }
                    },
                    _ => panic!(),
                }
    
            }

        } else {

            let row = &self.rows[row_index];
    
            for (variable, value) in self.variables.iter().zip(row.iter()) {
    
                if variable.variability == Variability::Continuous {
                    match value {
                        VariableValue::Float32(values) => {
                            fmu.setFloat32(&[variable.valueReference], values);
                        },
                        VariableValue::Float64(values) => {
                            fmu.setFloat64(&[variable.valueReference], values);
                        },
                        _ => panic!(),
                    }
                }
            }
        }


        Ok(())
    }
}
