fn main() {
    ridl::generate_client("../../idls/nameserver.ridl", "hello.rs").unwrap();
    ridl::generate_client("../../idls/serial.ridl", "serial.rs").unwrap();
}
