global init_start
section .text.init
bits 32
init_start:
    mov eax, 0x171717
    jmp init_start