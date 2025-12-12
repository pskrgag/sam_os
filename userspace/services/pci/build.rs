fn main() {
    ridl::generate_client("../../idls/nameserver.ridl", "nameserver.rs").unwrap();
    ridl::generate_server("../../idls/pci.ridl", "pci.rs", true).unwrap();
}
