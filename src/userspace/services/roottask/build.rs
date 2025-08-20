fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    println!("OUT_DIR: {}", out_dir);

    ridl::generate_client("../../../../test.ridl", "hello.rs").unwrap();
}
