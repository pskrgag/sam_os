fn main() {
    ridl::generate_client("../../idls/vfs.ridl", "vfs.rs").unwrap();
}
