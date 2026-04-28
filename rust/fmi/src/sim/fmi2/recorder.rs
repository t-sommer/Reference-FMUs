use crate::{
    fmi2::FMU2,
    model_description::{ModelVariable, VariableType}, sim::fmi2::{SimulationResult, Trajectory},
};

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

pub struct Recorder<'fmu, 'res, 'md, I> {
    pub fmu: &'fmu FMU2<I>,
    pub simulation_result: &'res mut SimulationResult<'md>,
}

impl<'fmu, 'res, 'md, I> Recorder<'fmu, 'res, 'md, I> {
    pub fn new(
        fmu: &'fmu FMU2<I>,
        simulation_result: &'res mut SimulationResult<'md>,
    ) -> Self {

        // let mut trajectories = vec![];

        // for variable in variables.iter() {

        //     let trajectory = match variable.variableType {
        //         VariableType::Float64 => Trajectory::Real(vec![]),
        //         VariableType::Int32 | VariableType::Enumeration => Trajectory::Integer(vec![]),
        //         VariableType::Boolean => Trajectory::Boolean(vec![]),
        //         VariableType::String => Trajectory::String(vec![]),
        //         _ => panic!("Unexpected variable type: {:?}", variable.variableType),
        //     };

        //     trajectories.push(trajectory);
        // }

        Recorder {
            fmu,
            simulation_result,
        }
    }

    // pub fn write_header(&mut self) -> std::io::Result<()> {
    //     write!(self.stream, "\"time\"")?;
    //     for variable in self.simulation_result.variables.iter() {
    //         write!(self.stream, ",\"{}\"", variable.name)?;
    //     }
    //     writeln!(self.stream)?;
    //     Ok(())
    // }

    pub fn sample(&mut self, time: f64) -> std::io::Result<()> {

        // write!(self.stream, "{time}")?;

        self.simulation_result.time.push(time);

        for (variable, trajectory) in self.simulation_result.variables.iter().zip(self.simulation_result
            .trajectories.iter_mut()) {
            // write!(self.stream, ",")?;
            let value_references = [variable.valueReference];
            match variable.variableType {
                VariableType::Float64 => {
                    let mut values = vec![0.0];
                    self.fmu.getReal(&value_references, &mut values);
                    // write_values!(values, self.stream);
                    if let Trajectory::Real(vec) = trajectory {
                        vec.push(values[0]);
                    } else {
                        panic!("Trajectory type mismatch for variable {}", variable.name);
                    }
                }
                VariableType::Int32 | VariableType::Enumeration => {
                    let mut values = vec![0];
                    self.fmu.getInteger(&value_references, &mut values);
                    // write_values!(values, self.stream);
                    if let Trajectory::Integer(vec) = trajectory {
                        vec.push(values[0]);
                    } else {
                        panic!("Trajectory type mismatch for variable {}", variable.name);
                    }
                }
                VariableType::Boolean => {
                    let mut values = vec![0];
                    self.fmu.getBoolean(&value_references, &mut values);
                    // write_values!(values, self.stream);
                    if let Trajectory::Boolean(vec) = trajectory {
                        vec.push(values[0]);
                    } else {
                        panic!("Trajectory type mismatch for variable {}", variable.name);
                    }
                }
                VariableType::String => {
                    let mut values = vec![String::new()];
                    self.fmu.getString(&value_references, &mut values);
                    // write_values!(values, self.stream);
                    if let Trajectory::String(vec) = trajectory {
                        vec.push(values[0].clone());
                    } else {
                        panic!("Trajectory type mismatch for variable {}", variable.name);
                    }
                }
                _ => panic!("Unexpected variable type: {:?}", variable.variableType),
            }
        }

        // writeln!(self.stream)?;

        Ok(())
    }
}
