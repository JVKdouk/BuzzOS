; This code performs the switching between scheduler context to process context.
; After the switching has been completed, enter the trap_return to setup the registers.

global switch
switch:
    mov eax, dword [esp + 4]
    mov edx, dword [esp + 8]
    
    push ebp
    push ebx
    push esi
    push edi
    
    mov [eax], dword esp
    mov esp, dword edx

    pop edi
    pop esi
    pop ebx
    pop ebp

    ret