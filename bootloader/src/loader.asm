load_kernel:
    ; Check if int 0x13, function 0x42 is available
    call check_int13h_extensions

    ; Memory buffer address
    mov eax, KERNEL_BUFFER
    mov [dap_buffer_addr], ax

    ; Number of sectors to load (512B each)
    mov word [dap_blocks], 1

    ; Address in the disk for the first sector. Convert from sector address to number.
    mov eax, 0x200
    shr eax, 9
    mov [dap_start_lba], eax

    ; Destination address
    mov edi, KERNEL_ENTRY

    ; Total number of iterations to load the entire Kernel
    mov ecx, KERNEL_SIZE
    add ecx, 511 ; align up
    shr ecx, 9

load_next_kernel_block_from_disk:
    ; Load block from disk
    mov si, dap
    mov ah, 0x42
    int 0x13
    jc kernel_load_failed

    ; Copy from buffer to destination address
    push ecx
    push esi
    mov ecx, 512 / 4
    movzx esi, word [dap_buffer_addr]
    a32 rep movsd
    pop esi
    pop ecx

    ; Prepare to load next sector
    mov eax, [dap_start_lba]
    add eax, 1
    mov [dap_start_lba], eax

    ; Count down the iterator
    sub ecx, 1
    jnz load_next_kernel_block_from_disk

    ret

check_int13h_extensions:
    mov ah, 0x41
    mov bx, 0x55aa
    int 0x13
    jc no_int13h_extensions

    ret

no_int13h_extensions:
    hlt

kernel_load_failed:
    hlt