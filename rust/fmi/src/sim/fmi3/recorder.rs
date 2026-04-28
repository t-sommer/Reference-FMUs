use crate::{
    fmi3::FMU3,
    model_description::{Dimension, VariableType}, sim::fmi3::{SimulationResult, Trajectory},
};

pub struct Recorder<'fmu, 'res, 'md> {
    pub fmu: &'fmu FMU3,
    pub simulation_result: &'res mut SimulationResult<'md>,
    pub sizes: Vec<usize>,
}

impl<'fmu, 'res, 'md> Recorder<'fmu, 'res, 'md> {
    pub fn new(
        fmu: &'fmu FMU3,
        simulation_result: &'res mut SimulationResult<'md>,
    ) -> Self {
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
            for (i, variable) in self.simulation_result.variables.iter().enumerate() {
                let size = self.sizes.get(i).unwrap();
                let value_references = [variable.valueReference];
                match variable.variableType {
                    VariableType::Float32 => {
                        let mut values = vec![0f32; *size];
                        self.fmu.getFloat32(&value_references, &mut values);
                        if let Trajectory::Float32(trajectory) = &mut self.simulation_result.trajectories[i] {
                            trajectory.push(values);
                        } else {
                            panic!("Expected Float32 trajectory for variable {}", variable.name);
                        }
                    }
                    VariableType::Float64 => {
                        let mut values = vec![0f64; *size];
                        self.fmu.getFloat64(&value_references, &mut values);
                        if let Trajectory::Float64(trajectory) = &mut self.simulation_result.trajectories[i] {
                            trajectory.push(values);
                        } else {
                            panic!("Expected Float64 trajectory for variable {}", variable.name);
                        }
                    }
                    VariableType::Int8 => {
                        let mut values = vec![0i8; *size];
                        self.fmu.getInt8(&value_references, &mut values);
                        if let Trajectory::Int8(trajectory) = &mut self.simulation_result.trajectories[i] {
                            trajectory.push(values);
                        } else {
                            panic!("Expected Int8 trajectory for variable {}", variable.name);
                        }
                    }
                    VariableType::UInt8 => {
                        let mut values = vec![0u8; *size];
                        self.fmu.getUInt8(&value_references, &mut values);
                        if let Trajectory::UInt8(trajectory) = &mut self.simulation_result.trajectories[i] {
                            trajectory.push(values);
                        } else {
                            panic!("Expected UInt8 trajectory for variable {}", variable.name);
                        }
                    }
                    VariableType::Int16 => {
                        let mut values = vec![0i16; *size];
                        self.fmu.getInt16(&value_references, &mut values);
                        if let Trajectory::Int16(trajectory) = &mut self.simulation_result.trajectories[i] {
                            trajectory.push(values);
                        } else {
                            panic!("Expected Int16 trajectory for variable {}", variable.name);
                        }
                    }
                    VariableType::UInt16 => {
                        let mut values = vec![0u16; *size];
                        self.fmu.getUInt16(&value_references, &mut values);
                        if let Trajectory::UInt16(trajectory) = &mut self.simulation_result.trajectories[i] {
                            trajectory.push(values);
                        } else {
                            panic!("Expected UInt16 trajectory for variable {}", variable.name);
                        }
                    }
                    VariableType::Int32 => {
                        let mut values = vec![0i32; *size];
                        self.fmu.getInt32(&value_references, &mut values);
                        if let Trajectory::Int32(trajectory) = &mut self.simulation_result.trajectories[i] {
                            trajectory.push(values);
                        } else {
                            panic!("Expected Int32 trajectory for variable {}", variable.name);
                        }
                    }
                    VariableType::UInt32 => {
                        let mut values = vec![0u32; *size];
                        self.fmu.getUInt32(&value_references, &mut values);
                        if let Trajectory::UInt32(trajectory) = &mut self.simulation_result.trajectories[i] {
                            trajectory.push(values);
                        } else {
                            panic!("Expected UInt32 trajectory for variable {}", variable.name);
                        }
                    }
                    VariableType::Int64 => {
                        let mut values = vec![0i64; *size];
                        self.fmu.getInt64(&value_references, &mut values);
                        if let Trajectory::Int64(trajectory) = &mut self.simulation_result.trajectories[i] {
                            trajectory.push(values);
                        } else {
                            panic!("Expected Int64 trajectory for variable {}", variable.name);
                        }
                    }
                    VariableType::UInt64 => {
                        let mut values = vec![0u64; *size];
                        self.fmu.getUInt64(&value_references, &mut values);
                        if let Trajectory::UInt64(trajectory) = &mut self.simulation_result.trajectories[i] {
                            trajectory.push(values);
                        } else {
                            panic!("Expected UInt64 trajectory for variable {}", variable.name);
                        }
                    }
                    VariableType::Boolean => {
                        let mut values = vec![false; *size];
                        self.fmu.getBoolean(&value_references, &mut values);
                        if let Trajectory::Boolean(trajectory) = &mut self.simulation_result.trajectories[i] {
                            trajectory.push(values);
                        } else {
                            panic!("Expected Boolean trajectory for variable {}", variable.name);
                        }
                    }
                    VariableType::String => {
                        let mut values = vec![String::new(); *size];
                        self.fmu.getString(&value_references, &mut values);
                        if let Trajectory::String(trajectory) = &mut self.simulation_result.trajectories[i] {
                            trajectory.push(values);
                        } else {
                            panic!("Expected String trajectory for variable {}", variable.name);
                        }
                    }
                    VariableType::Binary => {
                        let mut sizes = vec![0usize; *size];
                        let mut values = vec![std::ptr::null(); *size];
                        self.fmu.getBinary(&value_references, &mut sizes, &mut values);
                        let binary_values: Vec<Vec<u8>> = values
                            .iter()
                            .zip(sizes.iter())
                            .map(|(ptr, size)| {
                                if ptr.is_null() || *size == 0 {
                                    vec![]
                                } else {
                                    unsafe { std::slice::from_raw_parts(*ptr, *size).to_vec() }
                                }
                            })
                            .collect();
                        if let Trajectory::Binary(trajectory) = &mut self.simulation_result.trajectories[i] {
                            trajectory.push(binary_values);
                        } else {
                            panic!("Expected Binary trajectory for variable {}", variable.name);
                        }
                    }
                    _ => continue,
                }
            }
        Ok(())
    }
}
