OUTPUT_FORMAT("elf64-x86-64", "elf64-x86-64", "elf64-x86-64")
OUTPUT_ARCH(i386:x86-64)
ENTRY(start)

SECTIONS {
	. = 0x100000;

	.text.entry : {
		*(.text.entry)
	}

    .text : {
		*(.text .text.* .stub .gnu.linkonce.t.*)
		. = ALIGN(4096);
	}

	.data : {
		*(.data)
		. = ALIGN(4096);
	}

	.rodata : {
		*(.rodata*)
		. = ALIGN(4096);
	}

	.bss ALIGN (0x1000) : {
		*(COMMON)
		*(.bss .bss.*)
		. = ALIGN(4096);
	}

	.eh_frame : AT(ADDR(.eh_frame) - 0x100000) {
       *(.eh_frame)
        . = ALIGN(4096);
   	}

	KERNEL_END = .;
}