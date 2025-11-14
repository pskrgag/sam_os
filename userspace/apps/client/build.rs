fn main() {
    ridl::generate_client("../../idls/nameserver.ridl", "hello.rs").unwrap();
}
