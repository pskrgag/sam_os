fn main() {
    ridl::generate_server("../../idls/test.ridl", "hello.rs").unwrap();
}
