RUSTFLAGS =  -C link-arg=--library-path=.
RUSTFLAGS += -C link-arg=--script=aarch64-qemu.ld
RUSTFLAGS += -C opt-level=0
RUSTFLAGS += -C force-frame-pointers

TARGET = aarch64-unknown-none-softfloat
BINARY = target/$(TARGET)/debug/sam_kernel

.PHONY: readelf clean all qemu qemu_gdb init kernel ridl

all: kernel

ridl:
	cargo build -p ridl
	cargo run -p ridl transport interface.idl > userspace/fileserver/src/transport.rs
	cargo run -p ridl server interface.idl > userspace/init/src/interface.rs

fileserver: ridl
	cargo build --target $(TARGET) -p fileserver
	find  target -name "fileserver" -print0 | cpio -ocv0  > /tmp/archive.cpio

init: fileserver
	cargo build -p sam_os_init --target $(TARGET)

kernel: init
	RUSTFLAGS="$(RUSTFLAGS)" cargo build --target $(TARGET) --features qemu -p sam_kernel

qemu: all
	qemu-system-aarch64 -d mmu,guest_errors -D test.txt -machine virt,gic-version=2 -m 2048M -cpu cortex-a53 -smp 2 -nographic -kernel $(BINARY)

qemu_gdb:
	qemu-system-aarch64 -machine virt -m 2048M -cpu cortex-a53 -smp 2 -nographic -kernel $(BINARY) -s -S

clean:
	cargo clean

readelf: all
	aarch64-linux-gnu-readelf --headers $(BINARY)

objdump:
	aarch64-linux-gnu-objdump -D $(BINARY) | less

