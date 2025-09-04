#[derive(Clone)]
pub struct TestDescr {
    pub name: &'static str,
    pub module: &'static str,
    pub test_fn: fn(),
}
