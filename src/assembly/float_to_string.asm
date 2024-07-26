; ======================================
; float_to_string.asm
; Convert the float (represented internally as a double) stored in RAX into a string (represented internally as a null-terminated string)
; and return the string using RAX
; ======================================

; Pop the float off the stack and move it into xmm1
pop rax
movq xmm1, rax

; Allocate space for float-to-string on the heap
mov rcx, [rel heap_handle]
mov rdx, 12
mov r8, 256
sub rsp, 32
call HeapAlloc
add rsp, 32

; The pointer to the string is stored in r8
mov r8, rax

; Store the quotient in rbx
cvttsd2si rbx, xmm1

; Start by building the fraction in reverse, then add the decimal dot, then build
;   the quotient in reverse, then reverse the entire string to get the final result.

; Get 6 digits past the decimal point with none of the quotient
;   and store the fractional component in rcx
;   fraction = (f - quotient) * 1000000;
cvtsi2sd xmm2, rbx
subsd xmm1, xmm2
mov r14, 1000000
cvtsi2sd xmm2, r14
mulsd xmm1, xmm2
cvtsd2si rcx, xmm1

; The index into the string is stored in r9
xor r9, r9

; for (int j = 0; j < 6; j++)
; {{
;   result[i] = '0' + fraction % 10;
;   i++;
;   fraction = fraction / 10;
; }}

; Store the constant 10 in r10
mov r10, 10

; Loop counter j stored in r11
mov r11, 0

label_{loop_index}:
; Divide the int by 10 to strip off the next digit (quotient in RAX, remainder in RDX)
mov rax, rcx
xor rdx, rdx
idiv r10

; Convert digit to ASCII and add to the string
add rdx, 48 ; ASCII number for '0'
mov [r8 + r9], dl

inc r9

mov rcx, rax

inc r11
cmp r11, 6
jl label_{loop_index}

; Add the dot to the floating-point number
mov rdx, 46 ; ASCII number for '.'
mov [r8 + r9], dl
inc r9

; while (quotient > 0)
; {{
;   result[i] = '0' + quotient % 10;
;   i++;
;   quotient = quotient / 10;
; }}

label_{loop_index_2}:
; Divide the int by 10 to strip off the next digit (quotient in RAX, remainder in RDX)
mov rax, rbx
xor rdx, rdx
idiv r10

; Convert digit to ASCII and add to the string
add rdx, 48 ; ASCII number for '0'
mov [r8 + r9], dl

inc r9

mov rbx, rax

; Keep stripping digits from the int until it's empty
cmp rbx, 0
jnz label_{loop_index_2}

; Finally, reverse the string before pushing it onto the stack

; Before we reverse the string, check and see if the number was negative - if it was, add a minus sign
cmp r15, 1
jne label_{skip_negation}

; Add the minus sign to the end - it will be at the beginning once we reverse the string
mov byte [r8 + r9], 45 ; ASCII number for '-'
inc rcx

label_{skip_negation}:

; Prepare to reverse the string
; for (int i = 0; i < len / 2; i++)
; Where len is r9, r10 is len / 2, and i is RAX
mov r10, r9
sar r10, 1
mov rax, 0

mov r11, r9
dec r11

label_{loop_index_3}:
; Swap the characters
; rbx = string[i], rdx = string[len - 1 - i]
mov bl, [r8 + rax]
mov dl, [r8 + r11]
mov [r8 + rax], dl
mov [r8 + r11], bl

inc rax
dec r11
cmp rax, r10
jnz label_{loop_index_3}

; Push the final string onto the stack
push r8

; ======================================
; End of float_to_string.asm
; ======================================
