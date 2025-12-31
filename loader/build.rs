const CLANG_TARGET: &str = "--target=aarch64-unknown-none";

fn main() {
    println!("cargo::rerun-if-changed=src/arch/aarch64/aarch64-qemu.ld");
    println!("cargo::rerun-if-changed={}", env!("KERNEL_PATH"));
    println!("cargo:rustc-link-arg-bin=loader=--script=src/arch/aarch64/aarch64-qemu.ld");
    println!("cargo:rustc-link-arg=-z");
    println!("cargo:rustc-link-arg=nostart-stop-gc");
    println!("cargo:rustc-link-arg=--no-gc-sections");
    println!("cargo:rustc-link-arg=-pie");

    cc::Build::new()
        .compiler("clang")
        .flag(CLANG_TARGET)
        .flag("-fPIC")
        .file("src/arch/aarch64/boot.s")
        .compile("boot");
}
