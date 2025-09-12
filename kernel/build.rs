fn main() {
    println!("cargo::rerun-if-changed=src/arch/aarch64/mm/higher_half.c");
    println!("cargo::rerun-if-changed=src/arch/aarch64/aarch64-qemu.ld");
    println!(
        "cargo:rustc-link-arg-bin=sam_kernel=--script=kernel/src/arch/aarch64/aarch64-qemu.ld"
    );
}
