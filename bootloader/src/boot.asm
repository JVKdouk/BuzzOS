; Bootloader entry point. This file contains the first 512 bytes to be loaded by the BIOS.
; Its goal is to load the rest of the Kernel and setup the jump from 16 to 32 bits.

global _start
org 0x7c00    ; Bootloaders are commonly loaded at this address in memory

%include "src/defs.asm"

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

    ; Setup stack pointer
    mov sp, 0x7c00

enable_a20:
    ; enable A20-Line via IO-Port 92, might not work on all motherboards
    in al, 0x92
    test al, 2
    jnz prepare_load_kernel
    or al, 2
    and al, 0xFE
    out 0x92, al

prepare_load_kernel:
    ; Enable interrupts
    sti
    call load_kernel
    cli

prepare_protected_mode:
    ; LGDT loads the GDTR (GDT Register) with the provided value.
    lgdt [gdt32.pointer]

    ; Set protected bit in CR0. This prepares us to jump to protected mode (32 bits)
    mov eax, cr0
    or eax, 1
    mov cr0, eax

    ; Perform the jump to 32 bits
    jmp gdt32.code:start_32

%include "src/loader.asm"

bits 32
start_32:
    ; Update segment registers
    mov eax, gdt32.data
    mov ds, eax
    mov es, eax
    mov fs, eax
    mov gs, eax
    mov ss, eax

    ; Jump to Kernel entry point in memory
    mov ebx, KERNEL_ENTRY
    jmp ebx
    
    ; This part is unreachable. In case it is reached, something went very wrong,
    ; print fail and halt the processor.  
    mov eax, 0x4f414f46
    mov dword [0xb8000], eax
    mov eax, 0x4f4c4f49
    mov dword [0xb8004], eax
    hlt



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

; Device Access Packet. This packet is provided to the BIOS on interrupt 0x13, function
; 0x42 (extended read sector). It provides some information, such as number of sectors to read
; address of the memory buffer, etc.
; More information can be found here https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)
dap:
    db 0x10      ; size of dap
    db 0         ; unused
dap_blocks:
    dw 0         ; number of sectors
dap_buffer_addr:
    dw 0         ; offset to memory buffer
dap_buffer_seg:
    dw 0         ; segment of memory buffer
dap_start_lba:
    dq 0         ; start logical block address

; Add MBR signature to binary. This allows the BIOS to see this portion of the disk
; as a Master Boot Record (MBR).
times 510-($-$$) db 0x0
db 0x55
db 0xaa