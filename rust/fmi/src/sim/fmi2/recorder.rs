use crate::{
    fmi2::FMU2,
    model_description::VariableType, sim::fmi2::{SimulationResult, Trajectory},
};

pub struct Recorder<'fmu, 'res, 'md, I> {
    pub fmu: &'fmu FMU2<I>,
    pub simulation_result: &'res mut SimulationResult<'md>,
}

impl<'fmu, 'res, 'md, I> Recorder<'fmu, 'res, 'md, I> {
    pub fn new(
        fmu: &'fmu FMU2<I>,
        simulation_result: &'res mut SimulationResult<'md>,
    ) -> Self {
        Recorder {
            fmu,
            simulation_result,
        }
    }

    pub fn sample(&mut self, time: f64) -> std::io::Result<()> {

        self.simulation_result.time.push(time);

        for (variable, trajectory) in self.simulation_result.variables.iter().zip(self.simulation_result
            .trajectories.iter_mut()) {
            let value_references = [variable.valueReference];
            match variable.variableType {
                VariableType::Float64 => {
                    let mut values = vec![0.0];
                    self.fmu.getReal(&value_references, &mut values);
                    if let Trajectory::Real(vec) = trajectory {
                        vec.push(values[0]);
                    } else {
                        panic!("Trajectory type mismatch for variable {}", variable.name);
                    }
                }
                VariableType::Int32 | VariableType::Enumeration => {
                    let mut values = vec![0];
                    self.fmu.getInteger(&value_references, &mut values);
                    if let Trajectory::Integer(vec) = trajectory {
                        vec.push(values[0]);
                    } else {
                        panic!("Trajectory type mismatch for variable {}", variable.name);
                    }
                }
                VariableType::Boolean => {
                    let mut values = vec![0];
                    self.fmu.getBoolean(&value_references, &mut values);
                    if let Trajectory::Boolean(vec) = trajectory {
                        vec.push(values[0]);
                    } else {
                        panic!("Trajectory type mismatch for variable {}", variable.name);
                    }
                }
                VariableType::String => {
                    let mut values = vec![String::new()];
                    self.fmu.getString(&value_references, &mut values);
                    if let Trajectory::String(vec) = trajectory {
                        vec.push(values[0].clone());
                    } else {
                        panic!("Trajectory type mismatch for variable {}", variable.name);
                    }
                }
                _ => panic!("Unexpected variable type: {:?}", variable.variableType),
            }
        }

        Ok(())
    }
}
