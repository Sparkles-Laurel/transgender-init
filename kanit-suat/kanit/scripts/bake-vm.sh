#!/usr/bin/env bash

# Gets the vm itself to set init
# Used for CI as libguestfs doesn't work

###

former_pwd=$(pwd)
cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" || return

target=$1
image=$2

valid_targets=("debug" "release" "min")
is_valid=false

for v in "${valid_targets[@]}"; do
  [[ "$target" = "$v" ]] && is_valid=true
done

if [ "$is_valid" = "false" ]; then
  echo "invalid target '$target'"
  exit 1
fi

if [ ! -f "$former_pwd/$image" ]; then
  echo "'$image' not found"
  exit 1
fi

echo "creating share directory"

mkdir tmp.share

cp "$former_pwd/target/x86_64-unknown-linux-musl/$target/kanit-multicall" ./tmp.share

./bakery.exp "$former_pwd/$image"

rm -rf ./tmp.share
