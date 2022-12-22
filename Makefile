RUSTFLAGS =  -C link-arg=--library-path=.
RUSTFLAGS += -C link-arg=--script=aarch64-qemu.ld
RUSTFLAGS += -C opt-level=0
RUSTFLAGS += -C force-frame-pointers

TARGET = aarch64-unknown-none-softfloat
BINARY = target/$(TARGET)/debug/sam_os

all:
	RUSTFLAGS="$(RUSTFLAGS)" cargo build --target $(TARGET)

test:
	RUSTFLAGS="$(RUSTFLAGS)" cargo test --target $(TARGET) -Zbuild-std

qemu:
	qemu-system-aarch64 -d mmu,guest_errors -D test.txt -machine virt,gic-version=2 -m 2048M -cpu cortex-a53 -nographic -kernel $(BINARY)

qemu_gdb:
	~/Documents/kernel_workspace/qemu/build/qemu-system-aarch64 -machine virt -m 2048M -cpu cortex-a53 -nographic -kernel $(BINARY) -s -S

clean:
	cargo clean

readelf: all
	aarch64-linux-gnu-readelf --headers $(BINARY)

objdump:
	aarch64-linux-gnu-objdump -D $(BINARY) | less

.PHONY: test readelf clean all qemu qemu_gdb
