OUTPUT_FORMAT("elf64-x86-64", "elf64-x86-64", "elf64-x86-64")
OUTPUT_ARCH(i386:x86-64)
ENTRY(start)

SECTIONS {
	. = 0x100000;

    .text : AT(ADDR(.text) - 0x100000) {
		*(.text .text.* .stub .gnu.linkonce.t.*)
	}

	.data : AT(ADDR(.data) - 0x100000) {
		*(.data)
		. = ALIGN(4096);
	}

	.bss ALIGN (0x1000) : {
		*(COMMON)
		*(.bss .bss.*)
		. = ALIGN(4096);
	}

	.rodata : {
		*(.rodata*)
	}

	.eh_frame : AT(ADDR(.eh_frame) - 0x100000) {
       *(.eh_frame)
        . = ALIGN(4096);
   	}

	KERNEL_END = .;
}