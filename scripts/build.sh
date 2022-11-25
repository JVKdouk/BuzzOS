if [ $DEBUG_MODE == "vga" ];
    then qemu-system-x86_64 -drive file=build/buzz.img,index=0,media=disk,format=raw -no-reboot -no-shutdown -m 512;
    else qemu-system-x86_64 -s -S -drive file=build/buzz.img,index=0,media=disk,format=raw -no-reboot -no-shutdown -nographic -serial mon:stdio -m 512;
fi

# -d int,cpu_reset