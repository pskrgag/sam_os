fn main() {
    ridl::generate_client("../../idls/netstack.ridl", "netstack.rs").unwrap();
}
