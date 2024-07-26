use stdext::function_name;

mod common;

#[test]
fn hello_world() {
    let program = r#"
"Hello, world!" -> @;
"#;

    let expected_output = "Hello, world!";

    assert_eq!(
        common::compile_and_get_stripped_output(program, function_name!()),
        expected_output
    );
}
