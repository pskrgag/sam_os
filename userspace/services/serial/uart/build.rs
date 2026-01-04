fn main() {
    ridl::generate_server("../../../idls/serial.ridl", "serial.rs").unwrap();
    ridl::generate_client("../../../idls/nameserver.ridl", "nameserver.rs").unwrap();
}
