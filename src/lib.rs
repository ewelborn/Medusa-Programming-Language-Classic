// The following source file was heavily referenced:
// https://github.com/pest-parser/book/blob/master/examples/jlang-parser/src/main.rs

use std::collections::HashMap;
use std::fs;
use std::process::Command;

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "medusa.pest"]
pub struct MedusaParser;

#[derive(Debug, Clone, Copy, PartialEq)]
enum VariableDataType {
    INT,
    STRING,
    FLOAT,
}

struct CompilerContext {
    variable_index: u64,
    label_index: u64,
    assembly_text: String,
    assembly_data: String,
    variables: HashMap<String, VariableDataType>,
}

#[derive(Debug, Clone)]
pub struct CompileError {
    body: String,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.body)
    }
}

fn print_assembly_with_context(file_name: &str, context: &mut CompilerContext) {
    let source_text = std::fs::read_to_string(format!("./src/assembly/{file_name}.asm"))
        .expect(format!("Could not read assembly file {}", file_name).as_str());

    let label_matcher = regex::Regex::new(r"[^\{]\{(?<label>[^\{\}]+)\}[^\}]").unwrap();

    let mut labels_to_indices: HashMap<String, u64> = HashMap::new();

    for x in label_matcher.captures_iter(&source_text) {
        let label = x.name("label").unwrap().as_str();

        if !labels_to_indices.contains_key(label) {
            labels_to_indices.insert(label.to_string(), context.label_index);
            context.label_index += 1;
        }
    }

    let replacement = |x: &regex::Captures| -> String {
        // Return the index of the label while preserving the first and last character of the match
        // (which are not actually a part of the label, it's confusing, I know)
        let raw_capture = x.get(0).unwrap().as_str();
        raw_capture.chars().nth(0).unwrap().to_string()
            + labels_to_indices
                .get(x.name("label").unwrap().as_str())
                .unwrap()
                .to_string()
                .as_str()
            + raw_capture
                .chars()
                .nth(raw_capture.len() - 1)
                .unwrap()
                .to_string()
                .as_str()
    };

    let processed_text = label_matcher.replace_all(&source_text, &replacement);

    context.assembly_text += &processed_text;
}

/// Pops a float off the stack, converts it to an int, and pushes it back onto the stack.
fn medusa_float_to_int(context: &mut CompilerContext) {
    context.assembly_text += format!(
        "
pop rax
movq xmm1, rax
cvtsd2si rax, xmm1
push rax
"
    )
    .as_str();
}

/// Pops an int off the stack, converts it to a float, and pushes it back onto the stack.
fn medusa_int_to_float(context: &mut CompilerContext) {
    print_assembly_with_context("int_to_float", context);
}

/// Pops a float off the stack, converts it to a string, and pushes it back onto the stack.
fn medusa_float_to_string(context: &mut CompilerContext) {
    print_assembly_with_context("float_to_string", context);
}

/// Pops a string off the stack, converts it to a float, and pushes it back onto the stack.
fn medusa_string_to_float(context: &mut CompilerContext) {
    print_assembly_with_context("string_to_float", context);
}

/// Pops an int off the stack, converts it to a string, and pushes it back onto the stack.
fn medusa_int_to_string(context: &mut CompilerContext) {
    print_assembly_with_context("int_to_string", context);
}

/// Pops a string off the stack, converts it to an int, and pushes it back onto the stack.
fn medusa_string_to_int(context: &mut CompilerContext) {
    print_assembly_with_context("string_to_int", context);
}

