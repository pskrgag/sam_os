fn main() {
    ridl::generate_client("../../idls/test.ridl", "hello.rs").unwrap();
}
