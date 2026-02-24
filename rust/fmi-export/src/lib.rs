use fmi::types::fmiStatus;
use serde::{Deserialize, Serialize};

// Re-export the derive macro
pub use fmi_export_derive::ValueReference;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterfaceType {
    ModelExchange,
    CoSimulation,
}

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

    // fn solver(&mut self) -> Solver;

    // fn do_fixed_step(&mut self) {

        // let mut solver = self.solver();

        // solver.step(self, 0.1);

        // if let Some(mut solver) = self.data.solver.take() {
        //     solver.step(self, FIXED_STEP_SIZE);
        //     self.data.solver = Some(solver);
        // }

        // self.data.n_steps += 1;
        // self.data.time = self.data.n_steps as f64 * FIXED_STEP_SIZE;
    // }

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
