gdb:
	qemu-system-mipsel \
	-cpu 24Kc \
	-m 64 \
	-nographic -M malta \
	-no-reboot \
	-kernel target/mipsel-unknown-none/debug/kernel \
	-gdb tcp::1234 \
	-S