fn medusa_parse_expression(
    pair: pest::iterators::Pair<Rule>,
    context: &mut CompilerContext,
) -> VariableDataType {
    // Convert the entire expression to postfix notation and then convert it to assembly
    // https://www.andrew.cmu.edu/course/15-200/s06/applications/ln/junk.html
    let pairs = pair.into_inner();

    let stack_precedence_map = HashMap::from([
        (Rule::EOI, 0),
        (Rule::add, 2),
        (Rule::subtract, 2),
        (Rule::multiply, 4),
        (Rule::divide, 4),
        (Rule::modulo, 4),
        (Rule::cast, 6),
        (Rule::expression, 51),
    ]);

    let input_precedence_map = HashMap::from([
        (Rule::add, 1),
        (Rule::subtract, 1),
        (Rule::multiply, 3),
        (Rule::divide, 3),
        (Rule::modulo, 3),
        (Rule::cast, 5),
        (Rule::expression, 50),
    ]);

    let mut output = Vec::new();
    let mut stack: Vec<Pair<Rule>> = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::int | Rule::float | Rule::string | Rule::identifier => {
                output.push(pair);
            }
            Rule::add
            | Rule::subtract
            | Rule::multiply
            | Rule::divide
            | Rule::modulo
            | Rule::cast
            | Rule::expression => {
                loop {
                    let stack_precedence: i32 = match stack.last() {
                        Some(x) => *stack_precedence_map.get(&x.as_rule()).unwrap(),
                        None => 0,
                    };

                    let input_precedence = *input_precedence_map.get(&pair.as_rule()).unwrap();

                    if stack_precedence >= input_precedence {
                        if stack.last().is_none() {
                            // No clue what this case is
                            panic!("Unexpected case");
                        } else {
                            output.push(stack.pop().unwrap());
                        }
                    } else {
                        stack.push(pair);
                        break;
                    }
                }
            }
            _ => unreachable!(),
        };
    }

    // All left-over operators in the stack are pushed to the output
    while let Some(pair) = stack.pop() {
        output.push(pair);
    }

    //println!("Output!! {:#?}", output);
    //println!("Stack!! {:#?}", stack);

    // Generate assembly code from the postfix notation
    let mut stack = Vec::new();

    for token in output {
        //println!("{:?} - {:?}", token, stack);

        // If the current token is a variable, load it on the stack
        match token.as_rule() {
            Rule::int => {
                context.assembly_text +=
                    format!("mov rax, {}\npush rax\n", token.as_span().as_str()).as_str();
                stack.push(VariableDataType::INT);
            }
            Rule::float => {
                context.assembly_text += format!(
                    "mov rax, __float64__({})\npush rax\n",
                    token.as_span().as_str()
                )
                .as_str();
                stack.push(VariableDataType::FLOAT);
            }
            Rule::string => {
                // Copy the string into a variable in the data section
                let string_index = context.variable_index;
                context.variable_index += 1;

                let loop_index = context.label_index;
                context.label_index += 1;

                let break_index = context.label_index;
                context.label_index += 1;

                let raw_string = token.as_span().as_str();
                context.assembly_data +=
                    format!("string_{string_index} db {raw_string}, 0\n").as_str();

                // TODO: Fix memory leak - HeapAlloc for the string is never free'd

                context.assembly_text += format!(
                    "
; Allocate space for string {} on the heap
mov rcx, [rel heap_handle]
mov rdx, 12
mov r8, 256
sub rsp, 32
call HeapAlloc
add rsp, 32

; Copy the string
mov rcx, 0
lea r8, [rel string_{string_index}]

label_{loop_index}:
mov rdx, [r8 + rcx]
cmp rdx, 0
jz label_{break_index}

mov [rax + rcx], rdx
inc rcx
jmp label_{loop_index}

label_{break_index}:
push rax
",
                    token.as_span().as_str()
                )
                .as_str();
                stack.push(VariableDataType::STRING);
                //todo!();
            }
            Rule::expression => {
                stack.push(medusa_parse_expression(token, context));
                continue;
            }
            Rule::identifier => match context.variables.get(token.as_span().as_str()) {
                Some(variable) => {
                    let identifier = token.as_span().as_str();
                    match variable {
                        VariableDataType::INT => {
                            context.assembly_text +=
                                format!("mov rax, [rel var_{identifier}]\npush rax\n").as_str();
                            stack.push(VariableDataType::INT);
                        }
                        VariableDataType::FLOAT => {
                            context.assembly_text +=
                                format!("mov rax, [rel var_{identifier}]\npush rax\n").as_str();
                            stack.push(VariableDataType::FLOAT);
                        }
                        VariableDataType::STRING => {
                            context.assembly_text +=
                                format!("lea rax, [rel var_{identifier}]\npush rax\n").as_str();
                            stack.push(VariableDataType::STRING);
                        }
                    }
                }
                None => panic!("Variable used but not declared"),
            },
            _ => {}
        }

        // If the current token is an operation, make sure the semantic rules are not being violated (i.e. correct datatypes on all variables)
        match token.as_rule() {
            // Mathematical binary operations
            Rule::add
            | Rule::subtract
            | Rule::multiply
            | Rule::divide
            | Rule::modulo
            | Rule::power => {
                //println!("{:?}", stack);
                let left_operand = stack[stack.len() - 2];
                let right_operand = stack[stack.len() - 1];

                if left_operand != right_operand {
                    panic!("Datatypes don't match");
                } else if (left_operand != VariableDataType::INT
                    && left_operand != VariableDataType::FLOAT)
                    || (right_operand != VariableDataType::INT
                        && right_operand != VariableDataType::FLOAT)
                {
                    panic!("Math operation on non-number");
                }
            }
            _ => {}
        }

        // If the current token is an operation, execute it in assembly
        match token.as_rule() {
            Rule::add => {
                // Pop the top two numbers off the stack
                stack.pop();
                let datatype = stack.pop().unwrap();

                if datatype == VariableDataType::INT {
                    context.assembly_text += "pop rbx\npop rax\nadd rax, rbx\npush rax\n";
                } else if datatype == VariableDataType::FLOAT {
                    context.assembly_text += "
pop rbx
movq xmm2, rbx
pop rax
movq xmm1, rax
addpd xmm1, xmm2
movq rax, xmm1
push rax
";
                }

                stack.push(datatype);
            }
            Rule::subtract => {
                // Pop the top two numbers off the stack
                stack.pop();
                let datatype = stack.pop().unwrap();

                if datatype == VariableDataType::INT {
                    context.assembly_text += "pop rbx\npop rax\nsub rax, rbx\npush rax\n";
                } else if datatype == VariableDataType::FLOAT {
                    context.assembly_text += "
pop rbx
movq xmm2, rbx
pop rax
movq xmm1, rax
subpd xmm1, xmm2
movq rax, xmm1
push rax
";
                }

                stack.push(datatype);
            }
            Rule::multiply => {
                // Pop the top two numbers off the stack
                stack.pop();
                let datatype = stack.pop().unwrap();

                if datatype == VariableDataType::INT {
                    context.assembly_text += "pop rbx\npop rax\nxor rdx, rdx\nimul rbx\npush rax\n";
                } else if datatype == VariableDataType::FLOAT {
                    context.assembly_text += "
pop rbx
movq xmm2, rbx
pop rax
movq xmm1, rax
mulpd xmm1, xmm2
movq rax, xmm1
push rax
";
                }

                stack.push(datatype);
            }
            Rule::divide => {
                // Pop the top two numbers off the stack
                stack.pop();
                let datatype = stack.pop().unwrap();

                if datatype == VariableDataType::INT {
                    context.assembly_text += "pop rbx\npop rax\nxor rdx, rdx\nidiv rbx\npush rax\n";
                } else if datatype == VariableDataType::FLOAT {
                    context.assembly_text += "
pop rbx
movq xmm2, rbx
pop rax
movq xmm1, rax
divpd xmm1, xmm2
movq rax, xmm1
push rax
";
                }

                stack.push(datatype);
            }
            Rule::modulo => {
                // Pop the top two numbers off the stack
                stack.pop();
                let datatype = stack.pop().unwrap();

                if datatype == VariableDataType::INT {
                    context.assembly_text += "pop rbx\npop rax\nxor rdx, rdx\nidiv rbx\npush rdx\n";
                } else if datatype == VariableDataType::FLOAT {
                    panic!("Modulo is not supported for floats")
                }

                stack.push(datatype);
            }
            Rule::cast => 'cast: {
                let from_datatype = stack.pop().unwrap();
                let to_datatype = match token.as_span().as_str() {
                    "(int)" => VariableDataType::INT,
                    "(float)" => VariableDataType::FLOAT,
                    "(string)" => VariableDataType::STRING,
                    _ => unreachable!(),
                };

                if from_datatype == to_datatype {
                    // This is useless code - no conversion necessary
                    break 'cast;
                }

                match from_datatype {
                    VariableDataType::INT => match to_datatype {
                        VariableDataType::FLOAT => {
                            medusa_int_to_float(context);
                        }
                        VariableDataType::STRING => {
                            medusa_int_to_string(context);
                        }
                        _ => unreachable!(),
                    },
                    VariableDataType::FLOAT => match to_datatype {
                        VariableDataType::INT => {
                            medusa_float_to_int(context);
                        }
                        VariableDataType::STRING => {
                            medusa_float_to_string(context);
                        }
                        _ => unreachable!(),
                    },
                    VariableDataType::STRING => match to_datatype {
                        VariableDataType::INT => {
                            medusa_string_to_int(context);
                        }
                        VariableDataType::FLOAT => {
                            medusa_string_to_float(context);
                        }
                        _ => unreachable!(),
                    },
                }

                stack.push(to_datatype);
            }
            _ => {}
        }
    }

    return stack.pop().unwrap();
}

