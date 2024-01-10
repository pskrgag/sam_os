#[derive(Clone)]
pub struct Type {
    name: String,
    is_builtin: bool
}

impl Type {
    pub fn new(name: String) -> Self {
        Self { name, is_builtin: false }
    }
}
