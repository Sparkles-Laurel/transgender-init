#!/usr/bin/env bash

# Initializes an Alpine Linux vm for testing
IMAGE_URL="https://dl-cdn.alpinelinux.org/alpine/v3.19/releases/x86_64/alpine-virt-3.19.0-x86_64.iso"
IMAGE=${IMAGE_URL##*/}

###

former_pwd=$(pwd)
cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" || return

download_image() {
  echo "downloading alpine image"

  [ ! -f "$IMAGE" ] && curl -o "$IMAGE" $IMAGE_URL
  [ ! -f "$IMAGE.sha256" ] && curl -o "$IMAGE.sha256" "$IMAGE_URL.sha256"

  sha256sum -c "$IMAGE.sha256" || return 1
}

create_vm() {
  echo "initializing alpine vm"

  [ ! -f "alpine.qcow2" ] && qemu-img create -f qcow2 alpine.qcow2 8G

  ./controller.exp $IMAGE

  if [ -f "alpine.qcow2" ]; then
    echo "generated alpine.qcow2"
    mv alpine.qcow2 "$former_pwd"
  fi
}

cleanup() {
  echo "cleaning up"

  rm "$IMAGE" "$IMAGE.sha256"
}

download_image || return 1
create_vm || return 1
cleanup || return 1
