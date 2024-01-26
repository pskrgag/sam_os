set auto-load safe-path /

add-symbol-file target/aarch64-unknown-none-softfloat/debug/sam_kernel
# add-symbol-file target/aarch64-unknown-none-softfloat/debug/nameserver
# add-symbol-file target/aarch64-unknown-none-softfloat/debug/serial
add-symbol-file target/aarch64-unknown-none-softfloat/debug/console
target remote :1234
b *0x200
