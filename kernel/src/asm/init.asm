global init_start
section .text.init
bits 32
init_start:
    ; System Call Number
    mov eax, 0x0

    ; 4 Parameters
    mov ecx, 0x3
    mov edx, 0x2
    mov esi, 0x1
    mov edi, 0x0

    ; User System Calls Trap Number
    int 64

end:
    jmp end