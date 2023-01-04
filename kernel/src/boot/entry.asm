global start
extern long_mode_start

; This is the entry point of the Kernel. At this point, GRUB has put us in 32 bits mode and loaded
; the Kernel into memory. We are free to move on and not worry about A20, 32 bits jump, and disk image
; loading. A couple checks are necessary before we can move to the Rust part.

; Multibot check comes first, as EAX currently has the magic number
section .text.entry
bits 32
start:
    cli

    ; Save memory layout address
    mov edi, ebx       ; EDI is the first argument to be passed to a function (Linux calling convention)

    ; Quick checks before boot
    call check_cpuid     ; Check if CPUID is supported, this provides access to CPU information
    call check_long_mode ; Check if the device supports long mode (64 bit)

    lidt [zero_idt]

    ; We are dealing with a 4-level paging, so 4 pages are necessary, namely:
    ; - Page-Map Level-4 Table
    ; - Directory Pointer Table
    ; - Directory Table
    ; - Page Table
    ; This is required by the x86 CPU in 64 bits mode (long).
    call set_up_page_tables
    call enable_paging

    ; Long mode is indeed enabled, but we cannot use Long mode yet! To support old hardware,
    ; we must instruct the processor on how to perform segmentation. This is done via the usage
    ; of GDT (Global Descriptor Table). This is how Virtual Addresses were handled a long time
    ; ago. So far we have been using a 32-bit GDT provided by GRUB, so we need to build our
    ; 64 bits GDT.
    lgdt [gdt64.pointer]

    ; GDT has not yet reloaded our CS register. We must do a long jump to reload it.
    ; gdt64.code is the new CS (Code Segment) value, while long_mode_start is an external
    ; address we will jump to. Notice the syntax, CS:Address. This will set the address as an
    ; offset of CS.
    jmp gdt64.code:long_mode_start

    ; If the above instruction fails, we halt the processor.
    hlt

; To check CPUID, we must interact with the the EFLAGS register. pushfd pushes the current value of
; EFLAGS to the stack. We then pop that value from the stack into eax, copy it to ecx and flip the 22nd
; bit. If the this bit can flipped, then CPUID is supported. Finally, we restore EFLAGS
; More information here: https://en.wikipedia.org/wiki/CPUID
check_cpuid:
    pushfd
    pop eax

    ; Copy EFLAGS to ECX
    mov ecx, eax

    ; Flip the ID bit
    xor eax, 1 << 21

    ; Save the new EFLAGS
    push eax
    popfd

    ; Copy the EFLAGS back to EAX
    pushfd
    pop eax

    ; Restore EFLAGS to the previous version (unflipped ID bit)
    push ecx
    popfd

    ; Compare EAX and ECX. If EAX = ECX, they were not flipped
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov al, "1"
    jmp error

; To check if the processor has support to long mode (64 bits), we need to ask CPUID. CPUID has leaves,
; 0x80000000 is one of them, and checks for the "extended" mode. If the extended mode is available, CPUID
; will flip EAX least-significant bit.
check_long_mode:
    mov eax, 0x80000000    ; CPUID leaf setup
    cpuid                  ; Check for extended mode
    cmp eax, 0x80000001    ; If it supports extended function, then the first bit will be a 1
    jb .no_long_mode

    ; We know extended function is available, time to use it
    mov eax, 0x80000001    ; Requires the usage of the extended function
    cpuid                  ; Return feature bits
    test edx, 1 << 29      ; Test the Long Mode bit
    jz .no_long_mode       ; If it is zero, long mode is not supported
    ret
.no_long_mode:
    mov al, "2"
    jmp error

; Check for the magic number on EAX. Right before GRUB gives us the ownership over the processor,
; it sets EAX to the magic number of multiboot. If this number is present, our Kernel was booted
; via Multiboot. You can find more at https://wiki.osdev.org/Multiboot
check_multiboot:
    cmp eax, 0x36d76289 ; Magic number of Multiboot
    jne .no_multiboot   ; If it is not the magic number, error
    ret
.no_multiboot: ; [To keep in mind]: Symbolic Labels with a dot are not added to the symbolic table
    mov al, "0"
    jmp error

