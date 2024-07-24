mod common;

#[test]
fn float_to_string_1() {
    let program = r#"
5.5 -> @;
"#;

    let expected_output = "5.500000";

    assert_eq!(
        common::compile_and_get_stripped_output(program),
        expected_output
    );
}

#[test]
fn string_to_float_1() {
    let program = r#"
float x = (float) "5.5";
x -> @;
"#;

    let expected_output = "5.500000";

    assert_eq!(
        common::compile_and_get_stripped_output(program),
        expected_output
    );
}

#[test]
fn string_to_float_2() {
    let program = r#"
float x = (float) "17.65";
x -> @;
"#;

    let expected_output = "17.649999";

    assert_eq!(
        common::compile_and_get_stripped_output(program),
        expected_output
    );
}

#[test]
fn string_to_float_3() {
    let program = r#"
float x = (float) "-6.125";
x -> @;
"#;

    let expected_output = "-6.125000";

    assert_eq!(
        common::compile_and_get_stripped_output(program),
        expected_output
    );
}