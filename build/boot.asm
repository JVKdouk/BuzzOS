
../build/bootloader.elf:     file format elf32-i386


Disassembly of section .text:

00007c00 <_start_16>:
    7c00:	fa                   	cli
    7c01:	b8 20 e8 e8 38       	mov    $0x38e8e820,%eax
    7c06:	00 31                	add    %dh,(%ecx)
    7c08:	c0 8e d8 8e c0 8e d0 	rorb   $0xd0,-0x713f7128(%esi)
    7c0f:	8e e0                	mov    %eax,%fs
    7c11:	8e e8                	mov    %eax,%gs
    7c13:	fc                   	cld

00007c14 <enable_a20>:
    7c14:	e4 64                	in     $0x64,%al
    7c16:	a8 02                	test   $0x2,%al
    7c18:	75 fa                	jne    7c14 <enable_a20>
    7c1a:	b0 d1                	mov    $0xd1,%al
    7c1c:	e6 64                	out    %al,$0x64

00007c1e <enable_a20_2>:
    7c1e:	e4 64                	in     $0x64,%al
    7c20:	a8 02                	test   $0x2,%al
    7c22:	75 fa                	jne    7c1e <enable_a20_2>
    7c24:	b0 df                	mov    $0xdf,%al
    7c26:	e6 60                	out    %al,$0x60

00007c28 <prepare_protected_mode>:
    7c28:	0f 01 16             	lgdtl  (%esi)
    7c2b:	dd 7d 0f             	fnstsw 0xf(%ebp)
    7c2e:	20 c0                	and    %al,%al
    7c30:	66 83 c8 01          	or     $0x1,%ax
    7c34:	0f 22 c0             	mov    %eax,%cr0
    7c37:	ea                   	.byte 0xea
    7c38:	93                   	xchg   %eax,%ebx
    7c39:	7c 08                	jl     7c43 <do_e820+0x4>
    7c3b:	00 f4                	add    %dh,%ah

00007c3d <dest_addr>:
	...

00007c3f <do_e820>:
    7c3f:	a3 3d 7c 83 c0       	mov    %eax,0xc0837c3d
    7c44:	04 89                	add    $0x89,%al
    7c46:	c7                   	(bad)
    7c47:	66 31 db             	xor    %bx,%bx
    7c4a:	31 ed                	xor    %ebp,%ebp
    7c4c:	66 ba 50 41          	mov    $0x4150,%dx
    7c50:	4d                   	dec    %ebp
    7c51:	53                   	push   %ebx

00007c52 <do_e820.e820_loop>:
    7c52:	66 b8 20 e8          	mov    $0xe820,%ax
    7c56:	00 00                	add    %al,(%eax)
    7c58:	26 66 c7 45 14 01 00 	movw   $0x1,%es:0x14(%ebp)
    7c5f:	00 00                	add    %al,(%eax)
    7c61:	66 b9 18 00          	mov    $0x18,%cx
    7c65:	00 00                	add    %al,(%eax)
    7c67:	cd 15                	int    $0x15
    7c69:	72 1e                	jb     7c89 <do_e820.e820f>
    7c6b:	66 ba 50 41          	mov    $0x4150,%dx
    7c6f:	4d                   	dec    %ebp
    7c70:	53                   	push   %ebx
    7c71:	66 39 d0             	cmp    %dx,%ax
    7c74:	75 13                	jne    7c89 <do_e820.e820f>
    7c76:	66 85 db             	test   %bx,%bx
    7c79:	74 0e                	je     7c89 <do_e820.e820f>
    7c7b:	e3 07                	jecxz  7c84 <do_e820.skipent>
    7c7d:	80 f9 20             	cmp    $0x20,%cl
    7c80:	45                   	inc    %ebp
    7c81:	83 c7 18             	add    $0x18,%edi

00007c84 <do_e820.skipent>:
    7c84:	66 85 db             	test   %bx,%bx
    7c87:	75 c9                	jne    7c52 <do_e820.e820_loop>

