// The following source file was heavily referenced:
// https://github.com/pest-parser/book/blob/master/examples/jlang-parser/src/main.rs

use std::process::Command;
use std::fs;
use std::collections::HashMap;

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
    FLOAT
}

struct CompilerContext {
    variable_index: u64,
    label_index: u64,
    assembly_text: String,
    assembly_data: String,
    variables: HashMap<String, VariableDataType>,
}

/// Pops a float off the stack, converts it to an int, and pushes it back onto the stack.
fn medusa_float_to_int(context: &mut CompilerContext) {
    context.assembly_text += format!("
pop rax
movq xmm1, rax
cvtsd2si rax, xmm1
push rax
").as_str();
}

/// Pops an int off the stack, converts it to a float, and pushes it back onto the stack.
fn medusa_int_to_float(context: &mut CompilerContext) {
    context.assembly_text += format!("
pop rax
cvtsi2sd xmm1, rax
movq rax, xmm1
push rax
").as_str();
}

/// Pops an int off the stack, converts it to a string, and pushes it back onto the stack.
fn medusa_int_to_string(context: &mut CompilerContext) {
    let loop_index = context.label_index;
    context.label_index += 1;

    let skip_negation = context.label_index;
    context.label_index += 1;

    let loop_index_2 = context.label_index;
    context.label_index += 1;

    let break_index = context.label_index;
    context.label_index += 1;

    context.assembly_text += format!("
; Allocate space for int-to-string on the heap
mov rcx, [rel heap_handle]
mov rdx, 12
mov r8, 256
sub rsp, 32
call HeapAlloc
add rsp, 32

; Store empty string in R8
mov r8, rax
; Grab int from the stack and store it in R9
pop r9
; Constant for division later
mov r10, 10

; Length of the string is stored in RCX
xor rcx, rcx

; If the number is negative, set r15 to 1, then make the number positive for the rest of the calculations
xor r15, r15
cmp r9, 0
jge label_{loop_index}

; The number is indeed negative
mov r15, 1
neg r9

label_{loop_index}:

; Divide the int by 10 to strip off the next digit (stored in RDX by idiv)
mov rax, r9
xor rdx, rdx
idiv r10
mov r9, rax

; Convert digit to ASCII and add to the string
add rdx, 48 ; ASCII number for '0'
mov [r8 + rcx], dl

inc rcx

; Keep stripping digits from the int until it's empty
cmp r9, 0
jnz label_{loop_index}

; Before we reverse the string, check and see if the number was negative - if it was, add a minus sign
cmp r15, 1
jne label_{skip_negation}

; Add the minus sign to the end - it will be at the beginning once we reverse the string
mov byte [r8 + rcx], 45 ; ASCII number for '-'
inc rcx

label_{skip_negation}:

; Edge case: if the int was only one digit long, we shouldn't try to reverse the string
cmp rcx, 1
jz label_{break_index}

; Prepare to reverse the string
; for (int i = 0; i < len / 2; i++)
; Where len is RCX, r10 is len / 2, and i is RAX
mov r10, rcx
sar r10, 1
mov rax, 0

mov r11, rcx
dec r11

label_{loop_index_2}:
; Swap the characters
; rbx = string[i], rdx = string[len - 1 - i]
mov bl, [r8 + rax]
mov dl, [r8 + r11]
mov [r8 + rax], dl
mov [r8 + r11], bl

inc rax
dec r11
cmp rax, r10
jnz label_{loop_index_2}

label_{break_index}:


push r8
").as_str();
}

/// Pops a string off the stack, converts it to an int, and pushes it back onto the stack.
fn medusa_string_to_int(context: &mut CompilerContext) {
    let if_true = context.label_index;
    context.label_index += 1;

    let if_false = context.label_index;
    context.label_index += 1;

    let loop_index = context.label_index;
    context.label_index += 1;

    let break_index = context.label_index;
    context.label_index += 1;

    let finished_index = context.label_index;
    context.label_index += 1;

    context.assembly_text += format!("
; Pop the string into RAX
pop rax
; Set aside RBX for the resulting int
xor rbx, rbx

; Loop counter (i)
xor rcx, rcx

; Character holder
xor rdx, rdx

; Flag set to 1 if the number is negative, 0 if positive
xor r15, r15

; if (str[0] == '+') {{ i++; }}
mov dl, [rax]
cmp rdx, 43
je label_{if_true}

; else if (str[0] == '-') {{ r15 = 1; i++; }}
cmp rdx, 45
jne label_{if_false}

mov r15, 1
inc rcx

jmp label_{if_false}

label_{if_true}:
inc rcx

label_{if_false}:

; The odd condition here is because WinApi's ReadFile function returns a carriage return and newline character when
; we type something in on the console, which we want to ignore.
; for (; str[i] > 32; i++)
label_{loop_index}:

; Store the current character of the string in dl
mov dl, [rax + rcx]

cmp rdx, 32
jl label_{break_index}

; Multiply the result by 10 by adding together result * 2 + result * 8
mov r8, rbx
sal r8, 1
sal rbx, 3
add rbx, r8

; Add the new digit to the result
sub rdx, 48
add rbx, rdx

inc rcx
jmp label_{loop_index}

label_{break_index}:

; Lastly, check if the number was supposed to be negative - if it was, negate it
cmp r15, 1
jne label_{finished_index}

neg rbx

label_{finished_index}:
; Push the resulting int to the stack
push rbx
").as_str();
}

fn medusa_parse_expression(mut pair: pest::iterators::Pair<Rule>, context: &mut CompilerContext) -> VariableDataType {
    // Convert the entire expression to postfix notation and then convert it to assembly
    // https://www.andrew.cmu.edu/course/15-200/s06/applications/ln/junk.html
    let mut pairs = pair.into_inner();

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
            },
            Rule::add | Rule::subtract | Rule::multiply | Rule::divide | Rule::modulo | Rule::cast | Rule::expression => {
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
            },
            _ => unreachable!()
        };
    }

    // All left-over operators in the stack are pushed to the output
    while let Some(pair) = stack.pop() {
        output.push(pair);
    }

    println!("Output!! {:#?}", output);
    //println!("Stack!! {:#?}", stack);

    // Generate assembly code from the postfix notation
    let mut stack = Vec::new();

    for token in output {
        println!("{:?} - {:?}", token, stack);

        // If the current token is a variable, load it on the stack
        match token.as_rule() {
            Rule::int => {
                context.assembly_text += format!("mov rax, {}\npush rax\n", token.as_span().as_str()).as_str();
                stack.push(VariableDataType::INT);
            },
            Rule::float => {
                context.assembly_text += format!("mov rax, __float64__({})\npush rax\n", token.as_span().as_str()).as_str();
                stack.push(VariableDataType::FLOAT);
            },
            Rule::string => {
                // Copy the string into a variable in the data section
                let string_index = context.variable_index;
                context.variable_index += 1;

                let loop_index = context.label_index;
                context.label_index += 1;

                let break_index = context.label_index;
                context.label_index += 1;

                let raw_string = token.as_span().as_str();
                context.assembly_data += format!("string_{string_index} db {raw_string}, 0\n").as_str();

                // TODO: Fix memory leak - HeapAlloc for the string is never free'd

                context.assembly_text += format!("
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
", token.as_span().as_str()).as_str();
                stack.push(VariableDataType::STRING);
                //todo!();
            },
            Rule::expression => {
                stack.push(medusa_parse_expression(token, context));
                continue;
            },
            Rule::identifier => {
                match context.variables.get(token.as_span().as_str()) {
                    Some(variable) => {
                        let identifier = token.as_span().as_str();
                        match variable {
                            VariableDataType::INT => {
                                context.assembly_text += format!("mov rax, [rel var_{identifier}]\npush rax\n").as_str();
                                stack.push(VariableDataType::INT);
                            },
                            VariableDataType::FLOAT => {
                                context.assembly_text += format!("mov rax, [rel var_{identifier}]\npush rax\n").as_str();
                                stack.push(VariableDataType::FLOAT);
                            },
                            VariableDataType::STRING => {
                                context.assembly_text += format!("lea rax, [rel var_{identifier}]\npush rax\n").as_str();
                                stack.push(VariableDataType::STRING);
                            },
                        }
                    },
                    None => panic!("Variable used but not declared"),
                }
            },
            _ => {}
        }

        // If the current token is an operation, make sure the semantic rules are not being violated (i.e. correct datatypes on all variables)
        match token.as_rule() {
            // Mathematical binary operations
            Rule::add | Rule::subtract | Rule::multiply | Rule::divide | Rule::modulo | Rule::power => {
                //println!("{:?}", stack);
                let left_operand = stack[stack.len() - 2];
                let right_operand = stack[stack.len() - 1];

                if left_operand != right_operand {
                    panic!("Datatypes don't match");
                } else if 
                (left_operand != VariableDataType::INT && left_operand != VariableDataType::FLOAT) ||
                (right_operand != VariableDataType::INT && right_operand != VariableDataType::FLOAT) {
                    panic!("Math operation on non-number");
                }
            },
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
                    todo!();
                }

                stack.push(datatype);
            },
            Rule::subtract => {
                // Pop the top two numbers off the stack
                stack.pop();
                let datatype = stack.pop().unwrap();

                if datatype == VariableDataType::INT {
                    context.assembly_text += "pop rbx\npop rax\nsub rax, rbx\npush rax\n";
                } else if datatype == VariableDataType::FLOAT {
                    todo!();
                }

                stack.push(datatype);
            },
            Rule::multiply => {
                // Pop the top two numbers off the stack
                stack.pop();
                let datatype = stack.pop().unwrap();

                if datatype == VariableDataType::INT {
                    context.assembly_text += "pop rbx\npop rax\nxor rdx, rdx\nimul rbx\npush rax\n";
                } else if datatype == VariableDataType::FLOAT {
                    todo!();
                }

                stack.push(datatype);
            },
            Rule::divide => {
                // Pop the top two numbers off the stack
                stack.pop();
                let datatype = stack.pop().unwrap();

                if datatype == VariableDataType::INT {
                    context.assembly_text += "pop rbx\npop rax\nxor rdx, rdx\nidiv rbx\npush rax\n";
                } else if datatype == VariableDataType::FLOAT {
                    todo!();
                }

                stack.push(datatype);
            },
            Rule::modulo => {
                // Pop the top two numbers off the stack
                stack.pop();
                let datatype = stack.pop().unwrap();

                if datatype == VariableDataType::INT {
                    context.assembly_text += "pop rbx\npop rax\nxor rdx, rdx\nidiv rbx\npush rdx\n";
                } else if datatype == VariableDataType::FLOAT {
                    todo!();
                }

                stack.push(datatype);
            },
            Rule::cast => 'cast: {
                let from_datatype = stack.pop().unwrap();
                let to_datatype = match token.as_span().as_str() {
                    "(int)" => VariableDataType::INT,
                    "(float)" => VariableDataType::FLOAT,
                    "(string)" => VariableDataType::STRING,
                    _ => unreachable!()
                };

                if from_datatype == to_datatype {
                    // This is useless code - no conversion necessary
                    break 'cast;
                }

                match from_datatype {
                    VariableDataType::INT => {
                        match to_datatype {
                            VariableDataType::FLOAT => {
                                medusa_int_to_float(context);
                            }
                            VariableDataType::STRING => {
                                medusa_int_to_string(context);
                            },
                            _ => unreachable!()
                        }
                    },
                    VariableDataType::FLOAT => {
                        match to_datatype {
                            VariableDataType::INT => {
                                medusa_float_to_int(context);
                            }
                            VariableDataType::STRING => {
                                todo!();
                            },
                            _ => unreachable!()
                        }
                    },
                    VariableDataType::STRING => {
                        match to_datatype {
                            VariableDataType::INT => {
                                todo!();
                            }
                            VariableDataType::FLOAT => {
                                todo!();
                            },
                            _ => unreachable!()
                        }
                    }
                }

                stack.push(to_datatype);
            }
            _ => {}
        }
    }

    return stack.pop().unwrap();
}

