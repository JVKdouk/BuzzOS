global long_mode_start

section .initial
bits 64
long_mode_start:
    cli

    ; Old GDT is still loaded into the registers below. We need to ensure they have valid data (0) even
    ; though they are mostly unused by 64 bit mode.
    mov ax, 0  ; Accumulator Register
    mov ss, ax ; Stack Segment Register
    mov ds, ax ; Data Segment Register
    mov es, ax ; Extra Segment Register
    mov fs, ax ; Extra Segment Register
    mov gs, ax ; Extra Segment Register

    ; Finally, time to get Rusty
    extern _start
    call _start

    ; print `OKAY` to screen
    mov rax, 0x2f592f412f4b2f4f
    mov qword [0xb8000], rax
    hlt