fn medusa_parse_declaration(pair: pest::iterators::Pair<Rule>, context: &mut CompilerContext) {
    let mut pairs = pair.into_inner();

    let datatype = match pairs.next().unwrap().as_span().as_str() {
        "int" => VariableDataType::INT,
        "string" => VariableDataType::STRING,
        "float" => VariableDataType::FLOAT,
        _ => unreachable!(),
    };

    // There are three options for a declaration:
    //  1. Datatype and identifier
    //  2. Datatype and assignment
    //  3. Datatype and input statement
    // We need to figure out which option is being used

    let pair = pairs.next().unwrap();

    if pair.as_rule() == Rule::identifier {
        // This is option 1
        let name = pair.as_span().as_str().to_string();

        context.variables.insert(name.clone(), datatype);
    } else if pair.as_rule() == Rule::assignment {
        // This is option 2

        // Clone the pair and peek into it so we can grab the identifier of the variable from the input statement
        // before evaluating the input statement.

        let name = pair
            .clone()
            .into_inner()
            .peek()
            .unwrap()
            .as_span()
            .as_str()
            .to_string();

        context.variables.insert(name.clone(), datatype);

        medusa_parse_assignment(pair, context);
    } else {
        // This is option 3

        // Clone the pair and peek into it so we can grab the identifier of the variable from the input statement
        // before evaluating the input statement.

        let name = pair
            .clone()
            .into_inner()
            .peek()
            .unwrap()
            .as_span()
            .as_str()
            .to_string();

        context.variables.insert(name.clone(), datatype);

        medusa_parse_input(pair, context);
    }
}

