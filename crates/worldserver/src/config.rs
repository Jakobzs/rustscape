use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub cache: Cache,
}

#[derive(Deserialize)]
pub struct Cache {
    pub path: String,
}
