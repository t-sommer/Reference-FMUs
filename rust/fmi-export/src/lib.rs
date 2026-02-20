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
