fn main() {
    println!("cargo::rerun-if-changed=src/arch/aarch64/aarch64-qemu.ld");
    println!("cargo:rustc-link-arg-bin=kernel=--script=src/arch/aarch64/aarch64-qemu.ld");
}
