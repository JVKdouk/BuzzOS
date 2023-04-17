; Pop all registers from the trap frame, skip elements, and then return to the start of
; the process being called.

extern user_interrupt_handler

global trap_enter
global trap_return

trap_enter:
    push 0  ; Error Code
    push 64 ; General User Trap Number

    push dword ds
    push dword es
    push dword fs
    push dword gs
    pusha

    mov ax, 0x10
    mov ds, ax
    mov es, ax

    push esp
    call user_interrupt_handler
    add esp, 4

trap_return:
    popa
    pop dword gs
    pop dword fs
    pop dword es
    pop dword ds
    add esp, 0x8 ; Skip Trap Number and Error Code
    iret