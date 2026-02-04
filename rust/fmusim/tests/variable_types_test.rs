use std::fs;
use std::path::Path;

#[test]
fn test_variable_types() {

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/resources/example.csv");
    let contents = fs::read_to_string(path).expect("read fixture");

    

}