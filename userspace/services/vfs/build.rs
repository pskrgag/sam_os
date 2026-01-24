fn main() {
    ridl::generate_client("../../idls/blkdev.ridl", "blkdev.rs").unwrap();
    ridl::generate_client("../../idls/nameserver.ridl", "nameserver.rs").unwrap();
    ridl::generate_server("../../idls/vfs.ridl", "vfs.rs").unwrap();
}
