use serde::{de, Serialize};
use crate::common::{Error, Res};

pub fn stringify<T> (v: &T) -> Res<String>
where
    T: ?Sized + Serialize,
{
    serde_json::to_string(v).map_err(|_| Error::CastToJsonError)
}

pub fn parse<T>(s: &str) -> Res<T>
where
    T: de::DeserializeOwned,
{
    serde_json::from_str(s).map_err(|e| Error::ParseJsonError(e.to_string()))
}
