use std::path::Path;
use fmi::fmi3::FMU3;

fn main() {

    let log_fmi_call = |status: &fmi::types::fmiStatus, message: &str| {
        println!("[FMICall][{:?}] {}", status, message);
    };

    let log_message = |status: &fmi::types::fmiStatus, category: &str, message: &str| {
        println!("[Message][{:?}][{}] {}", status, category, message);
    };

    let path = Path::new(r"E:\WS\Reference-FMUs\rust\deploy\binaries\x86_64-windows\BouncingBall.dll");

    let mut fmu = FMU3::new(path, "instance1", Some(Box::new(log_fmi_call)), Some(Box::new(log_message))).expect("Failed to load FMU");

    fmu.instantiateCoSimulation("instance1", "{1AE5E10D-9521-4DE3-80B9-D0EAAA7D5AF1}", None, false, false, false, false, &[]);

    fmu.enterInitializationMode(None, 0.0, Some(1.0));

    fmu.exitInitializationMode();

    let value_references = [1];
    let mut values = [0.0];

    let mut eventHandlingNeeded: bool = false;
    let mut terminateSimulation: bool = false;
    let mut earlyReturn: bool = false;
    let mut lastSuccessfulTime: fmi::types::fmiFloat64 = 0.0;

    fmu.doStep(0.0, 0.1, true, &mut eventHandlingNeeded, &mut terminateSimulation, &mut earlyReturn, &mut lastSuccessfulTime); 

    fmu.getFloat64(&value_references, &mut values);

    fmu.terminate();

    fmu.freeInstance();
}
