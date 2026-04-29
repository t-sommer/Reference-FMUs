use crate::{
    fmi2::FMU2,
    model_description::VariableType,
    sim::fmi2::{SimulationResult, VariableValue},
};

pub struct Recorder<'fmu, 'res, 'md, I> {
    pub fmu: &'fmu FMU2<I>,
    pub simulation_result: &'res mut SimulationResult<'md>,
}

impl<'fmu, 'res, 'md, I> Recorder<'fmu, 'res, 'md, I> {
    pub fn new(fmu: &'fmu FMU2<I>, simulation_result: &'res mut SimulationResult<'md>) -> Self {
        Recorder {
            fmu,
            simulation_result,
        }
    }

    pub fn sample(&mut self, time: f64) -> std::io::Result<()> {
        self.simulation_result.time.push(time);

        let mut row = vec![];

        for variable in self.simulation_result.variables.iter() {
            let value_references = [variable.valueReference];

            // TODO: handle status

            let variable_value = match variable.variableType {
                VariableType::Float64 => {
                    let mut values = [0.0];
                    self.fmu.getReal(&value_references, &mut values);
                    VariableValue::Real(values[0])
                }
                VariableType::Int32 | VariableType::Enumeration => {
                    let mut values = [0];
                    self.fmu.getInteger(&value_references, &mut values);
                    VariableValue::Integer(values[0])
                }
                VariableType::Boolean => {
                    let mut values = [0];
                    self.fmu.getBoolean(&value_references, &mut values);
                    VariableValue::Boolean(values[0])
                }
                VariableType::String => {
                    let mut values = [String::new()];
                    self.fmu.getString(&value_references, &mut values);
                    VariableValue::String(values[0].clone())
                }
                _ => panic!("Unexpected variable type: {:?}", variable.variableType),
            };

            row.push(variable_value);
        }

        self.simulation_result.rows.push(row);

        Ok(())
    }
}