fn medusa_parse_assignment(pair: pest::iterators::Pair<Rule>, context: &mut CompilerContext) {
    let mut pairs = pair.into_inner();

    let name = pairs.next().unwrap().as_span().as_str().to_string();

    let datatype = match context.variables.get(&name) {
        Some(x) => *x,
        None => panic!("Variable does not exist for input"),
    };

    if let Some(expression) = pairs.next() {
        if medusa_parse_expression(expression, context) != datatype {
            panic!("Mismatched datatypes on declaration")
        };

        match datatype {
            VariableDataType::INT | VariableDataType::FLOAT => {
                context.assembly_text += format!("pop qword [rel var_{}]\n", name).as_str();
            }
            VariableDataType::STRING => {
                let loop_index = context.label_index;
                context.label_index += 1;

                let break_index = context.label_index;
                context.label_index += 1;

                context.assembly_text += format!(
                    "
; Copy the string
mov rcx, 0
pop r8
lea rax, [rel var_{name}]

label_{loop_index}:
mov dl, [r8 + rcx]
cmp dl, 0
jz label_{break_index}

mov [rax + rcx], dl
inc rcx
jmp label_{loop_index}

label_{break_index}:
"
                )
                .as_str();
            }
        }
    };
}

fn medusa_parse_output(pair: pest::iterators::Pair<Rule>, context: &mut CompilerContext) {
    let expression_pair = pair.into_inner().next().unwrap();

    let datatype = medusa_parse_expression(expression_pair, context);

    /*if datatype != VariableDataType::STRING {
        panic!("Expected string for output");
    }*/

    // If the datatype isn't a string, convert it to a string
    match datatype {
        VariableDataType::INT => {
            medusa_int_to_string(context);
        }
        VariableDataType::FLOAT => {
            medusa_float_to_string(context);
        }
        VariableDataType::STRING => {
            // Do nothing
        }
    }

    let loop_index = context.label_index;
    context.label_index += 1;

    let break_index = context.label_index;
    context.label_index += 1;

    context.assembly_text += format!(
        "
; find string length
pop rdx
xor r8, r8

label_{loop_index}:
mov al, [rdx + r8]
cmp al, 0
jz label_{break_index}
inc r8
jmp label_{loop_index}

label_{break_index}:

; print string
sub rsp, 16
mov rcx, [rel output_handle]
mov r9, ignore
call WriteFile

mov rcx, [rel output_handle]
lea rdx, [rel newline]
mov r8, 1
mov r9, ignore
call WriteFile
add rsp, 16
"
    )
    .as_str();
}

