.PHONY: readelf clean qemu qemu_gdb

SCRIPTS := $(wildcard build_scripts/*.toml)
TARGETS := $(patsubst %.toml,%, $(SCRIPTS))
BINARY = ./target/aarch64-unknown-none-softfloat/debug/sam_kernel


roottask:
	BOARD_TYPE=qemu cargo build -p roottask --target aarch64-unknown-none-softfloat

test_kernel: roottask
	BOARD_TYPE=qemu cargo test -p sam_kernel --target aarch64-unknown-none-softfloat --features qemu

run: roottask
	BOARD_TYPE=qemu cargo run -p sam_kernel --target aarch64-unknown-none-softfloat --features qemu

clippy:
	BOARD_TYPE=qemu cargo clippy -p sam_kernel --target aarch64-unknown-none-softfloat --features qemu

qemu:
	qemu-system-aarch64 -d mmu,guest_errors -D test.txt -machine virt,gic-version=2 -m 2048M -cpu cortex-a53 -smp 2 -nographic -kernel $(BINARY) -d int,mmu

qemu_gdb:
	qemu-system-aarch64 -machine virt -m 2048M -cpu cortex-a53 -smp 2 -nographic -kernel $(BINARY) -s -S

clean:
	cargo clean

readelf: all
	aarch64-linux-gnu-readelf --headers $(BINARY)

objdump:
	aarch64-linux-gnu-objdump -D $(BINARY) | less

