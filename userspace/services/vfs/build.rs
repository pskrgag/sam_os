fn main() {
    ridl::generate_client("../../idls/blkdev.ridl", "blkdev.rs").unwrap();
    ridl::generate_client("../../idls/nameserver.ridl", "nameserver.rs").unwrap();
}
