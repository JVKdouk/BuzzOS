; Pop all registers from the trap frame, skip elements, and then return to the start of
; the process being called.

extern interrupt_manager
extern vm_corruption_checker

global trap_enter
global trap_return

trap_enter:
    cli

    push ds
    push es
    push fs
    push gs
    pusha

    mov ax, 0x10
    mov ds, ax
    mov es, ax

    push esp
    call interrupt_manager
    add esp, 4

trap_return:
    popa
    pop gs
    pop fs
    pop es
    pop ds
    add esp, 0x8 ; Skip Trap Number and Error Code
    iret