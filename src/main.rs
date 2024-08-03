fn main() {
    let args: Vec<String> = std::env::args().collect();

    //println!("{}", args[0]);

    let input_file_name = if args.len() > 1 {
        args[1].clone()
    } else {
        panic!("No source file was provided. Pass in a file path as the first argument to the program.")
    };

    let output_file_name = if args.len() > 2 {
        args[2].clone()
    } else {
        // If no output file name was provided, use the same name as the source file, but strip off the
        // file extension (if it exists) (TODO)
        args[1].clone()
    };

    let source_text = std::fs::read_to_string(input_file_name.clone())
        .expect(format!("Could not read source file {}", input_file_name).as_str());

    match medusa_lang::compile_from_text(&source_text, &output_file_name) {
        Ok(()) => {}
        Err(e) => {
            panic!("Compile error: {}", e);
        }
    };
}
