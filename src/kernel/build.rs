extern crate cc;

fn main() {
    let features = ["qemu".to_uppercase(), "orpipc2".to_uppercase()];
    let mut flag = None;

    for i in features {
        if std::env::var(format!("CARGO_FEATURE_{}", i)).is_ok() {
            flag = Some(format!("-DCONFIG_BOARD_{}", i));
            break;
        }
    }

    cc::Build::new()
        .file("src/arch/aarch64/mm/higher_half.c")
        .compiler("clang")
        .flag("--target=aarch64-unknown-none-softfloat")
        .flag("-fPIC")
        .flag("-O2")
        .flag("-Wall")
        .flag(flag.unwrap().as_str())
        .compile("libfoo.a");
}
