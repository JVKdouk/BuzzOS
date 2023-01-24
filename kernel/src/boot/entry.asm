global entry

%define KERNEL_STACK_SIZE 16384

; This is the entry point of the Kernel. At this point, GRUB has put us in 32 bits mode and loaded
; the Kernel into memory. We are free to move on and not worry about A20, 32 bits jump, and disk image
; loading. A couple checks are necessary before we can move to the Rust part.

global kernel_start
kernel_start: equ entry - 0x80000000

; Multibot check comes first, as EAX currently has the magic number
section .text
bits 32
entry:
    mov esp, stack_top

    lidt [zero_idt]

    ; We are dealing with a 2-level paging, so 2 pages are necessary.
    call enable_paging

    ; Finally, time to get Rusty
    extern _start
    mov eax, _start
    jmp eax

    ; If the above instruction fails, we halt the processor.
    hlt

; With page table, we are now ready to enable long mode in our process. This involves setting
; CR3 to the address of PML4 (base of the process's page-mapped level 4).
enable_paging:
    ; Enable Size Extension (4MB per page)
    mov eax, cr4
    or eax, 0x00000010
    mov cr4, eax
    
    ; Set CR3 = PML4 Address (Our Kernel behaves like a process,
    ; and the CPU must translate every address accessed)
    mov eax, pd_table - 0x80000000
    mov cr3, eax

    mov eax, cr0    ; Read current value of CR0
    or eax, 1 << 31 ; Update 32nd bit (Paging Enable)
    mov cr0, eax    ; Update CR0

    ret

align 4096 ; Ensures page alignment
pd_table:
    dd 0x83 ; Allows access to [0, 4MiB) section of memory
    resd 511
    dd 0x83 ; Maps [KERNBASE, KERNBASE + 4MiB) section of memory to physical addresses
    resd 511
pt_end:

section .bss
; After our initial 4 pages, we set the stack (64 bytes). Stack grows downward, meaning we go from
; Stack top to bottom. If stack goes beyond the 64 allocated bytes, we may have corruption of the page
; tables.
stack_bottom:
    resb KERNEL_STACK_SIZE ; 16 kB of Stack
stack_top:

section .rodata
align 4
zero_idt:
    dw 0
    db 0