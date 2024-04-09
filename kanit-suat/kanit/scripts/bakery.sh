#!/usr/bin/env ash

touch gay

mkdir share

mount -t 9p -o trans=virtio host0 share

cp ./share/kanit-multicall .
cp ./share/kanit-multicall /sbin/kanit-multicall

ln -sf /sbin/kanit-multicall /sbin/init
ln -sf /sbin/kanit-multicall /sbin/kanit
ln -sf /sbin/kanit-multicall /sbin/kanit-supervisor

umount share

rm -rf share
