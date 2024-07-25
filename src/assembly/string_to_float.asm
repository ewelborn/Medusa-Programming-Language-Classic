; Store the string in RAX
pop rax

; The output floating-point number will be stored in xmm1

; The quotient will be stored in rbx
; The fraction will be stored in rdx
xor rbx, rbx
xor rdx, rdx

; The loop counter (also our index into the string) will be stored in rcx
xor rcx, rcx

; Store input[i] in r8
xor r8, r8
mov r8b, [rax + rcx]

; Read the quotient
; for (; input[i]; i++)
label_{quotient_loop_index}:

; if (input[i] == '.') {{
;   i++;
;   break;
; }}
cmp r8b, 46 ; ASCII code for '.'
jne label_{quotient_loop_not_break_index}
inc rcx
jmp label_{quotient_loop_break_index}

label_{quotient_loop_not_break_index}:

; quotient *= 10;
; quotient += input[i] - '0';
imul rbx, 10
add rbx, r8
sub rbx, 48 ; ASCII code for '0'

inc rcx
mov r8b, [rax + rcx]
cmp r8b, 0
jne label_{quotient_loop_index}

label_{quotient_loop_break_index}:

; j = i, where j is stored in r9
mov r9, rcx

; Read the fractional component
; for (; input[i]; i++)
label_{fraction_loop_index}:

; fraction *= 10;
; fraction += input[i] - '0';
imul rdx, 10
add rdx, r8
sub rdx, 48 ; ASCII code for '0'

inc rcx
mov r8b, [rax + rcx]
cmp r8b, 0
jne label_{fraction_loop_index}

; Now we have both the quotient and the fraction stored as integers
; in rbx and rdx respectively

; Move rdx into our float and divide it so that it's past the decimal point
cvtsi2sd xmm1, rbx

mov r10, 10
cvtsi2sd xmm2, r10

; for (; j < i; j++)
label_{division_loop_index}:

;divpd xmm1, xmm2

inc r9
cmp r9, rcx
jl label_{division_loop_index}

; Add the quotient to the float, and we're done!
;cvtsi2sd xmm2, rbx
;addpd xmm1, xmm2

; Now that the string has finally been converted, free the string...


; ... and push the float to the stack
movq rax, xmm1
push rax