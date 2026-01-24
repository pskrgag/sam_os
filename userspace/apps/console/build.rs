fn main() {
    ridl::generate_client("../../idls/nameserver.ridl", "nameserver.rs").unwrap();
    ridl::generate_client("../../idls/serial.ridl", "serial.rs").unwrap();
    ridl::generate_client("../../idls/vfs.ridl", "vfs.rs").unwrap();
}
