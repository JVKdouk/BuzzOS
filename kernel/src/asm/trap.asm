global trap_return
trap_return:
    popa
    pop gs
    pop fs
    pop es
    pop ds
    add esp, 0x8
    iret