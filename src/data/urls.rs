use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Urls {
    #[serde(rename = "odata.metadata")]
    pub metadata_url: String,

    #[serde(rename = "value")]
    pub data: Vec<Value>,
}

#[derive(Debug, Deserialize)]
pub struct Value {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "url")]
    pub url: String,
}
