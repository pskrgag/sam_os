extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/arch/aarch64/mm/higher_half.c")
        .compiler("clang-14")
        .flag("--target=aarch64-unknown-none-softfloat")
        .flag("-fPIC")
        .flag("-O1")
        .flag("-Wall")
        .compile("libfoo.a");
}
