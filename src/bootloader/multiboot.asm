section .multiboot_header
header_start:
    dd 0xe85250d6                ; magic number (multiboot 2)
    dd 0                         ; protected mode
    dd header_end - header_start ; header length
    
    ; We must have that checksum + magic + mode + architecture = 0, thus we need to subtract from 0x10000000
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

    ; required end tag
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
header_end: