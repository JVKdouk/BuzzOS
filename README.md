# Lab0: Booting BuzzOS

The goal of this lab is to get your environment setup, and some familiarity
gained with BuzzOS, particularly the boot process (which you'll be modifying in lab
1), as well as to gain some familiarity with the tools we'll be using as a part
of this course.

This is the only lab where you will submit answers to questions through Gradescope.
All other labs will be autograded.

## Requirements

In order to interface w/ our scripts, you must be able to run Bash and Docker.
Our recommendation for Windows users is to use WSL with Docker. Further help
can be provided through Piazza or Office Hours for getting this setup.

- Docker
- Bash
- Git

## Downloading, Compiling, and Running xv6

Start by checking out the xv6 repository on your local machine.

```bash
git clone git@github.gatech.edu:cs3210/buzzos.git
cd buzzos
git checkout lab0
```

Next, launch the docker instance for the class using the provided script.

```bash
./scripts/docker.sh --pull # download the image from DockerHub
./scripts/docker.sh # run container and mount the pwd as /xv6
```

Now that you're inside the docker instance, build your repository:

```bash
cargo make
```

This will build and launch the kernel. You can close qemu by pressing CTRL-a
followed by x.

## Observing behaviors with gdb

Now, we're going to run our code with gdb. We've attached a gdb flag to the BuzzOS
launcher script, please launch BuzzOS with gdb enabled:

```bash
cargo make gdb
```

This should pause qemu from launching, and wait for a gdb session to attach.
Now, we can connect to our Docker container in a separate terminal and launch
gdb from the build directory:

```bash
./scripts/docker.sh --attach
gdb
```

Once this is complete, it should take you to a gdb console, with the initial
BIOS `ljmp` instruction from the x86 machine's reset vector:

```x86
ljmp   $0xf000,$0xe05b
```

This is a 16-bit real-mode instruction (an obscure mode of the x86 processor run
at boot). The 0xf000 is the real-mode segment, with 0xe05b the jumped to
address. Look up real-mode addressing, what is the linear address to which it
is jumping? (this question is ungraded)

Find the address of \_start, the entry point of the kernel:

```bash
$ nm build/kernel.elf | grep _start
00100075 t kernel_start
**001000a0 T _start**
00137a40 T _ZN4core5slice5ascii30_$LT$impl$u20$$u5b$u8$u5d$$GT$16trim_ascii_start17hecf97417a20180cdE
```

The kernel address is at 001000a0.

Open gdb in the same directory, set a breakpoint and run to \_start as in the
following:

```bash
$ gdb
...
The target architecture is set to "auto" (currently "i386").
0x0000fff0 in ?? ()
+ symbol-file build/kernel.elf
warning: Missing auto-load script at offset 0 in section .debug_gdb_scripts
of file /home/jvck/Documents/Projects/BuzzOS/build/kernel.elf.
Use 'info auto-load python-scripts [REGEXP]' to list them.
(gdb) br *0x100000
Breakpoint 1 at 0x100000
(gdb) c
Continuing.
The target architecture is assumed to be i386

Breakpoint 1, 0x00100000 in entry ()
(gdb)


Look at the registers and stack:

(gdb) info reg
...
(gdb) x/24x $esp
...
(gdb)
```

The stack grows from higher addresses to lower in x86, so items pushed on the
stack will be at higher addresses the earlier they were pushed on.

## Graded Questions

Answer the following on Gradescope:

1. To what address is the stack initialized during the bootloading process? (Another way to answer this is to ask yourself what's the bottom of the stack?). Write your answer as a hexadecimal number (like `0xA1B2` for example).

2. What items are on the stack at this point (`pc = 0x100000`)?

To understand what is on the stack, you need to understand the boot procedure because at this point the kernel has not started, so anything on the stack was put there by the bootloader. Look at the files `bootloader/src/boot.asm`, and `bootloader/src/loader.asm`. Can you see what they are putting on the stack?

3. Restart QEMU and GDB as above but now set a break-point at `0x7C00`. This is the start of the bootloader (`bootloader/src/boot.asm`). Using the single instruction step (`si`) step through the bootloader. Where is the stack pointer initialized?

4. Single-step into the load_kernel function. Now, look at the stack using `x/24x $esp`. What is in there?

5. What does the initial assembly of the boot procedure do to the stack? (Look in `build/boot.asm` and try to reason in which moments the Stack is modified)

6. Continue tracing. You can use breakpoints to skip over things. Look for where `eip` is set to `0x100000`. What happens to the stack as a result of that call? What would happen if we used a jump instead of a call?
