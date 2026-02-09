use std::io::Write;
use crate::{fmi2::FMU2, model_description::{ModelVariable, VariableType}};

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
    pub fmu: &'a FMU2<'a>,
}

impl<'a, T: Write> Recorder<'a, T> {

    pub fn new(variables: &'a Vec<&'a ModelVariable>, stream: T, fmu: &'a FMU2<'a>) -> Recorder<'a, T> {
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
                },
                VariableType::Int32 | VariableType::Enumeration => {
                    let mut values = vec![0];
                    self.fmu.getInteger(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                VariableType::Boolean => {
                    let mut values = vec![0];
                    self.fmu.getBoolean(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                VariableType::String => {
                    let mut values = vec![String::new()];
                    self.fmu.getString(&value_references, &mut values);
                    write_values!(values, self.stream);
                },
                _ => panic!("Unexpected variable type: {:?}", variable.variableType),
            }
        }

        writeln!(self.stream)?;
        
        Ok(())
    }

}
