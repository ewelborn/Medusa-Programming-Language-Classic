; ======================================
; compute_integer_power.asm
; Pop integers x and y (represented internally as 64-bit signed integers) off the stack, compute the result
; of x^y, and return the result using RAX
; ======================================

; Pop the left integer (x) into rax, and pop the right integer (y) into rbx
pop rbx
pop rax

; If y is negative, don't bother calculating the power
mov rcx, 0
cmp rbx, 0
jl label_{skip_power}

; ======================================
; Exponentation by squaring
; https://stackoverflow.com/a/101613
; ======================================

; Store the result in rcx
mov rcx, 1

label_{square_loop}:

; Is y odd?
mov rdx, 1
and rdx, rbx

cmp rdx, 1
jne label_{y_is_even}

imul rcx, rax

label_{y_is_even}:

; Divide y by two. If it's zero, then we're done.
shr rbx, 1
jz label_{square_loop_break}

imul rax, rax
jmp label_{square_loop}

label_{square_loop_break}:

; ======================================
; Return the result
; ======================================

label_{skip_power}:

; Push the result
push rcx

; ======================================
; End of compute_integer_power.asm
; ======================================
