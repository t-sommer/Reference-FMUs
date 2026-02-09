use std::io::Write;
use crate::{fmi3::FMU3, model_description::{Dimension, ModelVariable, VariableType}};

macro_rules! write_values {
    ($values:expr, $stream:expr) => {{
        for (i, value) in $values.iter().enumerate() {
            if i > 0 {
                write!($stream, " ")?;
            }
            write!($stream, "{value}")?;
        }}  
    };
}

pub struct Recorder<'a, T: Write> {
    pub variables: &'a Vec<&'a ModelVariable>,
    pub stream: T,
    pub fmu: &'a FMU3<'a>,
    sizes: Vec<usize>,
}

impl<'a, T: Write> Recorder<'a, T> {

    pub fn new(variables: &'a Vec<&'a ModelVariable>, stream: T, fmu: &'a FMU3<'a>) -> Recorder<'a, T> {
        let mut recorder = Recorder { 
            variables,
            stream, 
            fmu, 
            sizes: vec![],
        };
        recorder.write_header().unwrap();
        recorder
    }

    pub fn write_header(&mut self) -> std::io::Result<()> {
        write!(self.stream, "\"time\"")?;
        for variable in self.variables.iter() {
            write!(self.stream, ",\"{}\"", variable.name)?;
        }
        writeln!(self.stream)?;
        Ok(())
    }

    pub fn update_sizes(&mut self) {
        self.sizes.clear();
        for variable in self.variables.iter() {
            let mut size = 1usize;
            for dimension in variable.dimensions.iter() {
                size *= match dimension {
                    Dimension::Fixed(value) => *value,
                    Dimension::Variable(value_reference) => {
                        let mut values = [0u64];
                        // TODO: handle status
                        self.fmu.getUInt64(&[*value_reference], &mut values);
                        values[0] as usize
                    },
                };
            }
            self.sizes.push(size);
        }
    }

    pub fn sample(&mut self, time: f64) -> std::io::Result<()> {

        if self.sizes.is_empty() {
            self.update_sizes();
        }

        write!(self.stream, "{time}")?;

        for (i, variable) in self.variables.iter().enumerate() {
            let size = self.sizes.get(i).unwrap();
            write!(self.stream, ",")?;
            let value_references = [variable.valueReference];
            match variable.variableType {
                VariableType::Float32 => {
                    let mut values = vec![0f32; *size];
                    self.fmu.getFloat32(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                VariableType::Float64 => {
                    let mut values = vec![0f64; *size];
                    self.fmu.getFloat64(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                VariableType::Int8 => {
                    let mut values = vec![0i8; *size];
                    self.fmu.getInt8(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                VariableType::UInt8 => {
                    let mut values = vec![0u8; *size];
                    self.fmu.getUInt8(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                VariableType::Int16 => {
                    let mut values = vec![0i16; *size];
                    self.fmu.getInt16(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                VariableType::UInt16 => {
                    let mut values = vec![0u16; *size];
                    self.fmu.getUInt16(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                VariableType::Int32 => {
                    let mut values = vec![0i32; *size];
                    self.fmu.getInt32(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                VariableType::UInt32 => {
                    let mut values = vec![0u32; *size];
                    self.fmu.getUInt32(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                VariableType::Int64 => {
                    let mut values = vec![0i64; *size];
                    self.fmu.getInt64(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                VariableType::UInt64 => {
                    let mut values = vec![0u64; *size];
                    self.fmu.getUInt64(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                VariableType::Boolean => {
                    let mut values = vec![false; *size];
                    self.fmu.getBoolean(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                VariableType::String => {
                    let mut values = vec![String::new(); *size];
                    self.fmu.getString(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                VariableType::Binary => {
                    let mut sizes = vec![0usize; *size];
                    let mut values = vec![std::ptr::null(); *size];
                    self.fmu.getBinary(&value_references, &mut sizes, &mut values);
                    let string_values: Vec<String> = values.iter().zip(sizes.iter()).map(|(ptr, size)| {
                        if ptr.is_null() || *size == 0 {
                            String::new()
                        } else {
                            unsafe {
                                let slice = std::slice::from_raw_parts(*ptr, *size);
                                slice.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("")
                            }
                        }
                    }).collect();
                    write_values!(string_values, self.stream);
                },
                VariableType::Enumeration => {
                    let mut values = vec![0i64; *size];
                    self.fmu.getInt64(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                _ => continue,
            }
        }
        writeln!(self.stream)?;
        Ok(())
    }

}
