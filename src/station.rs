use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Station {
    pub name: String,
    pub path: String,
    pub frequency: f32,
}