; PML4 -> PDP -> PD -> PT -> Page
set_up_page_tables:
    ; Set PML4 first entry to PDP table address
    ; Set PML4 entry flags to present and writable
    mov eax, pdp_table
    or eax, 0b11 
    mov [pml4_table], eax

    ; Set PDP first entry to PD table address
    ; Set PDP entry flags to present and writable
    mov eax, pd_table
    or eax, 0b11
    mov [pdp_table], eax

    ; Map Page Directory Entry to a 2 MiB page (512 entries * page size)
    ; Counter variable
    mov ecx, 0

; [Food for Thought]: Why not 4 page level now? Wouldn't it be the same as we did for PML4 and PDP?
; The first step to switching to long mode involves enabling PAE (Physical Address Extension), which
; tells the processor to use 3 pages instead of 2 (from 32 bits protected mode). At this point, we cannot
; have 4 page levels yet!
.map_pd_table:
    mov eax, 0x200000             ; Start of page
    mul ecx                       ; Multiply start by number of entries passed (0, 2MiB, 4MiB)
    or eax, 0b10000011            ; Tell x86 we are working with huge pages (no page tables), present and writable
    mov [pd_table + ecx * 8], eax ; Set entry to the right value

    inc ecx            ; Go to next entry
    cmp ecx, 512       ; If we reach 512 entries, this loop is done
    jne .map_pd_table  ; Go to next iteration

    ret

; Kata OS
; Joao Victor Cardoso Kdouk

; With page table, we are now ready to enable long mode in our process. This involves setting
; CR3 to the address of PML4 (base of the process's page-mapped level 4).
enable_paging:
    wbinvd
    mfence
    
    ; Set CR3 = PML4 Address (Our Kernel behaves like a process,
    ; and the CPU must translate every address accessed)
    mov eax, pml4_table
    mov cr3, eax

    ; First step to long mode enabling Page Address Extension, allowing 3 levels of paging + 64 bits per entry
    mov eax, cr4
    or eax, 1 << 5 ; 6th bit represents Page Address Extension
    mov cr4, eax

    ; Time to set the long mode bit and tell the processor we want long mode. AMD and Intel support this register
    mov ecx, 0xC0000080 ; Model Specific Register (MSR) Number
    rdmsr               ; Read Model Specific Register (at 0xC0000080)
    or eax, 1 << 8      ; Set long mode bit
    wrmsr               ; Write to register

    ; Fianlly, long mode is active, we just need to tell the processor to use paging.
    ; Memory addresses cannot be accessed directly now.
    mov eax, cr0    ; Read current value of CR0
    or eax, 1 << 31 ; Update 32nd bit (Paging Enable)
    mov cr0, eax    ; Update CR0

    ret


; Error handler
error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt

; GDTR Setup

; Global Description Table Register (GDTR), it is 79 bits long (64 bits mode), where upper
; 49 bits tell what the linear address of the GDT Descriptor is, and the lower 16 bits tell
; what the size of it is. Size is in bytes subtracted by 1. Notice the address pointer by the
; upper bits is still a linear address. Paging still applies.

; Global Description Table is composed of Segment Descriptors, each one being 8 bytes long. Every Segment
; Descriptor is referenced by its offset, and not its actual memory address, that way, GDTR + n always points
; to a valid segment. Calculating offset can be done by taking the current address ($) and subtracting
; the GDT label address from it. Segmentation is done below to simply fulfill the requirement, as there is not limit
; to any of the segments.

section .rodata

; Zero Entry (GDT always starts with an empty entry). dq defines a quad word (64 bits in x86)
gdt64:
    dq 0                  ; 8 bytes null segment descriptor
.code: equ $ - gdt64      ; Use current address minus the gdt64 address
    dq 0x0020980000000000 ; Define our Code Segment (Executable, Code, Present, 64 bits)
.data: equ $ - gdt64
    dq 0x0000920000000000 ; Define our Data Segment (Data, Present, Writable).
.pointer:
    dw $ - gdt64 - 1 ; Offsets to the above entry (.pointer - gdt64 - 1)
    dq gdt64         ; Define an entry for the initiated GDT

section .bss
align 4096 ; Ensures page alignment
pml4_table:
    resb 4096
pdp_table:
    resb 4096
pd_table:
    resb 4096
pt_end:

; After our initial 4 pages, we set the stack (64 bytes). Stack grows downward, meaning we go from
; Stack top to bottom. If stack goes beyond the 64 allocated bytes, we may have corruption of the page
; tables.
stack_bottom:
    resb 16384 ; 16 kB of Stack
stack_top:

section .rodata
align 4
zero_idt:
    dw 0
    db 0