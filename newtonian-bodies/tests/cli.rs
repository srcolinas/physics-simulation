use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

// Helper function to create a temporary test input file
fn create_test_input_file(temp_dir: &TempDir) -> String {
    let input_content = r#"[
        {
            "name": "TestBody1",
            "mass": 1.0e24,
            "position": {
                "x": 0.0,
                "y": 0.0,
                "z": 0.0
            },
            "velocity": {
                "x": 0.0,
                "y": 0.0,
                "z": 0.0
            }
        },
        {
            "name": "TestBody2",
            "mass": 5.0e23,
            "position": {
                "x": 1000000.0,
                "y": 0.0,
                "z": 0.0
            },
            "velocity": {
                "x": 0.0,
                "y": 1000.0,
                "z": 0.0
            }
        }
    ]"#;
    
    let input_path = temp_dir.path().join("test_input.json");
    fs::write(&input_path, input_content).expect("Failed to write test input file");
    input_path.to_string_lossy().to_string()
}

#[test]
fn test_basic_functionality() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let input_file = create_test_input_file(&temp_dir);
    let output_file = temp_dir.path().join("test_output.parquet");
    
    // Run the CLI with basic arguments
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            &input_file,
            "-o", output_file.to_str().unwrap(),
            "-g", "6.67430e-11",
            "-t", "1.0",
            "-d", "0.1",
            "-r", "1"
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute CLI");
    
    // Check that the command succeeded
    assert!(output.status.success(), 
        "CLI failed with stderr: {}", 
        String::from_utf8_lossy(&output.stderr)
    );
    
    // Check that output file was created
    assert!(output_file.exists(), "Output file was not created");
    
    // Check that output file has content (not empty)
    let metadata = fs::metadata(&output_file).expect("Failed to get output file metadata");
    assert!(metadata.len() > 0, "Output file is empty");
}

#[test]
fn test_default_output_filename() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let input_file = create_test_input_file(&temp_dir);
    
    // Run the CLI without specifying output file (should use default)
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            &input_file,
            "-g", "6.67430e-11",
            "-t", "1.0",
            "-d", "0.1",
            "-r", "1"
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute CLI");
    
    // Check that the command succeeded
    assert!(output.status.success(), 
        "CLI failed with stderr: {}", 
        String::from_utf8_lossy(&output.stderr)
    );
    
    // Check that default output file was created in current directory
    let default_output = Path::new("newtonian.parquet");
    assert!(default_output.exists(), "Default output file was not created");
    
    // Clean up the default output file
    fs::remove_file(default_output).expect("Failed to remove default output file");
}

#[test]
fn test_expression_parsing() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let input_file = create_test_input_file(&temp_dir);
    let output_file = temp_dir.path().join("test_output.parquet");
    
    // Test with mathematical expressions in arguments
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            &input_file,
            "-o", output_file.to_str().unwrap(),
            "-g", "6.67430e-11",
            "-t", "60*60",  // 1 hour in seconds
            "-d", "1.0/1000.0",  // 0.001 seconds
            "-r", "60*10"  // 600 seconds
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute CLI");
    
    // Check that the command succeeded
    assert!(output.status.success(), 
        "CLI failed with stderr: {}", 
        String::from_utf8_lossy(&output.stderr)
    );
    
    // Check that output file was created
    assert!(output_file.exists(), "Output file was not created");
}


#[test]
fn test_long_arguments() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let input_file = create_test_input_file(&temp_dir);
    let output_file = temp_dir.path().join("test_output.parquet");
    
    // Test with long argument forms
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            &input_file,
            "--output", output_file.to_str().unwrap(),
            "--gravity", "6.67430e-11",
            "--total-time", "1.0",
            "--delta-t", "0.1",
            "--record-interval", "1"
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute CLI");
    
    // Check that the command succeeded
    assert!(output.status.success(), 
        "CLI failed with stderr: {}", 
        String::from_utf8_lossy(&output.stderr)
    );
    
    // Check that output file was created
    assert!(output_file.exists(), "Output file was not created");
}



#[test]
fn test_invalid_input_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let invalid_input = temp_dir.path().join("nonexistent.json");
    
    // Test with non-existent input file
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            invalid_input.to_str().unwrap()
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute CLI");
    
    // Check that the command failed (as expected)
    assert!(!output.status.success(), "CLI should fail with invalid input file");
    
    // Check that error message contains relevant information
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No such file") || stderr.contains("not found"), 
        "Error message should indicate file not found: {}", stderr);
}


#[test]
fn test_invalid_gravity_expression() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let input_file = create_test_input_file(&temp_dir);
    
    // Test with invalid gravity expression
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            &input_file,
            "-g", "invalid_expression"
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute CLI");
    
    // Check that the command failed (as expected)
    assert!(!output.status.success(), "CLI should fail with invalid gravity expression");
    
    // Check that error message contains relevant information
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("invalid"), 
        "Error message should indicate expression parsing error: {}", stderr);
}

#[test]
fn test_output_file_permissions() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let input_file = create_test_input_file(&temp_dir);
    
    // Run the CLI to generate output file
    let output = Command::new("cargo")
        .args(&[
            "run", "--",
            &input_file,
            "-o", "test_output.parquet",
            "-g", "6.67430e-11",
            "-t", "1.0",
            "-d", "0.1",
            "-r", "1"
        ])
        .current_dir(".")
        .output()
        .expect("Failed to execute CLI");
    
    // Check that the command succeeded
    assert!(output.status.success(), 
        "CLI failed with stderr: {}", 
        String::from_utf8_lossy(&output.stderr)
    );
    
    // Check that output file was created and is readable
    let output_file_path = Path::new("test_output.parquet");
    assert!(output_file_path.exists(), "Output file was not created");
    
    // Try to read the file to verify permissions
    let content = fs::read(output_file_path).expect("Failed to read output file");
    assert!(!content.is_empty(), "Output file is empty");
    
    // Check file metadata
    let metadata = fs::metadata(output_file_path).expect("Failed to get output file metadata");
    assert!(metadata.is_file(), "Output should be a regular file");
    assert!(metadata.len() > 0, "Output file should have content");
    
    // Clean up the output file
    fs::remove_file(output_file_path).expect("Failed to remove test output file");
}
