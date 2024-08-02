// Retrieve all of the output from a compiled medusa program
pub fn compile_and_get_output(source_text: &str, test_name: &str) -> String {
    // The test name is in this kind of format: conversion_operations::string_to_int_3
    // Which isn't good, because Windows (and probably all other OS') don't like colons in file names,
    // So let's remove the colons and replace them with underscores
    let colon_matcher = regex::Regex::new(r"\:\:").unwrap();

    let test_name = &colon_matcher.replace(test_name, "_").to_string();

    // Make sure the tests are stored in the right directory
    let test_name: String = "./tests/".to_string() + test_name.as_str();

    medusa_lang::compile_from_text(source_text, &test_name).unwrap();

    // File is available at test.exe
    let output = std::process::Command::new(format!("./{test_name}.exe"))
        .output()
        .unwrap();

    // Destroy the .exe, .asm, .obj, and .lst now that we're done with them
    std::fs::remove_file(format!("{test_name}.exe")).unwrap();
    std::fs::remove_file(format!("{test_name}.asm")).unwrap();
    std::fs::remove_file(format!("{test_name}.obj")).unwrap();
    std::fs::remove_file(format!("{test_name}.lst")).unwrap();

    match output.status.code().unwrap() {
        0 => String::from_utf8(output.stdout).unwrap(),
        _ => {
            panic!("Failed to execute test")
        }
    }
}

// Retrieve only the program-specific output from a compiled medusa program, with all headers
//  and footers stripped (i.e. Medusa 1.0 and Program ended are removed) and all formatting
//  characters stripped (i.e. \n and \0)
pub fn compile_and_get_stripped_output(source_text: &str, test_name: &str) -> String {
    let output = compile_and_get_output(source_text, test_name);

    let medusa_version = env!("CARGO_PKG_VERSION");
    let header = regex::Regex::new(format!("Medusa {medusa_version}").as_str()).unwrap();
    let footer = regex::Regex::new(r"Program ended").unwrap();
    let formatting_characters = regex::Regex::new(r"(\n|\u{0})").unwrap();

    let output = header.replace(output.as_str(), "").to_string();
    let output = footer.replace(output.as_str(), "").to_string();
    let output = formatting_characters
        .replace_all(output.as_str(), "")
        .to_string();

    return output;
}
