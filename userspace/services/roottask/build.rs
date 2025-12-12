fn main() {
    ridl::generate_server("../../idls/nameserver.ridl", "hello.rs", false).unwrap();
}
