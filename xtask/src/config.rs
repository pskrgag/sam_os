use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BuildScript {
    pub name: String,
    pub board: String,
    pub component: Vec<Component>,
    pub extra_qemu_args: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Component {
    pub name: String,
}
