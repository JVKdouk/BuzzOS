global init_start

section .text.init
bits 32
init_start:
    ; Setup file system
    mov eax, 4
    int 64

    ; Exec User-Level Rust Init
    mov eax, 5
    mov edi, INIT_STRING
    mov edx, 5
    int 64

    jmp $

align 4
INIT_STRING: db "/init", 0