use crate::{
    fmi2::{FMU2, InterfaceType},
    model_description::{ModelVariable, VariableType},
};
use std::io::Write;

macro_rules! write_values {
    ($values:expr, $stream:expr) => {{
        for (i, value) in $values.iter().enumerate() {
            if i > 0 {
                write!($stream, " ")?;
            }
            write!($stream, "{value}")?;
        }
    }};
}
pub struct Recorder<'a, T: Write, I: InterfaceType> {
    pub variables: &'a Vec<&'a ModelVariable>,
    pub stream: T,
    pub fmu: &'a FMU2<I>,
}

impl<'a, T: Write, I: InterfaceType> Recorder<'a, T, I> {
    pub fn new(variables: &'a Vec<&'a ModelVariable>, stream: T, fmu: &'a FMU2<I>) -> Recorder<'a, T, I> {
        let mut recorder = Recorder {
            variables,
            stream,
            fmu,
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

    pub fn sample(&mut self, time: f64) -> std::io::Result<()> {
        write!(self.stream, "{time}")?;

        for variable in self.variables {
            write!(self.stream, ",")?;
            let value_references = [variable.valueReference];
            match variable.variableType {
                VariableType::Float64 => {
                    let mut values = vec![0.0];
                    self.fmu.getReal(&value_references, &mut values);
                    write_values!(values, self.stream);
                }
                VariableType::Int32 | VariableType::Enumeration => {
                    let mut values = vec![0];
                    self.fmu.getInteger(&value_references, &mut values);
                    write_values!(values, self.stream);
                }
                VariableType::Boolean => {
                    let mut values = vec![0];
                    self.fmu.getBoolean(&value_references, &mut values);
                    write_values!(values, self.stream);
                }
                VariableType::String => {
                    let mut values = vec![String::new()];
                    self.fmu.getString(&value_references, &mut values);
                    write_values!(values, self.stream);
                }
                _ => panic!("Unexpected variable type: {:?}", variable.variableType),
            }
        }

        writeln!(self.stream)?;

        Ok(())
    }
}
