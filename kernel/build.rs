extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/arch/aarch64/mm/higher_half.c")
        .compiler("clang")
        .flag("--target=aarch64-unknown-none-softfloat")
        .flag("-fPIC")
        .flag("-O3")
        .flag("-Wall")
        .compile("libfoo.a");
}
