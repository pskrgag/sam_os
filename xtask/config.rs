use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct BuildScript {
    pub name: String,
    pub board: String,
    pub component: Vec<Component>,
    pub extra_qemu_args: Option<String>,
    pub opt_level: Option<String>,
}

#[derive(Deserialize, Debug, Default, Serialize)]
pub struct Component {
    pub name: String,
    pub env: Option<String>,
}
