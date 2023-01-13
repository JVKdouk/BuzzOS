; Memory Locations
%define KERNEL_BUFFER        0x500
%define KERNEL_ENTRY         0x100000

; Misc
%define SECTOR_SIZE          512

; ATA PIO
%define ATA_DATA_REG         0x1F0
%define ATA_ERROR_REG        0x1F1
%define ATA_SECTOR_COUNT     0x1F2
%define ATA_LBA_LOW          0x1F3
%define ATA_LBA_MID          0x1F4
%define ATA_LBA_HIGH         0x1F5
%define ATA_DEVICE_SELECT    0x1F6
%define ATA_COMMAND_REG      0x1F7
%define ATA_STATUS_REG       0x1F7
%define ATA_CMD_READ_SECTORS 0x20

; ELF Header
%define ELF_ENTRY            24
%define ELF_PH_OFFSET        28
%define ELF_PHNUM_OFFSET     40

; ELF Program Header
%define ELF_PHPA_OFFSET      12
%define ELF_PHFILESZ_OFFSET  16
%define ELF_PHMEMSZ_OFFSET   20
%define ELF_PHOFF_OFFSET     4
%define ELF_PH_SIZE          32
%define ELF_MAGIC            0x464C457F