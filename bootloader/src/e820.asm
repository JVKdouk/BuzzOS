dest_addr:
  dw 0x0

global do_e820
do_e820:
  ; Save destination address and setup destination register (di)
  mov [dest_addr], ax
  add ax, 4
  mov di, ax

  ; Clear registers
  xor ebx, ebx
  xor bp, bp

  mov edx, E820_MAGIC_NUMBER

.e820_loop:
  mov eax, 0xe820
  mov [es:di + 20], dword 1 ; Force a valid ACPI entry
  mov ecx, 24
  int 0x15
  jc .e820f
  mov edx, E820_MAGIC_NUMBER ; BIOS compatibility procedure

  cmp eax, edx
  jne .e820f
  test ebx, ebx
  je .e820f
  jcxz .skipent
  cmp cl, 0x20
  inc bp
  add di, 24

.skipent:
  test ebx, ebx
  jne .e820_loop

.e820f:
  xor eax, eax
  mov ax, [dest_addr]
  mov [eax], bp
  ret