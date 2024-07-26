use stdext::function_name;

mod common;

#[test]
fn integer_addition_1() {
    let program = r#"
int x = 5;
x + 3 -> @;
"#;

    let expected_output = "8";

    assert_eq!(
        common::compile_and_get_stripped_output(program, function_name!()),
        expected_output
    );
}

#[test]
fn integer_addition_2() {
    let program = r#"
int x = 5;
x + -4 -> @;
"#;

    let expected_output = "1";

    assert_eq!(
        common::compile_and_get_stripped_output(program, function_name!()),
        expected_output
    );
}

#[test]
fn integer_addition_3() {
    let program = r#"
int x = 5;
x + -7 -> @;
"#;

    let expected_output = "-2";

    assert_eq!(
        common::compile_and_get_stripped_output(program, function_name!()),
        expected_output
    );
}

#[test]
fn integer_subtraction_1() {
    let program = r#"
int x = 5;
x - 4 -> @;
"#;

    let expected_output = "1";

    assert_eq!(
        common::compile_and_get_stripped_output(program, function_name!()),
        expected_output
    );
}

#[test]
fn integer_subtraction_2() {
    let program = r#"
int x = 5;
x - -5 -> @;
"#;

    let expected_output = "10";

    assert_eq!(
        common::compile_and_get_stripped_output(program, function_name!()),
        expected_output
    );
}

#[test]
fn integer_subtraction_3() {
    let program = r#"
int x = 5;
x - 8 -> @;
"#;

    let expected_output = "-3";

    assert_eq!(
        common::compile_and_get_stripped_output(program, function_name!()),
        expected_output
    );
}
