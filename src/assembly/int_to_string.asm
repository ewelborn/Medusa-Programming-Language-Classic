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