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

    ; mov eax, [0x400000]

    ; ; 4 Parameters
    ; mov ecx, 0x3
    ; mov edx, 0x2
    ; mov esi, 0x1
    ; mov edi, 0x0

    ; Print Trapframe
    ; mov eax, 0x0
    ; int 64

    ; Yield Process
    ; mov eax, 0x2
    ; int 64

    ; Sleep Process
    ; mov eax, 0x3
    ; int 64
    
    jmp $

align 4
INIT_STRING: db "/init", 0