QEMU_INTERRUPTS=''
if [ $SHOW_INTERRUPTS == true ];
    then QEMU_INTERRUPTS='-d int,cpu_reset'
    else QEMU_INTERRUPTS=''
fi

if [ $DEBUG_MODE == "vga" ];
    then qemu-system-i386 -drive file=build/buzz.img,index=0,media=disk,format=raw -no-reboot -no-shutdown -m 512 $QEMU_INTERRUPTS;
    else qemu-system-i386 -drive file=build/buzz.img,index=0,media=disk,format=raw -no-reboot -no-shutdown -nographic -serial mon:stdio -m 512;
fi