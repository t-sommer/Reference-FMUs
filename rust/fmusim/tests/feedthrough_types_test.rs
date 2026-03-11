use std::{fs::File, io::Read, path::PathBuf, process::Command};
use tempfile::TempDir;

// #[test]
fn test_feedthrough_types_simulation() {
    // Get the workspace root directory
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
    
    // Paths to input and expected output files
    let input_file = workspace_root.join("Feedthrough_types_in.csv");
    let expected_output_file = workspace_root.join("Feedthrough_types_expected_out.csv");
    
    // Create FMU from test resources
    let feedthrough_dir = workspace_root.join("fmi/tests/resources/fmi3/Feedthrough");
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let fmu_file = temp_dir.path().join("Feedthrough.fmu");
    
    // Create the FMU by zipping the Feedthrough directory
    create_fmu_from_directory(&feedthrough_dir, &fmu_file);
    
    // Create output file path
    let output_file = temp_dir.path().join("output.csv");
    
    // Build the fmusim binary
    let output = Command::new("cargo")
        .args(&["build", "--bin", "fmusim"])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to build fmusim");
    
    if !output.status.success() {
        panic!("Failed to build fmusim: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Run the simulation
    let fmusim_path = workspace_root.join("target/debug/fmusim");
    let simulation_output = Command::new(&fmusim_path)
        .args(&[
            &fmu_file.to_string_lossy(),
            "--input-file", &input_file.to_string_lossy(),
            "--output-file", &output_file.to_string_lossy(),
            "--start-time", "0.0",
            "--stop-time", "3.0",
            "--output-interval", "0.5",
        ])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to run fmusim");
    
    if !simulation_output.status.success() {
        panic!(
            "Simulation failed: {}\nStderr: {}",
            String::from_utf8_lossy(&simulation_output.stdout),
            String::from_utf8_lossy(&simulation_output.stderr)
        );
    }
    
    // Read the actual output
    let mut actual_output = String::new();
    File::open(&output_file)
        .expect("Failed to open output file")
        .read_to_string(&mut actual_output)
        .expect("Failed to read output file");
    
    // Read the expected output
    let mut expected_output = String::new();
    File::open(&expected_output_file)
        .expect("Failed to open expected output file")
        .read_to_string(&mut expected_output)
        .expect("Failed to read expected output file");
    
    // Compare the outputs
    compare_csv_outputs(&actual_output, &expected_output);
}

fn create_fmu_from_directory(source_dir: &PathBuf, fmu_path: &PathBuf) {
    use zip::{ZipWriter, write::FileOptions, CompressionMethod};
    
    let file = File::create(fmu_path).expect("Failed to create FMU file");
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Stored);
    
    // Add all files from the source directory to the ZIP
    add_dir_to_zip(&mut zip, source_dir, source_dir, &options).expect("Failed to create FMU");
    
    zip.finish().expect("Failed to finalize FMU");
}

fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<File>,
    source_dir: &PathBuf,
    base_dir: &PathBuf,
    options: &zip::write::FileOptions<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(base_dir)?;
        
        if path.is_dir() {
            // Add directory
            zip.add_directory(relative_path.to_string_lossy(), *options)?;
            // Recursively add contents
            add_dir_to_zip(zip, &path, base_dir, options)?;
        } else {
            // Add file
            zip.start_file(relative_path.to_string_lossy(), *options)?;
            let mut file = File::open(&path)?;
            std::io::copy(&mut file, zip)?;
        }
    }
    
    Ok(())
}

fn compare_csv_outputs(actual: &str, expected: &str) {
    let actual_lines: Vec<&str> = actual.trim().lines().collect();
    let expected_lines: Vec<&str> = expected.trim().lines().collect();
    
    // Compare number of lines
    assert_eq!(
        actual_lines.len(),
        expected_lines.len(),
        "Number of output lines differs. Expected: {}, Actual: {}",
        expected_lines.len(),
        actual_lines.len()
    );
    
    // Compare headers
    if !actual_lines.is_empty() && !expected_lines.is_empty() {
        assert_eq!(
            actual_lines[0],
            expected_lines[0],
            "CSV headers differ"
        );
    }
    
    // Compare data rows
    for (i, (actual_line, expected_line)) in actual_lines.iter().zip(expected_lines.iter()).enumerate() {
        if i == 0 {
            continue; // Skip header
        }
        
        let actual_values: Vec<&str> = actual_line.split(',').collect();
        let expected_values: Vec<&str> = expected_line.split(',').collect();
        
        assert_eq!(
            actual_values.len(),
            expected_values.len(),
            "Number of columns differs in line {}: Expected: {}, Actual: {}",
            i + 1,
            expected_values.len(),
            actual_values.len()
        );
        
        // Compare each value with tolerance for floating point numbers
        for (j, (actual_val, expected_val)) in actual_values.iter().zip(expected_values.iter()).enumerate() {
            let actual_val = actual_val.trim_matches('"');
            let expected_val = expected_val.trim_matches('"');
            
            // Try to parse as floating point numbers for comparison with tolerance
            if let (Ok(actual_f), Ok(expected_f)) = (actual_val.parse::<f64>(), expected_val.parse::<f64>()) {
                let tolerance = 1e-10;
                assert!(
                    (actual_f - expected_f).abs() < tolerance,
                    "Floating point values differ beyond tolerance at line {}, column {}: Expected: {}, Actual: {}, Difference: {}",
                    i + 1,
                    j + 1,
                    expected_f,
                    actual_f,
                    (actual_f - expected_f).abs()
                );
            } else {
                // For non-numeric values, do exact string comparison
                assert_eq!(
                    actual_val,
                    expected_val,
                    "Values differ at line {}, column {}: Expected: '{}', Actual: '{}'",
                    i + 1,
                    j + 1,
                    expected_val,
                    actual_val
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_csv_comparison() {
        let actual = r#""time","value1","value2"
0.5,1.23,hello
1.0,2.46,world"#;
        
        let expected = r#""time","value1","value2"
0.5,1.23,hello
1.0,2.46,world"#;
        
        compare_csv_outputs(actual, expected);
    }
    
    #[test]
    #[should_panic(expected = "Floating point values differ beyond tolerance")]
    fn test_csv_comparison_float_difference() {
        let actual = r#""time","value"
0.5,1.23456789"#;
        
        let expected = r#""time","value"
0.5,1.23456788"#;
        
        compare_csv_outputs(actual, expected);
    }
}