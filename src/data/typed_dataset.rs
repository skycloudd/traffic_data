use super::simple;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TypedDataset {
    #[serde(rename = "odata.metadata")]
    pub metadata_url: String,

    #[serde(rename = "value")]
    pub data: Vec<Value>,
}

#[derive(Debug, Deserialize)]
pub struct Value {
    #[serde(rename = "ID")]
    pub id: usize,

    #[serde(rename = "Geslacht")]
    pub geslacht: String,

    #[serde(rename = "Persoonskenmerken")]
    pub persoonskenmerken: String,

    #[serde(rename = "Perioden")]
    pub perioden: String,

    #[serde(
        rename = "Verkeersdeelname_1",
        deserialize_with = "serialize_maybe_nan"
    )]
    pub verkeersdeelname: Option<f64>,

    #[serde(
        rename = "GebruikVanHetOpenbaarVervoer_2",
        deserialize_with = "serialize_maybe_nan"
    )]
    pub gebruik_openbaar_vervoer: Option<f64>,
}

fn serialize_maybe_nan<'de, D>(d: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    d.deserialize_option(MyVisitor)
}

struct MyVisitor;

impl<'de> serde::de::Visitor<'de> for MyVisitor {
    type Value = Option<f64>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a float")
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Some(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "NaN" => Ok(None),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(v),
                &self,
            )),
        }
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

#[derive(Debug)]
pub struct DataRow {
    pub id: usize,
    pub geslacht: simple::Value,
    pub persoonskenmerken: simple::Value,
    pub perioden: simple::Value,
    pub verkeersdeelname: Option<f64>,
    pub gebruik_openbaar_vervoer: Option<f64>,
}
