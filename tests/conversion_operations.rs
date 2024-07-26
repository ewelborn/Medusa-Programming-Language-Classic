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

#[test]
fn string_to_float_4() {
    let program = r#"
float x = (float) "4.2abw";
x -> @;
"#;

    let expected_output = "0.000000";

    assert_eq!(
        common::compile_and_get_stripped_output(program),
        expected_output
    );
}

#[test]
fn string_to_float_5() {
    let program = r#"
float x = (float) "5x.29";
x -> @;
"#;

    let expected_output = "0.000000";

    assert_eq!(
        common::compile_and_get_stripped_output(program),
        expected_output
    );
}

#[test]
fn string_to_float_6() {
    let program = r#"
float x = (float) "123.0";
x -> @;
"#;

    let expected_output = "123.000000";

    assert_eq!(
        common::compile_and_get_stripped_output(program),
        expected_output
    );
}

#[test]
fn string_to_float_7() {
    let program = r#"
float x = (float) "0.564";
x -> @;
"#;

    let expected_output = "0.564000";

    assert_eq!(
        common::compile_and_get_stripped_output(program),
        expected_output
    );
}

#[test]
fn string_to_int_1() {
    let program = r#"
int x = (int) "15";
x -> @;
"#;

    let expected_output = "15";

    assert_eq!(
        common::compile_and_get_stripped_output(program),
        expected_output
    );
}

#[test]
fn string_to_int_2() {
    let program = r#"
int x = (int) "-174";
x -> @;
"#;

    let expected_output = "-174";

    assert_eq!(
        common::compile_and_get_stripped_output(program),
        expected_output
    );
}

#[test]
fn string_to_int_3() {
    let program = r#"
int x = (int) "6ac2";
x -> @;
"#;

    let expected_output = "0";

    assert_eq!(
        common::compile_and_get_stripped_output(program),
        expected_output
    );
}
