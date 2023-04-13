global entry

%define KERNEL_STACK_SIZE 16384

; This is the entry point of the Kernel. At this point, the bootload has put us in 32 bits mode and loaded
; the Kernel into memory. Now, it is our turn to perform any changes that are needed by our Kernel to
; initialized correctly, such as setting up the empty IDT and enabling paging.

global kernel_start
kernel_start: equ entry - 0x80000000

section .text.kernel
bits 32
entry:
    mov esp, stack_top

    lidt [zero_idt]

    call enable_paging

    ; Finally, time to get Rusty
    extern _start
    mov eax, _start
    jmp eax

    ; If the above instruction fails, we halt the processor.
    hlt

; Time to enable paging. Here, page size extensions are enabled to allow for bigger pages
; and the initial page directory is loaded into CR3. Notice this page directory will be replace
; as soon as we can, so we can build something more flexible.
enable_paging:
    ; Enable Size Extension (4MB per page)
    mov eax, cr4
    or eax, 0x00000010
    mov cr4, eax
    
    ; Set CR3 = Page Dir Address (Our Kernel behaves like a process,
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

; After the bootloading process, we need to setup a more reliable, reserved space for our Stack.
; This is done in this step. It will, however, later be replaced with a dynamic page allocation for
; the stack.
section .bss
stack_bottom:
    resb KERNEL_STACK_SIZE ; 16 kB of Stack
stack_top:

; Before we are able of setting up the Interrupt Descriptor Table, we need to provide the Kernel with an
; empty one. You can find more about IDT here: https://wiki.osdev.org/Interrupt_Descriptor_Table
section .rodata
align 4
zero_idt:
    dw 0
    db 0