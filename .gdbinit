set $lastcs = -1

define hook-stop
  set architecture i386:x86-64
  set $lastcs = $cs
end

echo + target remote localhost:1234\n
target remote localhost:1234

echo + symbol-file build/bootloader.elf\n
symbol-file build/bootloader.elf

echo + symbol-file build/kernel.elf\n
symbol-file build/kernel.elf