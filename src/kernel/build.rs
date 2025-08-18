extern crate cc;

fn main() {
    println!("cargo::rerun-if-changed=src/arch/aarch64/mm/higher_half.c");
    println!("cargo::rerun-if-changed=src/kernel/src/arch/aarch64/aarch64-qemu.ld");
    println!(
        "cargo:rustc-link-arg-bin=sam_kernel=--script=src/kernel/src/arch/aarch64/aarch64-qemu.ld"
    );

    let flag = if env!("BOARD_TYPE") == "qemu" {
        "-DCONFIG_BOARD_QEMU"
    } else if env!("BOARD_TYPE") == "orpipc2" {
        "-DCONFIG_BOARD_ORPIPC2"
    } else {
        panic!("Unknown board");
    };

    cc::Build::new()
        .file("src/arch/aarch64/mm/higher_half.c")
        .compiler("clang")
        .flag("--target=aarch64")
        .flag("-fPIC")
        .flag("-O2")
        .flag("-fno-tree-vectorize")
        .flag("-Wall")
        .flag(flag)
        .compile("libboot.a");
}
