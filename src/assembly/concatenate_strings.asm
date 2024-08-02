; ======================================
; concatenate_strings.asm
; Concatenate the two strings on top of the stack and return the new, concatenated string using RAX
; ======================================

; Pop the left string into RAX and the right string into RBX
pop rbx
pop rax

; ======================================
; Find the length of the left string so that we know where to start copying
; ======================================

; Use RCX to hold the left string's length
mov rcx, -1

; Use RDX to hold the character from the string
xor rdx, rdx

label_{string_len_loop}:
inc rcx
mov dl, [rax + rcx]

; If we haven't reached the null terminator, then keep looping
cmp dl, 0
jnz label_{string_len_loop}

; ======================================
; Copy the right string to the left string, and hope we don't overflow
; ======================================

; Use R8 to hold the right string's length
xor r8, r8

label_{string_copy_loop}:
mov dl, [rbx + r8]
inc r8

; Copy the character from the right string to the left string
mov [rax + rcx], dl
inc rcx

; If the character wasn't the null terminator, then keep looping
cmp dl, 0
jnz label_{string_copy_loop}


; Push the new, concatenated string
push rax

; ======================================
; End of concatenate_strings.asm
; ======================================