00007c89 <do_e820.e820f>:
    7c89:	66 31 c0             	xor    %ax,%ax
    7c8c:	a1 3d 7c 67 89       	mov    0x89677c3d,%eax
    7c91:	28 c3                	sub    %al,%bl

00007c93 <start_32>:
    7c93:	66 b8 10 00          	mov    $0x10,%ax
    7c97:	8e d8                	mov    %eax,%ds
    7c99:	8e c0                	mov    %eax,%es
    7c9b:	8e d0                	mov    %eax,%ss
    7c9d:	66 b8 00 00          	mov    $0x0,%ax
    7ca1:	8e e8                	mov    %eax,%gs
    7ca3:	8e e0                	mov    %eax,%fs
    7ca5:	66 bc 00 7c          	mov    $0x7c00,%sp

00007ca9 <prepare_load_kernel>:
    7ca9:	e8 03 00 00 00       	call   7cb1 <load_kernel>
    7cae:	ff d3                	call   *%ebx
    7cb0:	f4                   	hlt

00007cb1 <load_kernel>:
    7cb1:	ba 00 05 00 00       	mov    $0x500,%edx
    7cb6:	bf 00 10 00 00       	mov    $0x1000,%edi
    7cbb:	be 00 00 00 00       	mov    $0x0,%esi
    7cc0:	e8 60 00 00 00       	call   7d25 <load_seg>
    7cc5:	81 3d 00 05 00 00 7f 	cmpl   $0x464c457f,0x500
    7ccc:	45 4c 46 
    7ccf:	0f 85 ee 00 00 00    	jne    7dc3 <kernel_load_failed>
    7cd5:	66 31 c0             	xor    %ax,%ax
    7cd8:	8b 1d 1c 05 00 00    	mov    0x51c,%ebx
    7cde:	81 c3 00 05 00 00    	add    $0x500,%ebx
    7ce4:	66 a1 28 05 00 00    	mov    0x528,%ax
    7cea:	66 b9 20 00          	mov    $0x20,%cx
    7cee:	66 f7 e1             	mul    %cx
    7cf1:	01 d8                	add    %ebx,%eax

00007cf3 <load_program_headers>:
    7cf3:	8b 53 0c             	mov    0xc(%ebx),%edx
    7cf6:	8b 7b 10             	mov    0x10(%ebx),%edi
    7cf9:	8b 73 04             	mov    0x4(%ebx),%esi
    7cfc:	e8 24 00 00 00       	call   7d25 <load_seg>
    7d01:	8b 7b 14             	mov    0x14(%ebx),%edi
    7d04:	3b 7b 10             	cmp    0x10(%ebx),%edi
    7d07:	7e 0e                	jle    7d17 <load_program_headers.next>
    7d09:	03 73 10             	add    0x10(%ebx),%esi
    7d0c:	b0 00                	mov    $0x0,%al
    7d0e:	8b 4b 14             	mov    0x14(%ebx),%ecx
    7d11:	2b 4b 10             	sub    0x10(%ebx),%ecx
    7d14:	fc                   	cld
    7d15:	f3 aa                	rep stos %al,%es:(%edi)

00007d17 <load_program_headers.next>:
    7d17:	83 c3 20             	add    $0x20,%ebx
    7d1a:	39 c3                	cmp    %eax,%ebx
    7d1c:	7c d5                	jl     7cf3 <load_program_headers>
    7d1e:	8b 1d 18 05 00 00    	mov    0x518,%ebx
    7d24:	c3                   	ret

00007d25 <load_seg>:
    7d25:	89 d3                	mov    %edx,%ebx
    7d27:	01 d7                	add    %edx,%edi
    7d29:	b9 00 02 00 00       	mov    $0x200,%ecx
    7d2e:	ba 00 00 00 00       	mov    $0x0,%edx
    7d33:	89 f0                	mov    %esi,%eax
    7d35:	f7 f1                	div    %ecx
    7d37:	89 d0                	mov    %edx,%eax
    7d39:	29 c3                	sub    %eax,%ebx
    7d3b:	ba 00 00 00 00       	mov    $0x0,%edx
    7d40:	89 f0                	mov    %esi,%eax
    7d42:	b9 00 02 00 00       	mov    $0x200,%ecx
    7d47:	f7 f1                	div    %ecx
    7d49:	89 c6                	mov    %eax,%esi
    7d4b:	83 c6 01             	add    $0x1,%esi
    7d4e:	89 f9                	mov    %edi,%ecx
    7d50:	29 d9                	sub    %ebx,%ecx

