use alloc::string::{String, ToString};
use rtl::error::ErrorType;

/// Current working directory
pub struct Cwd {
    name: String,
}

impl Cwd {
    pub async fn root() -> Result<Self, ErrorType> {
        Ok(Self {
            name: "/".to_string(),
        })
    }

    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
