#!/usr/bin/env ash

mount /dev/sda3 /mnt
mount /dev/sda1 /mnt/boot

mount -t proc /proc /mnt/proc
mount --rbind /sys /mnt/sys
mount --rbind /dev /mnt/dev
