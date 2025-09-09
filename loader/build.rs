fn main() {
    println!("cargo::rerun-if-changed=loader/src/arch/aarch64-qemu.ld");
    println!("cargo:rustc-link-arg-bin=loader=--script=loader/src/arch/aarch64-qemu.ld");
    println!("cargo:rustc-link-arg=-z");
    println!("cargo:rustc-link-arg=nostart-stop-gc");
    println!("cargo:rustc-link-arg=--no-gc-sections");
    println!("cargo:rustc-link-arg=-pie");
}