fn medusa_parse_input(pair: pest::iterators::Pair<Rule>, context: &mut CompilerContext) {
    let identifier = pair.into_inner().next().unwrap().as_span().as_str();

    print_assembly_with_context("input", context);

    let datatype = match context.variables.get(identifier) {
        Some(x) => x,
        None => panic!("Variable does not exist for input"),
    };

    // If the datatype of our variable isn't a string, convert the input accordingly before storing
    match datatype {
        VariableDataType::INT => {
            medusa_string_to_int(context);
            context.assembly_text += format!(
                "
; Store the input into the variable
pop qword [rel var_{identifier}]
"
            )
            .as_str();
        }
        VariableDataType::FLOAT => {
            medusa_string_to_float(context);
            context.assembly_text += format!(
                "
; Store the input into the variable
pop qword [rel var_{identifier}]
"
            )
            .as_str();
        }
        VariableDataType::STRING => {
            // Copy the string into the variable
            let loop_index = context.label_index;
            context.label_index += 1;

            let break_index = context.label_index;
            context.label_index += 1;

            context.assembly_text += format!(
                "
; Copy the string from the stack into our variable
pop rax
mov rcx, 0
lea rbx, [rel var_{identifier}]

label_{loop_index}:
mov dl, [rax + rcx]
cmp dl, 0
jz label_{break_index}

mov [rbx + rcx], dl
inc rcx
jmp label_{loop_index}

label_{break_index}:
"
            )
            .as_str();
        }
    }

    // Store the input into the variable !!!NO!!!
    /*context.assembly_text += format!("
    ; Store the input into the variable
    mov [rel var_{identifier}], rax
    ").as_str();*/
}

