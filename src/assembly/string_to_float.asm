; ======================================
; string_to_float.asm
; Convert the string (represented internally as a null-terminated string) stored in RAX into a float (represented internally as a double)
; and return the float using RAX
; ======================================

; Store the string in RAX
pop rax

; The quotient will be stored in rbx
; The fraction will be stored in rdx
xor rbx, rbx
xor rdx, rdx

; The output floating-point number will be stored in xmm1
cvtsi2sd xmm1, rbx

; The loop counter (also our index into the string) will be stored in rcx
xor rcx, rcx

; Store input[i] in r8
xor r8, r8

; Store the negative flag in r14 (0 if the float is positive, 1 if the float is negative)
xor r14, r14

; ======================================
; Read the quotient
; ======================================

; for (; input[i]; i++)
label_{quotient_loop_index}:

mov r8b, [rax + rcx]
inc rcx

; If this is the first character, and it's a minus sign, then set the negative flag
cmp rcx, 1
jne label_{no_minus_sign}
cmp r8b, '-'
jne label_{no_minus_sign}

; It is a minus sign! Loop again.
inc r14
jmp label_{quotient_loop_index}

label_{no_minus_sign}:

; If we reached the end of the string, stop parsing the string

cmp r8b, 0
je label_{start_processing_quotient_and_fraction}

; If we reached the decimal point, break the loop and move on

cmp r8b, 46 ; ASCII code for '.'
je label_{quotient_loop_break_index}

; If the string is not a valid float, then we need to cancel the conversion process
; if (input[i] < '0' || input[i] > '9') {{
;   goto bad_input;
; }}

cmp r8b, '0'
jl label_{bad_input}
cmp r8b, '9'
jg label_{bad_input}

; quotient *= 10;
; quotient += input[i] - '0';
imul rbx, 10
add rbx, r8
sub rbx, 48 ; ASCII code for '0'

jmp label_{quotient_loop_index}

label_{quotient_loop_break_index}:

; ======================================
; Read the fraction
; If we reach this point, we know that the quotient and the decimal point were read successfully
; ======================================

; j = i, where j is stored in r9
mov r9, rcx
inc r9

; Read the fractional component
; for (; input[i]; i++)
label_{fraction_loop_index}:

; Load in the next number from the fraction
mov r8b, [rax + rcx]
inc rcx

; Break the loop if we reached the end of the string (the null terminator)
cmp r8b, 0
je label_{start_processing_quotient_and_fraction}

; If the string is not a valid float, then we need to cancel the conversion process...
; if (input[i] < '0' || input[i] > '9') {{
;   goto bad_input;
; }}

cmp r8b, '0'
jl label_{bad_input}
cmp r8b, '9'
jg label_{bad_input}

; fraction *= 10;
; fraction += input[i] - '0';
imul rdx, 10
add rdx, r8
sub rdx, 48 ; ASCII code for '0'

jmp label_{fraction_loop_index}

; ======================================
; Process the quotient and float
; Now we have both the quotient and the fraction stored as integers in rbx and rdx respectively
; If we reach this point, we know that the entire string was parsed successfully
; ======================================

label_{start_processing_quotient_and_fraction}:

; Move rdx into our float and divide it so that it's past the decimal point
cvtsi2sd xmm1, rdx

mov r10, 10
cvtsi2sd xmm2, r10

; for (; j < i; j++)
label_{division_loop_index}:

divpd xmm1, xmm2

inc r9
cmp r9, rcx
jl label_{division_loop_index}

; Add the quotient to the float, and we're done!
cvtsi2sd xmm2, rbx
addpd xmm1, xmm2

; ... Well, almost! Check and see if the float is supposed to be negative
cmp r14, 1
jne label_{end}

; Set the sign bit
mov r14, 0x8000000000000000
movq rbx, xmm1
or rbx, r14
movq xmm1, rbx

; We did not run into any issues, so we'll jump over the bad_input label
jmp label_{end}

; ======================================
; Return the float
; If we reach this point, then the string was either converted successfully, or we're returning 0.0
; ======================================

label_{bad_input}:
; The string is not a valid float, so we will return 0.0
xor rbx, rbx
cvtsi2sd xmm1, rbx

label_{end}:
; Regardless if the string was a valid float or not,

; Free the string...

; ... and push the float to the stack
movq rax, xmm1
push rax

; ======================================
; End of string_to_float.asm
; ======================================
