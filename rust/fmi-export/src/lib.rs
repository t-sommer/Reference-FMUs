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
            self.log_error("This model has no continuous states");
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
