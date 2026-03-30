//!
//! FMI Model Description Validator using libxml2
//!
use std::ffi::{CStr, CString};

// FFI bindings to libxml2
#[link(name = "libxml2s", kind = "static")]
unsafe extern "C" {
    pub fn xmlSchemaNewMemParserCtxt(buffer: *const u8, size: i32) -> *mut XmlSchemaParserCtxt;
    pub fn xmlSchemaParse(parserCtxt: *mut XmlSchemaParserCtxt) -> *mut XmlSchema;
    pub fn xmlSchemaNewValidCtxt(schema: *mut XmlSchema) -> *mut XmlSchemaValidCtxt;
    pub fn xmlSchemaValidateFile(
        ctxt: *mut XmlSchemaValidCtxt,
        filename: *const u8,
    ) -> i32;
    pub fn xmlSchemaSetValidErrors(
        ctxt: *mut XmlSchemaValidCtxt,
        err: Option<unsafe extern "C" fn(*mut std::os::raw::c_void, *const u8)>,
        warn: Option<unsafe extern "C" fn(*mut std::os::raw::c_void, *const u8)>,
        ctx: *mut std::os::raw::c_void,
    );
    pub fn xmlGetLastError() -> *mut XmlError;
    pub fn xmlResetError(err: *mut XmlError);
    pub fn xmlSchemaFreeParserCtxt(ctxt: *mut XmlSchemaParserCtxt);
    pub fn xmlSchemaFreeValidCtxt(ctxt: *mut XmlSchemaValidCtxt);
    pub fn xmlSchemaFree(schema: *mut XmlSchema);
}

// Opaque struct types from libxml2
#[repr(C)]
pub struct XmlSchemaParserCtxt;

#[repr(C)]
pub struct XmlSchema;

#[repr(C)]
pub struct XmlSchemaValidCtxt;

#[repr(C)]
pub struct XmlError {
    pub domain: i32,
    pub code: i32,
    pub message: *mut u8,
    pub level: i32,
    pub file: *mut u8,
    pub line: i32,
    pub str1: *mut u8,
    pub str2: *mut u8,
    pub str3: *mut u8,
    pub int1: i32,
    pub int2: i32,
    pub ctx: *mut std::os::raw::c_void,
    pub node: *mut std::os::raw::c_void,
}

// Global error collector
std::thread_local! {
    static VALIDATION_ERRORS: std::cell::RefCell<Vec<String>> = std::cell::RefCell::new(Vec::new());
}

// Error callback function - just collect error happened flag
unsafe extern "C" fn error_callback(ctx: *mut std::os::raw::c_void, msg: *const u8) {
    // Error will be retrieved via xmlGetLastError()
}

// Warning callback function
unsafe extern "C" fn warning_callback(ctx: *mut std::os::raw::c_void, msg: *const u8) {
    if msg.is_null() {
        return;
    }
    
    if let Ok(msg_str) = CStr::from_ptr(msg as *const i8).to_str() {
        VALIDATION_ERRORS.with(|errors| {
            errors.borrow_mut().push(format!("WARNING: {}", msg_str));
        });
    }
}

fn main() {
    let xsd_path = r"C:\Users\tsr2\Downloads\fmi-standard-3.0.2\schema\fmi3ModelDescriptionFlat.xsd";
    let xml_path = r"E:\WS\Reference-FMUs\rust\BouncingBall\fmi3\modelDescription.xml";

    unsafe {
        // Read XSD schema from file
        let schema_content = std::fs::read(xsd_path)
            .expect("Failed to read XSD file");

        // Create parser context from memory buffer
        let parser_ctxt = xmlSchemaNewMemParserCtxt(
            schema_content.as_ptr(),
            schema_content.len() as i32,
        );

        if parser_ctxt.is_null() {
            eprintln!("Failed to create schema parser context");
            return;
        }

        // Parse the schema
        let schema = xmlSchemaParse(parser_ctxt);
        xmlSchemaFreeParserCtxt(parser_ctxt);

        if schema.is_null() {
            eprintln!("Failed to parse schema");
            return;
        }

        // Create validation context
        let valid_ctxt = xmlSchemaNewValidCtxt(schema);
        if valid_ctxt.is_null() {
            eprintln!("Failed to create schema validation context");
            xmlSchemaFree(schema);
            return;
        }

        // Set error handlers to capture validation errors
        xmlSchemaSetValidErrors(
            valid_ctxt,
            Some(error_callback),
            Some(warning_callback),
            std::ptr::null_mut(),
        );

        // Clear any previous errors
        VALIDATION_ERRORS.with(|errors| {
            errors.borrow_mut().clear();
        });

        // Validate XML file
        let xml_c_path = CString::new(xml_path).expect("Invalid XML path");
        let result = xmlSchemaValidateFile(valid_ctxt, xml_c_path.as_ptr() as *const u8);

        // Collect all validation errors from libxml2
        loop {
            let err = xmlGetLastError();
            if err.is_null() {
                break;
            }
            
            let error_msg = if !(*err).message.is_null() {
                CStr::from_ptr((*err).message as *const i8)
                    .to_string_lossy()
                    .to_string()
            } else {
                "Unknown error".to_string()
            };
            
            let file = if !(*err).file.is_null() {
                CStr::from_ptr((*err).file as *const i8)
                    .to_string_lossy()
                    .to_string()
            } else {
                "(no file)".to_string()
            };
            
            let error_line = format!("{}:{}: {}", file, (*err).line, error_msg);
            
            VALIDATION_ERRORS.with(|errors| {
                errors.borrow_mut().push(error_line);
            });
            
            xmlResetError(err);
        }

        // Cleanup
        xmlSchemaFreeValidCtxt(valid_ctxt);
        xmlSchemaFree(schema);

        // Display results
        if result == 0 {
            println!("✓ XML document is valid according to the schema");
        } else {
            eprintln!("✗ XML document validation failed (code: {})", result);
        }

        // Display collected errors and warnings
        VALIDATION_ERRORS.with(|errors| {
            let errors_vec = errors.borrow();
            if !errors_vec.is_empty() {
                println!("\nValidation Messages:");
                for error in errors_vec.iter() {
                    println!("  {}", error);
                }
            }
        });
    }
}