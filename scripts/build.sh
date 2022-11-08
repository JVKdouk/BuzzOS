if [ $DEBUG_MODE == "vga" ];
    then qemu-system-x86_64 -cdrom build/os.iso -no-reboot -no-shutdown -d int,cpu_reset;
    else qemu-system-x86_64 -cdrom build/os.iso -no-reboot -no-shutdown -nographic -serial mon:stdio;
fi
