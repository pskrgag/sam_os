fn main() {
    ridl::generate_client("../../idls/nameserver.ridl", "nameserver.rs").unwrap();
    ridl::generate_client("../../idls/nic.ridl", "nic.rs").unwrap();
}

