KERNEL_SIZE=$(stat -c %s build/kernel.bin)
KERNEL_SIZE_SECTORS=$(echo $KERNEL_SIZE | perl -nl -MPOSIX -e 'print ceil($_);')
echo $KERNEL_SIZE_SECTORS