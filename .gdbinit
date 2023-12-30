set auto-load safe-path /

add-symbol-file target/aarch64-unknown-none-softfloat/debug/sam_kernel
add-symbol-file target/aarch64-unknown-none-softfloat/debug/init
add-symbol-file target/aarch64-unknown-none-softfloat/debug/fileserver
target remote :1234
b *0x200
