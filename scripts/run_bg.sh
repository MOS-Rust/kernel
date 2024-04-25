#!/bin/sh
qemu-system-mipsel -cpu 4Kc -m 64 -nographic -M malta -no-reboot -kernel target/mipsel-unknown-none/debug/kernel -s -S < /dev/null > /dev/null &
echo "QEMU started"