fn medusa_parse_if(pair: pest::iterators::Pair<Rule>, context: &mut CompilerContext) {
    let mut pairs = pair.into_inner();

    let left_expression_pair = pairs.next().unwrap();
    let conditional_operator_pair = pairs.next().unwrap();
    let right_expression_pair = pairs.next().unwrap();

    let left_datatype = medusa_parse_expression(left_expression_pair, context);
    let right_datatype = medusa_parse_expression(right_expression_pair, context);

    if left_datatype != right_datatype {
        panic!("Datatypes in if statement do not match!");
    }

    // The result of the right expression is on top of the stack, and the result of the left expression is just below it
    // Let's store left in RAX and right in RBX
    context.assembly_text += "pop rbx\npop rax\n";

    let conditional_operator = conditional_operator_pair.as_span().as_str();

    let skip_if_statement_label = context.label_index;
    context.label_index += 1;

    match conditional_operator {
        ">" => match left_datatype {
            VariableDataType::INT => {
                context.assembly_text +=
                    &format!("cmp rax, rbx\njle label_{skip_if_statement_label}\n")
            }
            VariableDataType::FLOAT => {
                todo!()
            }
            VariableDataType::STRING => {
                panic!("Cannot compare strings with >");
            }
        },
        "<" => match left_datatype {
            VariableDataType::INT => {
                context.assembly_text +=
                    &format!("cmp rax, rbx\njge label_{skip_if_statement_label}\n")
            }
            VariableDataType::FLOAT => {
                todo!()
            }
            VariableDataType::STRING => {
                panic!("Cannot compare strings with <");
            }
        },
        ">=" => match left_datatype {
            VariableDataType::INT => {
                context.assembly_text +=
                    &format!("cmp rax, rbx\njl label_{skip_if_statement_label}\n")
            }
            VariableDataType::FLOAT => {
                todo!()
            }
            VariableDataType::STRING => {
                panic!("Cannot compare strings with >=");
            }
        },
        "<=" => match left_datatype {
            VariableDataType::INT => {
                context.assembly_text +=
                    &format!("cmp rax, rbx\njg label_{skip_if_statement_label}\n")
            }
            VariableDataType::FLOAT => {
                todo!()
            }
            VariableDataType::STRING => {
                panic!("Cannot compare strings with <=");
            }
        },
        "==" => match left_datatype {
            VariableDataType::INT => {
                context.assembly_text +=
                    &format!("cmp rax, rbx\njne label_{skip_if_statement_label}\n")
            }
            VariableDataType::FLOAT => {
                todo!()
            }
            VariableDataType::STRING => {
                todo!()
            }
        },
        "!=" => match left_datatype {
            VariableDataType::INT => {
                context.assembly_text +=
                    &format!("cmp rax, rbx\nje label_{skip_if_statement_label}\n")
            }
            VariableDataType::FLOAT => {
                todo!()
            }
            VariableDataType::STRING => {
                todo!()
            }
        },
        _ => {
            unreachable!();
        }
    }

    let mut else_statement: Option<Pair<Rule>> = None;

    // Print all of the code inside the if statement
    loop {
        match pairs.next() {
            Some(pair) => {
                if pair.as_rule() == Rule::else_ {
                    else_statement = Some(pair);
                    break;
                } else {
                    medusa_parse_statement(pair, context);
                }
            }
            None => {
                break;
            }
        }
    }

    match else_statement {
        Some(else_statement) => {
            // This if statement has an else statement tagging along with it

            let skip_else_statement_label = context.label_index;
            context.label_index += 1;

            // The instructions are ordered in this way so that, if the if statement is true,
            // it will hit the jump and skip all of the else statement code
            context.assembly_text += &format!("\njmp label_{skip_else_statement_label}\n");
            context.assembly_text += &format!("\nlabel_{skip_if_statement_label}:\n");

            // Print all of the code inside the else statement
            let mut else_statement_pairs = else_statement.into_inner();
            loop {
                match else_statement_pairs.next() {
                    Some(pair) => {
                        medusa_parse_statement(pair, context);
                    }
                    None => {
                        break;
                    }
                }
            }

            context.assembly_text += &format!("\nlabel_{skip_else_statement_label}:\n");
        }
        None => {
            // Just a normal if statement - emit the ending label

            context.assembly_text += &format!("\nlabel_{skip_if_statement_label}:\n");
        }
    }
}

fn medusa_parse_statement(pair: pest::iterators::Pair<Rule>, context: &mut CompilerContext) {
    match pair.as_rule() {
        Rule::declaration => {
            medusa_parse_declaration(pair, context);
        }
        Rule::assignment => {
            medusa_parse_assignment(pair, context);
        }
        Rule::output => {
            medusa_parse_output(pair, context);
        }
        Rule::input => {
            medusa_parse_input(pair, context);
        }
        Rule::if_ => {
            medusa_parse_if(pair, context);
        }
        Rule::EOI => {}
        _ => {
            panic!("Unexpected rule!");
        }
    };
}

