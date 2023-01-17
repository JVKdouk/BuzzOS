global entry

%define KERNEL_STACK_SIZE 16384

; This is the entry point of the Kernel. At this point, GRUB has put us in 32 bits mode and loaded
; the Kernel into memory. We are free to move on and not worry about A20, 32 bits jump, and disk image
; loading. A couple checks are necessary before we can move to the Rust part.

; Multibot check comes first, as EAX currently has the magic number
section .text
bits 32
entry:
    mov esp, stack_top

    lidt [zero_idt]

    ; We are dealing with a 2-level paging, so 2 pages are necessary.
    call set_up_page_tables
    call enable_paging

    ; Finally, time to get Rusty
    extern _start
    mov eax, _start
    jmp _start

    ; If the above instruction fails, we halt the processor.
    hlt

; PML4 -> PDP -> PD -> PT -> Page
set_up_page_tables:
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

; With page table, we are now ready to enable long mode in our process. This involves setting
; CR3 to the address of PML4 (base of the process's page-mapped level 4).
enable_paging:
    ; Enable Size Extension (4MB per page)
    mov eax, cr4
    or eax, 0x00000010
    mov cr4, eax
    
    ; Set CR3 = PML4 Address (Our Kernel behaves like a process,
    ; and the CPU must translate every address accessed)
    mov eax, pd_table
    mov cr3, eax

    mov eax, cr0    ; Read current value of CR0
    or eax, 1 << 31 ; Update 32nd bit (Paging Enable)
    mov cr0, eax    ; Update CR0

    ret

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

section .bss
align 4096 ; Ensures page alignment
pd_table:
    resb 4096
pt_end:

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