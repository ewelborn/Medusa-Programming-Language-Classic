; ======================================
; input.asm
; Prompts the user to type in a string, strips any newline and carriage return characters, and pushes the string onto
; the stack so that it can be converted, if necessary, and stored into a variable.
; ======================================

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

; ======================================
; End of input.asm
; ======================================
