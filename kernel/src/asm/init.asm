global init_start
section .text.init
bits 32
init_start:

    ; 4 Parameters
    mov ecx, 0x3
    mov edx, 0x2
    mov esi, 0x1
    mov edi, 0x0

    ; Print Trapframe
    mov eax, 0x0
    int 64

    ; Yield Process
    mov eax, 0x2
    int 64
    
    ; Exit Process
    mov eax, 0x1
    int 64