pub fn compile_from_text(source_text: &str, output_file_name: &str) -> Result<(), CompileError> {
    let parse_result = MedusaParser::parse(Rule::program, &source_text);

    let file = match parse_result {
        Err(e) => {
            return Err(CompileError {
                body: format!("{:#?}", e),
            });
        }
        Ok(v) => v,
    };

    let mut context = CompilerContext {
        variable_index: 0,
        label_index: 0,
        assembly_text: String::new(),
        assembly_data: String::new(),
        variables: HashMap::new(),
    };

    for pair in file {
        medusa_parse_statement(pair, &mut context);
    }

    for variable in context.variables {
        context.assembly_data += format!(
            "var_{}: {}\n",
            variable.0.as_str(),
            match variable.1 {
                VariableDataType::INT => "dq 0",
                VariableDataType::FLOAT => "dq 0",
                VariableDataType::STRING => "resb 1000",
            }
        )
        .as_str();
    }

    let assembly_text = context.assembly_text;
    let assembly_data = context.assembly_data;

    let assembly_source = format!(
        "
bits 64

global mainCRTStartup

extern MessageBoxA
extern WriteFile
extern ReadFile
extern GetStdHandle
extern ExitProcess
extern GetProcessHeap
extern HeapAlloc

section .text
mainCRTStartup:

sub rsp, 40

mov rcx, -10
call GetStdHandle
mov [rel input_handle], rax

mov rcx, -11
call GetStdHandle
mov [rel output_handle], rax

call GetProcessHeap
mov [rel heap_handle], rax

sub rsp, 16
mov rcx, [rel output_handle]
mov rdx, medusa_string
mov r8, medusa_string_length
mov r9, ignore
call WriteFile
add rsp, 16

{assembly_text}

sub rsp, 16
mov rcx, [rel output_handle]
mov rdx, program_ended_string
mov r8, program_ended_string_length
mov r9, ignore
call WriteFile
add rsp, 16

mov rcx, 0
call ExitProcess

add rsp, 40

ret

section .data
input_handle dq 0
output_handle dq 0
heap_handle dq 0
medusa_string db \"Medusa 1.0\", 10, 0
medusa_string_length equ $-medusa_string
program_ended_string db \"Program ended\", 10, 0
program_ended_string_length equ $-program_ended_string
newline db 10
ignore dq 0
{assembly_data}

section .bss
buffer_string resb 1024"
    );

    fs::write(format!("{}.asm", output_file_name), assembly_source)
        .expect("Could not write assembly source file");

    let assembler_output = Command::new("./windows/nasm.exe")
        .args([
            "-f win64",
            format!("{}.asm", output_file_name).as_str(),
            format!("-o {}.obj", output_file_name).as_str(),
            format!("-l {}.lst", output_file_name).as_str(),
        ])
        .output()
        .unwrap();

    match assembler_output.status.code().unwrap() {
        0 => {
            let error_text = String::from_utf8(assembler_output.stderr).unwrap();

            if error_text.len() > 0 {
                println!("Assembler succeeded with warnings: {}", error_text);
            } else {
                println!("Assembler succeeded");
            }
        }
        _ => {
            return Err(CompileError {
                body: format!(
                    "Assembler failed: {}",
                    String::from_utf8(assembler_output.stderr).unwrap()
                ),
            });
        }
    }

    let linker_output = Command::new("./windows/ld.lld.exe")
        .args([
            format!("{}.obj", output_file_name).as_str(),
            format!("-o{}.exe", output_file_name).as_str(),
            "C:/Windows/System32/user32.dll",
            "C:/Program Files (x86)/Windows Kits/10/Lib/10.0.22000.0/um/x64/kernel32.lib",
        ])
        .output()
        .unwrap();

    match linker_output.status.code().unwrap() {
        0 => {
            let error_text = String::from_utf8(linker_output.stderr).unwrap();

            if error_text.len() > 0 {
                println!("Linker succeeded with warnings: {}", error_text);
            } else {
                println!("Linker succeeded");
            }
        }
        _ => {
            return Err(CompileError {
                body: format!(
                    "Linker failed: {}",
                    String::from_utf8(linker_output.stderr).unwrap()
                ),
            });
        }
    }

    return Ok(());
}
