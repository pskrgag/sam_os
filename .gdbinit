set auto-load safe-path /

add-symbol-file target/aarch64-unknown-none-softfloat/debug/loader -o 0x0000000040200000
add-symbol-file target/aarch64-unknown-none-softfloat/debug/sam_kernel
# add-symbol-file ./target/aarch64-unknown-none-softfloat/debug/roottask
# add-symbol-file target/aarch64-unknown-none-softfloat/debug/serial
target remote :1234
b *0x200
