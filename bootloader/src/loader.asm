bits 32
load_kernel:
    mov edx, KERNEL_BUFFER  ; Destination Address
    mov edi, 4096           ; Bytes to Read
    mov esi, 0              ; Offset
    call load_seg

    ; ELF files have a magic number to facilitate identification. If the chunk of data that was
    ; read does not have the magic number, then the Kernel loading procedure failed.
    cmp dword [KERNEL_BUFFER], ELF_MAGIC
    jne kernel_load_failed

    ; With the ELF Header successfuly loaded, we start decoding it to find the program headers
    xor ax, ax

    ; Program Header Offset
    mov ebx, dword [KERNEL_BUFFER + ELF_PH_OFFSET] 
    add ebx, KERNEL_BUFFER

    ; Number of Program Headers (Loop Iterator)
    mov ax, word [KERNEL_BUFFER + ELF_PHNUM_OFFSET]
    mov cx, ELF_PH_SIZE
    mul cx
    add eax, ebx

; Load each program header into their respective address in memory
load_program_headers:
    mov edx, dword [ebx + ELF_PHPA_OFFSET] ; Physical Address
    mov edi, dword [ebx + ELF_PHFILESZ_OFFSET] ; File Size
    mov esi, dword [ebx + ELF_PHOFF_OFFSET]  ; Sector Offset 
    call load_seg

    ; Check if the size in memory is bigger than the file size. The excess size in memory must be
    ; padded with zeroes.
    mov edi, dword [ebx + ELF_PHMEMSZ_OFFSET]
    cmp edi, dword [ebx + ELF_PHFILESZ_OFFSET]
    jle .next

    ; Fill empty spaces with zeroes
    add esi, dword [ebx + ELF_PHFILESZ_OFFSET]
    mov al, 0
    mov ecx, dword [ebx + ELF_PHMEMSZ_OFFSET]
    sub ecx, dword [ebx + ELF_PHFILESZ_OFFSET]
    cld
    rep stosb

; Get the next program header
.next:
    add ebx, ELF_PH_SIZE
    cmp ebx, eax
    jl load_program_headers

    ret

; Load segment from disk
; edx - Destination in Memory
; edi - Number of bits to read
; esi - Sector offset to start reading
load_seg:
    mov ebx, edx
    
    ; Calculate stop address
    add edi, edx

    ; Find first sector to read
    mov ecx, SECTOR_SIZE
    mov edx, 0
    mov eax, esi
    div ecx
    mov eax, edx
    sub ebx, eax

    ; Format offset
    mov edx, 0
    mov eax, esi
    mov ecx, SECTOR_SIZE
    div ecx
    mov esi, eax
    add esi, 1   ; Sector Offset

    mov ecx, edi
    sub ecx, ebx ; Loop Counter

load_sector:
    call is_disk_available

    ; Number of Blocks (1)
    mov al, 1
    mov dx, ATA_SECTOR_COUNT
    out dx, al

    mov eax, esi

    ; Offset low 8 bits
    mov dx, ATA_LBA_LOW
    out dx, al

    ; Offset next 8 bits
    shr eax, 8
    mov dx, ATA_LBA_MID
    out dx, al

    ; Offset next 8 bits
    shr eax, 8
    mov dx, ATA_LBA_HIGH
    out dx, al

    ; Offset next 8 bits
    shr eax, 8
    or eax, 0xe0
    mov dx, ATA_DEVICE_SELECT
    out dx, al

    ; Read sectors
    mov al,  ATA_CMD_READ_SECTORS
    mov edx, ATA_COMMAND_REG
    out dx, al

    call is_disk_available

    push ecx
    mov edi, ebx             ; Where to store (Temporary Buffer)
    mov edx, 0x1f0           ; Port address to copy from
    mov ecx, SECTOR_SIZE / 4 ; How many bytes to copy
    
    ; Copy data into memory
    cld
    rep insd
    pop ecx

    ; Prepare next iteration
    add esi, 1
    add ebx, SECTOR_SIZE
    sub ecx, SECTOR_SIZE

    ; Loop
    cmp ecx, 0
    jg load_sector

    ret

is_disk_available:
    xor al, al
    mov dx, ATA_STATUS_REG
    in  al, dx
    and al, 0xc0
    cmp al, 0x40
    jne is_disk_available
    ret

kernel_load_failed:
    cli
    hlt