fn medusa_parse_declaration(mut pair: pest::iterators::Pair<Rule>, context: &mut CompilerContext) {
    let mut pairs = pair.into_inner();
    
    let datatype = match pairs.next().unwrap().as_span().as_str() {
        "int" => VariableDataType::INT,
        "string" => VariableDataType::STRING,
        "float" => VariableDataType::FLOAT,
        _ => unreachable!()
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

        let name = pair.clone().into_inner().peek().unwrap().as_span().as_str().to_string();

        context.variables.insert(name.clone(), datatype);

        medusa_parse_assignment(pair, context);
    } else {
        // This is option 3

        // Clone the pair and peek into it so we can grab the identifier of the variable from the input statement
        // before evaluating the input statement.

        let name = pair.clone().into_inner().peek().unwrap().as_span().as_str().to_string();

        context.variables.insert(name.clone(), datatype);

        medusa_parse_input(pair, context);
    }
}

fn medusa_parse_assignment(mut pair: pest::iterators::Pair<Rule>, context: &mut CompilerContext) {
    let mut pairs = pair.into_inner();

    let name = pairs.next().unwrap().as_span().as_str().to_string();

    let datatype = match context.variables.get(&name) {
        Some(x) => *x,
        None => panic!("Variable does not exist for input")
    };

    if let Some(expression) = pairs.next() {
        if medusa_parse_expression(expression, context) != datatype {
            panic!("Mismatched datatypes on declaration")
        };

        match datatype {
            VariableDataType::INT | VariableDataType::FLOAT => {
                context.assembly_text += format!("pop qword [rel var_{}]\n", name).as_str();
            },
            VariableDataType::STRING => {
                let loop_index = context.label_index;
                context.label_index += 1;

                let break_index = context.label_index;
                context.label_index += 1;

                context.assembly_text += format!("
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
").as_str();
            }
        }
    };
}

fn medusa_parse_output(mut pair: pest::iterators::Pair<Rule>, context: &mut CompilerContext) {
    let expression_pair = pair.into_inner().next().unwrap();

    let datatype = medusa_parse_expression(expression_pair, context);
    
    /*if datatype != VariableDataType::STRING {
        panic!("Expected string for output");
    }*/

    // If the datatype isn't a string, convert it to a string
    match datatype {
        VariableDataType::INT => {
            medusa_int_to_string(context);
        },
        VariableDataType::FLOAT => {
            todo!();
        },
        VariableDataType::STRING => {
            // Do nothing
        },
    }

    let loop_index = context.label_index;
    context.label_index += 1;

    let break_index = context.label_index;
    context.label_index += 1;

    context.assembly_text += format!("
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
").as_str();
}

fn medusa_parse_input(mut pair: pest::iterators::Pair<Rule>, context: &mut CompilerContext) {
    let identifier = pair.into_inner().next().unwrap().as_span().as_str();

    let datatype = match context.variables.get(identifier) {
        Some(x) => x,
        None => panic!("Variable does not exist for input")
    };

    context.assembly_text += format!("
; Allocate a string to hold the input
mov rcx, [rel heap_handle]
mov rdx, 12
mov r8, 256
sub rsp, 32
call HeapAlloc
add rsp, 32

; String pointer stored in RAX assuming no errors - back it up in R12
mov r12, rax

; Read the input from the user
mov rcx, [rel input_handle]
mov rdx, rax
mov r8, 255
lea r9, [rel ignore]
sub rsp, 48
call ReadFile
add rsp, 48

; Push the string to the stack
push r12
").as_str();

    // If the datatype of our variable isn't a string, convert the input accordingly before storing
    match datatype {
        VariableDataType::INT => {
            medusa_string_to_int(context);
            context.assembly_text += format!("
; Store the input into the variable
pop qword [rel var_{identifier}]
").as_str();
        },
        VariableDataType::FLOAT => {
            todo!();
        },
        VariableDataType::STRING => {
            // Copy the string into the variable
            let loop_index = context.label_index;
            context.label_index += 1;

            let break_index = context.label_index;
            context.label_index += 1;

            context.assembly_text += format!("
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
").as_str();
        },
    }

    // Store the input into the variable !!!NO!!!
    /*context.assembly_text += format!("
; Store the input into the variable
mov [rel var_{identifier}], rax
").as_str();*/

}

fn medusa_parse_statement(mut pair: pest::iterators::Pair<Rule>, context: &mut CompilerContext) {
    match pair.as_rule() {
        Rule::declaration => {
            medusa_parse_declaration(pair, context);
        },
        Rule::assignment => {
            medusa_parse_assignment(pair, context);
        },
        Rule::output => {
            medusa_parse_output(pair, context);
        }
        Rule::input => {
            medusa_parse_input(pair, context);
        }
        Rule::EOI => {},
        _ => {
            panic!("Unexpected rule!");
        }
    };
}

fn main() {
    let source_text = std::fs::read_to_string("input.med")
        .expect("Could not read input.med");

    //let abstract_syntax_tree = parse(&source_text).expect("Could not parse the source text");

    let parse_result = MedusaParser::parse(Rule::program, &source_text);

    let file = match parse_result {
        Err(e) => {
            println!("{:#?}", e);
            panic!("AHH");
        },
        Ok(v) => v
    };
        //.expect("Could not parse the source text");

    //println!("{:#?}", file);

    /*variables.insert("s".to_string(), VariableDataType::STRING);
    variables.insert("x".to_string(), VariableDataType::INT);
    variables.insert("q".to_string(), VariableDataType::FLOAT);*/

    /*let assembly_text = "
    mov rcx, 0
    lea rdx, [rel message]
    mov r8, 0
    mov r9, 0
    call MessageBoxA
    ".to_string();
    let mut assembly_data = "message: db \"Hello, world! :)\", 0\n".to_string();*/

    let mut context = CompilerContext {
        variable_index: 0,
        label_index: 0,
        assembly_text: String::new(),
        assembly_data: String::new(),
        variables: HashMap::new(),
    };

    for pair in file {
        medusa_parse_statement(pair, &mut context);
    };

    for variable in context.variables {
        context.assembly_data += format!(
            "var_{}: {}\n", 
            variable.0.as_str(), 
            match variable.1 {
                VariableDataType::INT => "dq 0",
                VariableDataType::FLOAT => "dq 0",
                VariableDataType::STRING => "resb 1000"
            }
        ).as_str();
    }

    let assembly_text = context.assembly_text;
    let assembly_data = context.assembly_data;

    let assembly_source = format!("
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
buffer_string resb 1024");

    fs::write("medusa_output.asm", assembly_source).expect("Could not write assembly source file");

    let assembler_output = Command::new("./windows/nasm.exe")
        .args(["-f win64", "medusa_output.asm", "-o medusa_output.obj", "-l medusa_output.lst"])
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
        },
        _ => {
            panic!("Assembler failed: {}", String::from_utf8(assembler_output.stderr).unwrap());
        }
    }

    let linker_output = Command::new("./windows/ld.lld.exe")
        .args(["medusa_output.obj", "-omedusa_output.exe", "C:/Windows/System32/user32.dll", "C:/Program Files (x86)/Windows Kits/10/Lib/10.0.22000.0/um/x64/kernel32.lib"])
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
        },
        _ => {
            panic!("Linker failed: {}", String::from_utf8(linker_output.stderr).unwrap());
        }
    }
}
