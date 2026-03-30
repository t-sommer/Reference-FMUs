//!
//! FMI Model Description Validator using xmllint.exe
//!
use std::process::Command;

fn main() {
    let xmllint_exe = r"C:\Users\tsr2\Downloads\libxml2-v2.15.2\install\bin\xmllint.exe";
    let schema_path = r"C:\Users\tsr2\Downloads\fmi-standard-3.0.2\schema\fmi3ModelDescription.xsd";
    let xml_path = r"E:\WS\Reference-FMUs\rust\BouncingBall\fmi3\modelDescription.xml";

    // Call xmllint.exe with schema validation
    let output = Command::new(xmllint_exe)
        .args(&["--noout", "--schema", schema_path, xml_path])
        .output()
        .expect("Failed to execute xmllint.exe");

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Display results
    if exit_code == 0 {
        println!("✓ XML document is valid according to the schema");
        if !stdout.is_empty() {
            println!("\n{}", stdout);
        }
    } else {
        eprintln!("✗ XML document validation failed (exit code: {})", exit_code);
        if !stderr.is_empty() {
            eprintln!("\n{}", stderr);
        }
        if !stdout.is_empty() {
            eprintln!("\n{}", stdout);
        }
    }
}