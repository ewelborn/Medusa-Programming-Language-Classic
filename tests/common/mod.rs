// Retrieve all of the output from a compiled medusa program
pub fn compile_and_get_output(source_text: &str) -> String {
    medusa_lang::compile_from_text(source_text, "test".to_string()).unwrap();

    // File is available at test.exe
    let output = std::process::Command::new("./test.exe").output().unwrap();
    match output.status.code().unwrap() {
        0 => {
            String::from_utf8(output.stdout).unwrap()
        },
        _ => {
            panic!("Failed to execute test.exe")
        }
    }
}

// Retrieve only the program-specific output from a compiled medusa program, with all headers
//  and footers stripped (i.e. Medusa 1.0 and Program ended are removed) and all formatting
//  characters stripped (i.e. \n and \0)
pub fn compile_and_get_stripped_output(source_text: &str) -> String {
    let output = compile_and_get_output(source_text);
    
    let header = regex::Regex::new(r"Medusa 1.0").unwrap();
    let footer = regex::Regex::new(r"Program ended").unwrap();
    let formatting_characters = regex::Regex::new(r"(\n|\u{0})").unwrap();

    let output = header.replace(output.as_str(), "").to_string();
    let output = footer.replace(output.as_str(), "").to_string();
    let output = formatting_characters.replace_all(output.as_str(), "").to_string();

    return output;
}