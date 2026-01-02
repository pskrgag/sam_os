fn main() {
    ridl::generate_server("../../../idls/serial.ridl", "serial.rs", false).unwrap();
    ridl::generate_client("../../../idls/nameserver.ridl", "nameserver.rs").unwrap();
}
