; Bootloader entry point. This file contains the first 512 bytes to be loaded by the BIOS.
; Its goal is to load the rest of the Kernel and setup the jump from 16 to 32 bits.

global _start_16

%include "src/defs.asm"

section .text
bits 16
_start_16:
    ; Clear interrupts
    cli
    
    ; Zero out segment registers
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    ; Clear direction bits
    cld

enable_a20:
    in al, 0x64
    test al, 0x2
    jnz enable_a20
    mov al, 0xd1
    out 0x64, al
enable_a20_2:
    in al, 0x64
    test al, 0x2
    jnz enable_a20_2
    mov al, 0xdf
    out 0x60, al

prepare_protected_mode:
    ; LGDT loads the GDTR (GDT Register) with the provided value.
    lgdt [gdt32.pointer]

    ; Set protected bit in CR0. This prepares us to jump to protected mode (32 bits)
    mov eax, cr0
    or eax, 1
    mov cr0, eax

    ; Perform the jump to 32 bits
    jmp gdt32.code:start_32

bits 32
start_32:
    ; Update segment registers
    mov ax, gdt32.data
    mov ds, ax
    mov es, ax
    mov ss, ax

    ; Reset other selector registers
    mov ax, 0
    mov gs, ax
    mov fs, ax

    ; Setup stack pointer
    mov sp, 0x7C00

prepare_load_kernel:
    call load_kernel

    ; Jump to Kernel entry point in memory
    call ebx
    
    ; This part is unreachable. In case it is reached, something went very wrong.
    hlt

%include "src/loader.asm"

; General descriptor table. This is used to perform linear address translation.
; The first entry of the GDT must be zero. The second entry is commonly used for the
; code segment and the third entry is the data segment. Segment selectors (such as CS,
; DS etc), use these entries to perform linear translation.
; More information can be found here https://wiki.osdev.org/Global_Descriptor_Table
gdt32:
    dq 0
.code: equ $ - gdt32
    dq 0x00CF9A000000FFFF
.data: equ $ - gdt32
    dq 0x00CF92000000FFFF
.pointer:
    dw $ - gdt32 - 1  ; GDT Table Size
    dq gdt32          ; GDT Table Offset

; Add MBR signature to binary. This allows the BIOS to see this portion of the disk
; as a Master Boot Record (MBR).
times 510-($-$$) db 0x0
db 0x55
db 0xaa