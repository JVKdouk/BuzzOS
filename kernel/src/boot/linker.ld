OUTPUT_FORMAT("elf32-i386", "elf32-i386", "elf32-i386")
OUTPUT_ARCH(i386)
ENTRY(kernel_start)

SECTIONS {
	. = 0x80100000;

	.text : AT(0x100000) {
		*(.text.* .text .stub .gnu.linkonce.t.*)
	}

	.interp : {
		*(.interp)
	}

	PROVIDE(etext = .);	/* Define the 'etext' symbol to this value */

	.rodata : {
		*(.rodata .rodata.* .gnu.linkonce.r.*)
	}

	.data.rel.ro : {
		*(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
	}

	. = ALIGN(0x1000);

	PROVIDE(KERNEL_DATA = .);

	/* The data segment */
	.data : AT(ADDR(.data) - 0x80000000) {
    *(.data)
	}

	.dynamic : {
		*(.dynamic)
	}

	.static : {
		*(.static)
	}

	/* Include debugging information in kernel memory */
	.stab : {
		PROVIDE(__STAB_BEGIN__ = .);
		*(.stab);
		PROVIDE(__STAB_END__ = .);
		BYTE(0)		/* Force the linker to allocate space
				   for this section */
	}

	.stabstr : {
		PROVIDE(__STABSTR_BEGIN__ = .);
		*(.stabstr);
		PROVIDE(__STABSTR_END__ = .);
		BYTE(0)		/* Force the linker to allocate space
				   for this section */
	}

	PROVIDE(edata = .);

	.bss : AT(ADDR(.bss) - 0x80000000) {
		*(.bss)
	}

	.absolute : {
		*(.absolute)
	}

	.undefined : {
		*(.undefined)
	}

	PROVIDE(KERNEL_END = .);

	/DISCARD/ : {
		*(.eh_frame .note.GNU-stack)
	}
}