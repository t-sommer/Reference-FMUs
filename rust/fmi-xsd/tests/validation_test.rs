use fmi_xsd::validate_model_description_against_xsd;


#[test]
fn valid_model_description_fmi2() {
    let path = std::env::current_dir().unwrap().join("tests/resources/valid_fmi2.xml");
    let result = validate_model_description_against_xsd(&path, 2);
    assert_eq!(result, Ok(()));
}

#[test]
fn valid_model_description_fmi3() {
    let path = std::env::current_dir().unwrap().join("tests/resources/valid_fmi3.xml");
    let result = validate_model_description_against_xsd(&path, 3);
    assert_eq!(result, Ok(()));
}

#[test]
fn invalid_model_description_fmi2() {
    let path = std::env::current_dir().unwrap().join("tests/resources/invalid_fmi2.xml");
    let result = validate_model_description_against_xsd(&path, 2);
    assert_eq!(result, Err(vec![
        "Element 'ModelExchange', attribute 'canGetAndSetFMUstate': 'yes' is not a valid value of the atomic type 'xs:boolean'.\n".to_string()
    ]));
}

#[test]
fn invalid_model_description_fmi3() {
    let path = std::env::current_dir().unwrap().join("tests/resources/invalid_fmi3.xml");
    let result = validate_model_description_against_xsd(&path, 3);
    assert_eq!(result, Err(vec![
        "Element 'ModelExchange', attribute 'canGetAndSetFMUState': 'yes' is not a valid value of the atomic type 'xs:boolean'.\n".to_string()
    ]));
}
