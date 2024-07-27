; ======================================
; string_to_int.asm
; Convert the string (represented internally as a null-terminated string) stored in RAX into an int (represented internally as a 64-bit signed integer) 
; and return the int using RAX
; ======================================

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

; ======================================
; End of string_to_int.asm
; ======================================
