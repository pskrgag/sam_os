use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BuildScript {
    pub name: String,
    pub board: String,
    pub component: Vec<Component>,
}

#[derive(Deserialize, Debug)]
pub struct Component {
    pub name: String,
}
