fn main() {
    ridl::generate_client("../../idls/nameserver.ridl", "nameserver.rs").unwrap();
}
