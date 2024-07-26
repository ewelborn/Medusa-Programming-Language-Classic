; ======================================
; int_to_float.asm
; Convert the int (represented internally as a 64-bit signed integer) stored in RAX into a float (represented internally as a double)
; and return the float using RAX
; ======================================

pop rax
cvtsi2sd xmm1, rax
movq rax, xmm1
push rax

; ======================================
; End of int_to_float.asm
; ======================================
