use serde::{Deserialize, Serialize};

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