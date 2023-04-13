; TODO: Add scheduler-specific context
global switch
switch:
mov edx, dword [esp + 4]
; mov edx, dword [esp + 8]

push ebp
push ebx
push esi
push edi

; mov [eax], dword esp
mov esp, dword edx

pop edi
pop esi
pop ebx
pop ebp

ret