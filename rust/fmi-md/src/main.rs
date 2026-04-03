//!
//! FMI Model Description Validator
//!

// FFI bindings to libxml2 validation code
unsafe extern "C" {

    // int validate_model_description(const char* model_description_path, int fmi_major_version, char*** messages)
    pub fn validate_model_description(
        model_description_path: *const i8,
        fmi_major_version: i32,
        messages: *mut *mut *const i8,
    ) -> i32;

}

fn main() {
    use std::ffi::CString;

    let model_description_path =
        r"E:\WS\Reference-FMUs\rust\BouncingBall\fmi3\modelDescription.xml";
    let fmi_major_version = 3;

    // Convert to CString (null-terminated C string)
    let path_cstring = CString::new(model_description_path).expect("Path contains null bytes");

    let mut messages: *mut *const i8 = std::ptr::null_mut();
    let n_messages = unsafe {
        validate_model_description(path_cstring.as_ptr(), fmi_major_version, &mut messages)
    };

    println!("Number of validation messages: {n_messages}");
    if n_messages > 0 && !messages.is_null() {
        let messages_slice = unsafe { std::slice::from_raw_parts(messages, n_messages as usize) };
        for (i, &msg_ptr) in messages_slice.iter().enumerate() {
            if !msg_ptr.is_null() {
                let msg_cstr = unsafe { std::ffi::CStr::from_ptr(msg_ptr) };
                let msg_str = msg_cstr.to_str().unwrap_or("<invalid UTF-8>");
                println!("Message {}: {}", i + 1, msg_str);
            }
        }
    } else {
        println!("Model description is valid according to the schema.");
    }

    // // Find xmllint.exe - check multiple locations
    // let xmllint_paths = vec![
    //     // First, try in the same directory as the executable
    //     std::env::current_exe()
    //         .ok()
    //         .and_then(|exe_path| exe_path.parent().map(|p| p.join("xmllint.exe"))),
    //     // Try in resources subdirectory
    //     std::env::current_dir()
    //         .ok()
    //         .map(|cwd| cwd.join("resources/xmllint.exe")),
    //     // Try relative to cargo manifest (during development)
    //     std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //         .parent()
    //         .map(|p| p.join("fmi-md/resources/xmllint.exe")),
    // ];

    // let mut xmllint_exe = None;
    // for path_opt in xmllint_paths {
    //     if let Some(path) = path_opt {
    //         if path.exists() {
    //             xmllint_exe = Some(path);
    //             break;
    //         }
    //     }
    // }

    // let xmllint_exe = xmllint_exe.expect(
    //     "Could not find xmllint.exe. Make sure it's in: \
    //      1) Same directory as the executable, or \
    //      2) In resources/ subdirectory"
    // );

    // let schema_path = r"C:\Users\tsr2\Downloads\fmi-standard-3.0.2\schema\fmi3ModelDescriptionFlat.xsd";
    // let xml_path = r"E:\WS\Reference-FMUs\rust\BouncingBall\fmi3\modelDescription.xml";

    // // Call xmllint.exe with schema validation
    // let output = std::process::Command::new(&xmllint_exe)
    //     .args(&["--noout", "--schema", schema_path, xml_path])
    //     .output()
    //     .expect("Failed to execute xmllint.exe");

    // let exit_code = output.status.code().unwrap_or(-1);
    // let stdout = String::from_utf8_lossy(&output.stdout);
    // let stderr = String::from_utf8_lossy(&output.stderr);

    // // Display results
    // if exit_code == 0 {
    //     println!("✓ XML document is valid according to the schema");
    //     if !stdout.is_empty() {
    //         println!("\n{}", stdout);
    //     }
    // } else {
    //     eprintln!("✗ XML document validation failed (exit code: {})", exit_code);
    //     if !stderr.is_empty() {
    //         eprintln!("\n{}", stderr);
    //     }
    //     if !stdout.is_empty() {
    //         eprintln!("\n{}", stdout);
    //     }
    // }
}
