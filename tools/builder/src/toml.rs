use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BuildScript {
    pub name: String,
    pub component: Vec<Component>,
}

#[derive(Deserialize, Debug)]
pub struct Component {
    pub name: String,
    pub implements: Option<Vec<String>>,
    pub depends: Option<Vec<String>>,
}

pub fn process_toml(script: &str) -> Result<BuildScript, ()> {
    let config: BuildScript = toml::from_str(script).unwrap();

    debug!("{:?}", config);
    Ok(config)
}
