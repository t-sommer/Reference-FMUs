use std::{fs::{File, read_to_string}, io::Read, path::PathBuf, process::Command, str::FromStr};
use rstest::*;
use tempfile::TempDir;

#[fixture]
pub fn workspace_root() -> PathBuf {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
    workspace_root
}

#[rstest]
fn should_success(workspace_root: PathBuf) {

    // dbg!(&workspace_root);

    // let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let fmu_file = workspace_root
        .join("fmusim")
        .join("tests")
        .join("resources")
        .join("Reference-FMUs")
        .join("2.0")
        .join("Feedthrough.fmu");

    let input_file = workspace_root
        .join("fmusim")
        .join("tests")
        .join("resources")
        .join("fmi2")
        .join("Feedthrough_interpolation_in.csv");

    let output_file = workspace_root
        .join("fmusim")
        .join("tests")
        .join("resources")
        .join("fmi2")
        .join("Feedthrough_interpolation_out.csv");

    // let output_file = PathBuf::from_str(r"E:\WS\Reference-FMUs\rust\fmusim\tests\resources\fmi2\Feedthrough_types_out.csv").unwrap(); // temp_dir.path().join("Feedthrough_types_out.csv");

    let expected_output_file = workspace_root
        .join("fmusim")
        .join("tests")
        .join("resources")
        .join("fmi2")
        .join("Feedthrough_interpolation_expected.csv");

    let fmusim_path = workspace_root.join("target/debug/fmusim");

    let simulation_output = Command::new(&fmusim_path)
        .args(&[
            &fmu_file.to_string_lossy(),
            "--input-file", &input_file.to_string_lossy(),
            "--output-file", &output_file.to_string_lossy(),
            "--log-fmi-calls",
            "--stop-time=3",
            "--output-interval=0.1",
            "--output-variable=Float64_continuous_input",
        ])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to run fmusim");

    let out = String::from_utf8_lossy(&simulation_output.stdout);
    print!("{out}");

    let err = String::from_utf8_lossy(&simulation_output.stderr);
    print!("{err}");
    
    if !simulation_output.status.success() {
        panic!(
            "Simulation failed: {}\nStderr: {}",
            String::from_utf8_lossy(&simulation_output.stdout),
            String::from_utf8_lossy(&simulation_output.stderr)
        );
    }

    let expected = read_to_string(expected_output_file).unwrap();
    let actual = read_to_string(output_file).unwrap();
    
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();
    
    // assert_eq!(expected_lines, actual_lines);
}