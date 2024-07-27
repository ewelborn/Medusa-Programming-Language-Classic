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

; Restore the string
mov rax, r12

; Loop through the string and see if we find a carriage return or newline character - if we do,
; then replace it with a null terminator.

; RCX will be our index into the string
xor rcx, rcx

; R8 will be the current character
xor r8, r8

label_{string_loop}:

mov r8b, [rax + rcx]

; Is the character a null terminator? Time to stop.
cmp r8, 0
je label_{string_break}

; Is the character a carriage return or newline character? If so, replace it and stop looping.
cmp r8, 13 ; ASCII code for carriage return
je label_{replace_and_break}
cmp r8, 10 ; ASCII code for newline character
je label_{replace_and_break}

inc rcx
jmp label_{string_loop}

label_{replace_and_break}:
mov byte [rax + rcx], 0

label_{string_break}:

; Push the string to the stack
push rax

; ======================================
; End of input.asm
; ======================================
