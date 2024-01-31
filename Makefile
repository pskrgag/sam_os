RUSTFLAGS += -C link-arg=--script=src/kernel/src/arch/aarch64/aarch64-qemu.ld
RUSTFLAGS += -C opt-level=0
RUSTFLAGS += -C force-frame-pointers

BINARY = ./target/aarch64-unknown-none-softfloat/debug/sam_kernel

.PHONY: readelf clean qemu qemu_gdb

SCRIPTS := $(wildcard build_scripts/*.toml)
TARGETS := $(patsubst %.toml,%, $(SCRIPTS))

%:
	cargo run -p builder build_scripts/$@.toml test

qemu:
	qemu-system-aarch64 -d mmu,guest_errors -D test.txt -machine virt,gic-version=2 -m 2048M -cpu cortex-a53 -smp 2 -nographic -kernel $(BINARY)

qemu_gdb:
	qemu-system-aarch64 -machine virt -m 2048M -cpu cortex-a53 -smp 2 -nographic -kernel $(BINARY) -s -S

clean:
	cargo clean

readelf: all
	aarch64-linux-gnu-readelf --headers $(BINARY)

objdump:
	aarch64-linux-gnu-objdump -D $(BINARY) | less