00007d52 <load_sector>:
    7d52:	e8 5e 00 00 00       	call   7db5 <is_disk_available>
    7d57:	b0 01                	mov    $0x1,%al
    7d59:	66 ba f2 01          	mov    $0x1f2,%dx
    7d5d:	ee                   	out    %al,(%dx)
    7d5e:	89 f0                	mov    %esi,%eax
    7d60:	66 ba f3 01          	mov    $0x1f3,%dx
    7d64:	ee                   	out    %al,(%dx)
    7d65:	c1 e8 08             	shr    $0x8,%eax
    7d68:	66 ba f4 01          	mov    $0x1f4,%dx
    7d6c:	ee                   	out    %al,(%dx)
    7d6d:	c1 e8 08             	shr    $0x8,%eax
    7d70:	66 ba f5 01          	mov    $0x1f5,%dx
    7d74:	ee                   	out    %al,(%dx)
    7d75:	c1 e8 08             	shr    $0x8,%eax
    7d78:	0d e0 00 00 00       	or     $0xe0,%eax
    7d7d:	66 ba f6 01          	mov    $0x1f6,%dx
    7d81:	ee                   	out    %al,(%dx)
    7d82:	b0 20                	mov    $0x20,%al
    7d84:	ba f7 01 00 00       	mov    $0x1f7,%edx
    7d89:	ee                   	out    %al,(%dx)
    7d8a:	e8 26 00 00 00       	call   7db5 <is_disk_available>
    7d8f:	51                   	push   %ecx
    7d90:	89 df                	mov    %ebx,%edi
    7d92:	ba f0 01 00 00       	mov    $0x1f0,%edx
    7d97:	b9 80 00 00 00       	mov    $0x80,%ecx
    7d9c:	fc                   	cld
    7d9d:	f3 6d                	rep insl (%dx),%es:(%edi)
    7d9f:	59                   	pop    %ecx
    7da0:	83 c6 01             	add    $0x1,%esi
    7da3:	81 c3 00 02 00 00    	add    $0x200,%ebx
    7da9:	81 e9 00 02 00 00    	sub    $0x200,%ecx
    7daf:	83 f9 00             	cmp    $0x0,%ecx
    7db2:	7f 9e                	jg     7d52 <load_sector>
    7db4:	c3                   	ret

00007db5 <is_disk_available>:
    7db5:	30 c0                	xor    %al,%al
    7db7:	66 ba f7 01          	mov    $0x1f7,%dx
    7dbb:	ec                   	in     (%dx),%al
    7dbc:	24 c0                	and    $0xc0,%al
    7dbe:	3c 40                	cmp    $0x40,%al
    7dc0:	75 f3                	jne    7db5 <is_disk_available>
    7dc2:	c3                   	ret

00007dc3 <kernel_load_failed>:
    7dc3:	fa                   	cli
    7dc4:	f4                   	hlt

00007dc5 <gdt32>:
	...
    7dcd:	ff                   	(bad)
    7dce:	ff 00                	incl   (%eax)
    7dd0:	00 00                	add    %al,(%eax)
    7dd2:	9a cf 00 ff ff 00 00 	lcall  $0x0,$0xffff00cf
    7dd9:	00                   	.byte 0x0
    7dda:	92                   	xchg   %eax,%edx
    7ddb:	cf                   	iret
	...

00007ddd <gdt32.pointer>:
    7ddd:	17                   	pop    %ss
    7dde:	00 c5                	add    %al,%ch
    7de0:	7d 00                	jge    7de2 <gdt32.pointer+0x5>
	...
    7dfe:	55                   	push   %ebp
    7dff:	aa                   	stos   %al,%es:(%edi)
