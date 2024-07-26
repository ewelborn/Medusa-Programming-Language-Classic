fn main() {
    let args: Vec<String> = std::env::args().collect();

    let input_file_name = if (args.len() > 1) {
        args[1].clone()
    } else {
        "input.med".to_string()
    };

    let output_file_name = if (args.len() > 2) {
        args[2].clone()
    } else {
        "medusa_output".to_string()
    };

    let source_text = std::fs::read_to_string(input_file_name.clone())
        .expect(format!("Could not read source file {}", input_file_name).as_str());

    match medusa_lang::compile_from_text(&source_text, output_file_name) {
        Ok(()) => {}
        Err(e) => {
            panic!("Compile error: {}", e);
        }
    };
}
