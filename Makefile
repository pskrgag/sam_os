RUSTFLAGS += -C link-arg=--script=src/kernel/src/arch/aarch64/aarch64-qemu.ld
RUSTFLAGS += -C opt-level=0
RUSTFLAGS += -C force-frame-pointers

TARGET = aarch64-unknown-none-softfloat
BINARY = target/$(TARGET)/debug/sam_kernel

.PHONY: readelf clean all qemu qemu_gdb init kernel ridl

all: kernel

ridl:
	cargo build -p ridll

	cargo run -p ridl transport src/userspace/interfaces/idls/nameserver.idl > src/userspace/interfaces/src/client/nameserver.rs
	cargo run -p ridl server src/userspace/interfaces/idls/nameserver.idl > src/userspace/interfaces/src/server/nameserver.rs

	cargo run -p ridl transport src/userspace/interfaces/idls/serial.idl > src/userspace/interfaces/src/client/serial.rs
	cargo run -p ridl server src/userspace/interfaces/idls/serial.idl > src/userspace/interfaces/src/server/serial.rs

app: ridl
	cargo build -p console --target $(TARGET)

serial: app
	cargo build --target $(TARGET) -p serial

init: serial
	find ./target -name console -o -name serial | cpio -ocv > /tmp/archive.cpio
	cargo build -p nameserver --target $(TARGET)

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

