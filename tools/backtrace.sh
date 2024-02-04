#/usr/bin/bash

FILENAME=$1
BACKTRACE=$(cat $FILENAME | awk '{$1=$2=""; print $0}' | sed -n "/--- cut here ---/,//p" | tail -n +5)

while read -r line
do
  T=$(echo "$line" | grep -o "\[.*\]" | tr -d '[]' | xargs aarch64-linux-gnu-addr2line -e ./target/aarch64-unknown-none-softfloat/debug/sam_kernel)

  echo "$line		$T"
done <<< "$BACKTRACE"
