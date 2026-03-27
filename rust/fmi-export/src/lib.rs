#![allow(non_snake_case)]

use fmi::types::fmiStatus;
use serde::{Deserialize, Serialize};

// Re-export the derive macro
pub use fmi_export_derive::ValueReference;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelMode {
    Instantiated,
    InitializationMode,
    EventMode,
    ContinuousTimeMode,
    StepMode,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Solver {
    x: Vec<f64>,
    der_x: Vec<f64>,
    z: Vec<f64>,
    pre_z: Vec<f64>,
}

impl Solver {
    
    pub fn new() -> Self {
        Solver {
            x: Vec::new(),
            der_x: Vec::new(),
            z: Vec::new(),
            pre_z: Vec::new(),
        }
    }

    pub fn reset(&mut self, nx: usize, nz: usize) {
        self.x = vec![0.0; nx];
        self.der_x = vec![0.0; nx];
        self.z = vec![0.0; nz];
        self.pre_z = vec![0.0; nz];
    }

    pub fn step(&mut self, model: &mut dyn BaseModel, h: f64) -> (bool, fmiStatus) {
        
        let status = model.get_continuous_states(&mut self.x);
        
        if status != fmiStatus::fmiOK {
            return (false, status);
        }
        
        let status = model.get_continuous_state_derivatives(&mut self.der_x);
        
        if status != fmiStatus::fmiOK {
            return (false, status);
        }
        
        for i in 0..self.x.len() {
            self.x[i] += self.der_x[i] * h;
        }
        
        model.set_continuous_states(&self.x);

        let status = model.get_event_indicators(&mut self.z);

        if status != fmiStatus::fmiOK {
            return (false, status);
        }

        let mut zero_crossing_occurred = false;

        for i in 0..self.z.len() {
            if self.pre_z[i] < 0.0 && self.z[i] >= 0.0 || self.pre_z[i] > 0.0 && self.z[i] <= 0.0 {
                zero_crossing_occurred = true;
                break;
            }
        }

        self.pre_z = self.z.clone();

        (zero_crossing_occurred, fmiStatus::fmiOK)
    }

}

pub trait BaseModel {

    fn log_error(&self, message: &str);

    fn time(&self) -> f64;

    // fn solver(&self) -> Option<&Solver>;
    
    // fn solver_mut(&mut self) -> &mut Option<Solver>;

    fn solver(&mut self) -> Option<Solver>;

    fn set_solver(&mut self, solver: Solver);

    // fn interface_type(&self) -> &InterfaceType;
    
    // fn interface_type_mut(&mut self) -> &mut InterfaceType;

    fn do_fixed_step(&mut self, current_time: f64, step_size: f64) -> fmiStatus where Self: Sized {

        let solver = self.solver();

        if let Some(mut solver) = solver {
            let (zero_crossing_occurred, _status) = solver.step(self, step_size);
            if zero_crossing_occurred {
                self.update_discrete_states();
            }
            self.set_solver(solver);
            fmiStatus::fmiOK
        } else {
            fmiStatus::fmiError
        }

        // let mut solver_opt = None;

        // if let InterfaceType::CoSimulation(solver) = self.interface_type_mut() {
        //     solver_opt = solver.take();
        // }

        // if let Some(mut solver) = solver_opt {
        //     let (zero_crossing_occurred, _status) = solver.step(self, step_size);
        //     if zero_crossing_occurred {
        //         self.update_discrete_states();
        //     }
        //     if let InterfaceType::CoSimulation(s) = self.interface_type_mut() {
        //         *s = Some(solver);
        //     }
        //     fmiStatus::fmiOK
        // } else {
        //     self.log_error("do_fixed_step called on a model that is not a CoSimulation");
        //     fmiStatus::fmiError
        // }

        // fmiStatus::fmiOK
    }

    // fn solver(&mut self) -> &mut Solver;

    fn set_mode(&mut self, mode: ModelMode);

    fn enter_initialization_mode(&mut self) -> fmiStatus {
        self.set_mode(ModelMode::InitializationMode);
        fmiStatus::fmiOK
    }

    fn enter_continuous_time_mode(&mut self) -> fmiStatus {
        self.set_mode(ModelMode::ContinuousTimeMode);
        fmiStatus::fmiOK
    }

    fn exit_initialization_mode(&mut self) -> fmiStatus {

        let nx = self.get_number_of_continuous_states();
        let nz = self.get_number_of_event_indicators();

        let mode = if let Some(mut solver) = self.solver() {
            solver.reset(nx, nz);
            self.set_solver(solver);
            ModelMode::StepMode
        } else {
            ModelMode::EventMode
        };

        self.set_mode(mode);

        // let nx = self.get_number_of_continuous_states();
        // let nz = self.get_number_of_event_indicators();

        // self.solver().reset(nx, nz);

        fmiStatus::fmiOK
    }

    fn set_time(&mut self, time: f64) -> fmiStatus;

    fn get_event_indicators(&self, z: &mut [f64]) -> fmiStatus {
        if z.len() > 0 {
            self.log_error("This model has no event indicators");
            fmiStatus::fmiError
        } else {
            fmiStatus::fmiOK
        }
    }

    fn get_continuous_states(&self, x: &mut [f64]) -> fmiStatus {
        if x.len() > 0 {
            self.log_error("get_continuous_states: This model has no continuous states");
            return fmiStatus::fmiError;
        } else {
            fmiStatus::fmiOK
        }
    }

    fn get_nominals_of_continuous_states(&self, nominals: &mut [f64]) -> fmiStatus {
        if nominals.len() > 0 {
            self.log_error("This model has no continuous states");
            return fmiStatus::fmiError;
        } else {
            fmiStatus::fmiOK
        }
    }

    fn update_discrete_states(&mut self) -> fmiStatus {
        fmiStatus::fmiOK
    }

    fn set_continuous_states(&mut self, x: &[f64]) -> fmiStatus {
        if x.len() > 0 {
            self.log_error("This model has no continuous states");
            return fmiStatus::fmiError;
        } else {
            fmiStatus::fmiOK
        }
    }

    fn get_continuous_state_derivatives(&self, der_x: &mut [f64]) -> fmiStatus {
        if der_x.len() > 0 {
            self.log_error("This model has no continuous states");
            return fmiStatus::fmiError;
        } else {
            fmiStatus::fmiOK
        }
    }

    fn get_number_of_continuous_states(&self) -> usize {
        0
    }

    fn get_number_of_event_indicators(&self) -> usize {
        0
    }

    fn get_Float64(&self, value_reference: u32, _value: &mut f64) -> fmiStatus {
        let message = format!("Unknown value reference for type Float64: {:?}", value_reference);
        self.log_error(&message);
        fmiStatus::fmiError
    }
}


/// Macro to automatically implement TryFrom<u32> for value reference enums
/// 
/// Usage:
/// ```
/// fmi_export::impl_value_reference! {
///     #[derive(Debug)]
///     enum ValueReference {
///         time = 0,
///         x = 1,
///         der_x = 2,
///         k = 3,
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_value_reference {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident = $value:expr),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[repr(u32)]
        $vis enum $name {
            $($variant = $value),*
        }

        impl TryFrom<u32> for $name {
            type Error = ();

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                match value {
                    $($value => Ok($name::$variant),)*
                    _ => Err(()),
                }
            }
        }
    };
}
