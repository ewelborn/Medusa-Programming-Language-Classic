mod common;

#[test]
fn hello_world() {
    let program = r#"
"Hello, world!" -> @;
"#;

    let expected_output = "Hello, world!";

    assert_eq!(
        common::compile_and_get_stripped_output(program),
        expected_output
    );
}
