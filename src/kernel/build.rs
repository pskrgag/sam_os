extern crate cc;

fn main() {
    let mut flag = None;
    let c_path = "src/arch/aarch64/mm/higher_half.c";

    if env!("BOARD_TYPE") == "qemu" {
        flag = Some("-DCONFIG_BOARD_QEMU");
    } else if env!("BOARD_TYPE") == "orpipc2" {
        flag = Some("-DCONFIG_BOARD_ORPIPC2");
    }

    println!(
        "cargo:rustc-link-arg-bin=sam_kernel=--script=src/kernel/src/arch/aarch64/aarch64-qemu.ld"
    );
    println!("cargo:rerun-if-changed={c_path}");

    cc::Build::new()
        .file(c_path)
        .compiler("clang")
        .flag("--target=aarch64")
        .flag("-fPIC")
        .flag("-O2")
        .flag("-Wall")
        .flag(flag.unwrap())
        .compile("libfoo.a");
}
