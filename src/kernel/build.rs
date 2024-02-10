extern crate cc;

fn main() {
    let mut flag = None;

    if env!("BOARD_TYPE") == "qemu" {
        flag = Some("-DCONFIG_BOARD_QEMU");
    } else if env!("BOARD_TYPE") == "orpipc2" {
        flag = Some("-DCONFIG_BOARD_ORPIPC2");
    }

    cc::Build::new()
        .file("src/arch/aarch64/mm/higher_half.c")
        .compiler("clang")
        .flag("--target=aarch64-unknown-none-softfloat")
        .flag("-fPIC")
        .flag("-O2")
        .flag("-Wall")
        .flag(flag.unwrap())
        .compile("libfoo.a");
}
