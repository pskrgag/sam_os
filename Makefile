RUSTFLAGS =  -C link-arg=--library-path=.
RUSTFLAGS += -C link-arg=--script=aarch64-qemu.ld
RUSTFLAGS += -C opt-level=0

TARGET = aarch64-unknown-none-softfloat
BINARY = target/$(TARGET)/release/rust_os

all:
	RUSTFLAGS="$(RUSTFLAGS)" cargo build --target $(TARGET) --release

test:
	RUSTFLAGS="$(RUSTFLAGS)" cargo test --target $(TARGET) -Zbuild-std

qemu:
	qemu-system-aarch64 -machine virt -m 2048M -cpu cortex-a53 -nographic -kernel $(BINARY)

qemu_gdb:
	qemu-system-aarch64 -machine virt -m 2048M -cpu cortex-a53 -nographic -kernel $(BINARY) -s -S

clean:
	cargo clean

readelf: all
	aarch64-linux-gnu-readelf --headers $(BINARY)

objdump:
	aarch64-linux-gnu-objdump -D $(BINARY) | less

.PHONY: test readelf clean all qemu qemu_gdb
