use std::{ptr::null_mut, vec};

use crate::{
    fmi3::{FMU3, types::{fmi3Binary, fmi3Byte}},
    model_description::{Dimension, VariableType},
    sim::fmi3::{SimulationResult, VariableValue},
};

pub struct Recorder<'fmu, 'res, 'md> {
    pub fmu: &'fmu FMU3,
    pub simulation_result: &'res mut SimulationResult<'md>,
    pub sizes: Vec<usize>,
}

impl<'fmu, 'res, 'md> Recorder<'fmu, 'res, 'md> {
    pub fn new(fmu: &'fmu FMU3, simulation_result: &'res mut SimulationResult<'md>) -> Self {
        let mut recorder = Recorder {
            fmu,
            simulation_result,
            sizes: vec![],
        };
        recorder.update_sizes();
        recorder
    }

    pub fn update_sizes(&mut self) {
        self.sizes.clear();
        for variable in self.simulation_result.variables.iter() {
            let mut size = 1usize;
            for dimension in variable.dimensions.iter() {
                size *= match dimension {
                    Dimension::Fixed(value) => *value,
                    Dimension::Variable(value_reference) => {
                        let mut values = [0u64];
                        // TODO: handle status
                        self.fmu.getUInt64(&[*value_reference], &mut values);
                        values[0] as usize
                    }
                };
            }
            self.sizes.push(size);
        }
    }

    pub fn sample(&mut self, time: f64) -> Result<(), Box<dyn std::error::Error>> {
        
        if self.sizes.is_empty() {
            self.update_sizes();
        }

        // TODO: handle FMI status

        self.simulation_result.time.push(time);

        let mut row = vec![];

        for (i, variable) in self.simulation_result.variables.iter().enumerate() {
            
            let size = self.sizes.get(i).unwrap();
            
            let value_references = [variable.valueReference];
            
            let variable_value = match variable.variableType {
                VariableType::Float32 => {
                    let mut values = vec![0f32; *size];
                    self.fmu.getFloat32(&value_references, &mut values);
                    VariableValue::Float32(values)
                }
                VariableType::Float64 => {
                    let mut values = vec![0f64; *size];
                    self.fmu.getFloat64(&value_references, &mut values);
                    VariableValue::Float64(values)
                }
                VariableType::Int8 => {
                    let mut values = vec![0i8; *size];
                    self.fmu.getInt8(&value_references, &mut values);
                    VariableValue::Int8(values)
                }
                VariableType::UInt8 => {
                    let mut values = vec![0u8; *size];
                    self.fmu.getUInt8(&value_references, &mut values);
                    VariableValue::UInt8(values)
                }
                VariableType::Int16 => {
                    let mut values = vec![0i16; *size];
                    self.fmu.getInt16(&value_references, &mut values);
                    VariableValue::Int16(values)
                }
                VariableType::UInt16 => {
                    let mut values = vec![0u16; *size];
                    self.fmu.getUInt16(&value_references, &mut values);
                    VariableValue::UInt16(values)
                }
                VariableType::Int32 => {
                    let mut values = vec![0i32; *size];
                    self.fmu.getInt32(&value_references, &mut values);
                    VariableValue::Int32(values)
                }
                VariableType::UInt32 => {
                    let mut values = vec![0u32; *size];
                    self.fmu.getUInt32(&value_references, &mut values);
                    VariableValue::UInt32(values)
                }
                VariableType::Int64 | VariableType::Enumeration => {
                    let mut values = vec![0i64; *size];
                    self.fmu.getInt64(&value_references, &mut values);
                    VariableValue::Int64(values)
                }
                VariableType::UInt64 => {
                    let mut values = vec![0u64; *size];
                    self.fmu.getUInt64(&value_references, &mut values);
                    VariableValue::UInt64(values)
                }
                VariableType::Boolean => {
                    let mut values = vec![false; *size];
                    self.fmu.getBoolean(&value_references, &mut values);
                    VariableValue::Boolean(values)
                }
                VariableType::String => {
                    let mut values = vec![String::new(); *size];
                    self.fmu.getString(&value_references, &mut values);
                    VariableValue::String(values)
                }
                VariableType::Binary => {
                    let mut values = vec![vec![]; *size];
                    self.fmu.getBinary(&value_references, &mut values);
                    VariableValue::Binary(values)
                }
                _ => continue
            };
            
            row.push(variable_value);
        }

        self.simulation_result.rows.push(row);
        
        Ok(())
    }
}
