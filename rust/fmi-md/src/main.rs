//!
//! FMI Model Description Validator using libxml2
//!
use std::ffi::CString;

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

        // Validate XML file
        let xml_c_path = CString::new(xml_path).expect("Invalid XML path");
        let result = xmlSchemaValidateFile(valid_ctxt, xml_c_path.as_ptr() as *const u8);

        // Cleanup
        xmlSchemaFreeValidCtxt(valid_ctxt);
        xmlSchemaFree(schema);

        if result == 0 {
            println!("✓ XML document is valid according to the schema");
        } else {
            eprintln!("✗ XML document validation failed (code: {})", result);
        }
    }
}