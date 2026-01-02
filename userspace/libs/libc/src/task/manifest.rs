use heapless::String;
use serde::Deserialize;

type ComponentString = String<200>;

#[derive(Deserialize, Debug, Default)]
pub struct Manifest {
    pub name: ComponentString,
    pub env: Option<ComponentString>,
}
