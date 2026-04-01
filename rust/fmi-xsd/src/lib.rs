use std::path::Path;
use std::sync::Mutex;

// Global mutex to ensure validate_model_description is only called by one thread at a time
// This is necessary because libxml2 has internal state that isn't thread-safe
static VALIDATION_LOCK: Mutex<()> = Mutex::new(());


pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

unsafe extern "C" {

    // int validate_model_description(const char* model_description_path, int fmi_major_version, char*** messages)
    fn validate_model_description(
        model_description_path: *const i8,
        fmi_major_version: i32,
        messages: *mut *mut *const i8,
    ) -> i32;

}

pub fn validate_model_description_against_xsd(model_description_path: &Path, fmi_major_version: i32) -> Result<(), Vec<String>> {

    use std::ffi::CString;

    let path = model_description_path.to_str().unwrap();

    let path_cstring = CString::new(path)
        .expect("Path contains null bytes");

    let mut messages: *mut *const i8 = std::ptr::null_mut();
    
    // Acquire lock to ensure thread-safe access to the C function
    let _lock = VALIDATION_LOCK.lock().unwrap();
    
    let n_messages = unsafe { 
        validate_model_description(
            path_cstring.as_ptr(),
            fmi_major_version,
            &mut messages
        ) 
    };

    if n_messages > 0 && !messages.is_null() {
        let messages_slice = unsafe { std::slice::from_raw_parts(messages, n_messages as usize) };
        let messages_vec = messages_slice.iter().filter_map(|&msg_ptr| {
            if !msg_ptr.is_null() {
                let msg_cstr = unsafe { std::ffi::CStr::from_ptr(msg_ptr) };
                msg_cstr.to_str().ok().map(|s| s.to_string())
            } else {
                None
            }
        }).collect();
        Err(messages_vec)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_validate_model_description_against_xsd() {
        let model_description_path = Path::new(r"E:\WS\Reference-FMUs\rust\BouncingBall\fmi3\modelDescription.xml");
        match validate_model_description_against_xsd(model_description_path, 3) {
            Ok(()) => println!("Model description is valid."),
            Err(messages) => {
                println!("Model description is invalid. Validation messages:");
                for msg in messages {
                    println!("- {}", msg);
                }
            }
        }
    }
}
