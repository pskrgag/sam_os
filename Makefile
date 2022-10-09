RUSTFLAGS =  -C link-arg=--library-path=.
RUSTFLAGS += -C link-arg=--script=aarch64-qemu.ld
RUSTFLAGS += -C relocation-model=pic

all:
	RUSTFLAGS="$(RUSTFLAGS)" cargo build --target aarch64-unknown-none-softfloat

qemu:
	qemu-system-aarch64 -machine virt -m 2048M -cpu cortex-a53 -nographic -kernel target/aarch64-unknown-none-softfloat/debug/rust_os

qemu_gdb:
	qemu-system-aarch64 -machine virt -m 2048M -cpu cortex-a53 -nographic -kernel target/aarch64-unknown-none-softfloat/debug/rust_os -s -S
