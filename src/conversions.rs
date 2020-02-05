// use toml;
use json;
use yaml_rust as yaml;

// pub(crate) struct TomlValue(toml::Value);
pub(crate) struct YamlValue(pub yaml::Yaml);
pub(crate) struct JsonValue(pub json::JsonValue);

pub(crate) struct JsonObject(json::object::Object);
pub(crate) struct YamlHash(yaml::yaml::Hash);

impl From<JsonObject> for YamlHash {
    fn from(JsonObject(value): JsonObject) -> Self {
        Self(
            value
                .iter()
                .map(|(key, value)| {
                    (
                        yaml::yaml::Yaml::String(key.into()),
                        YamlValue::from(JsonValue(value.clone())).0,
                    )
                })
                .collect::<yaml::yaml::Hash>(),
        )
    }
}

impl From<JsonValue> for YamlValue {
    fn from(JsonValue(value): JsonValue) -> Self {
        YamlValue(match value {
            json::JsonValue::Null => yaml::Yaml::Null,
            json::JsonValue::Short(value) => yaml::Yaml::String(value.into()),
            json::JsonValue::Number(value) => yaml::Yaml::Integer(value.into()),
            json::JsonValue::String(value) => yaml::Yaml::String(value),
            json::JsonValue::Boolean(value) => yaml::Yaml::Boolean(value),
            json::JsonValue::Object(value) => yaml::Yaml::Hash(YamlHash::from(JsonObject(value)).0),
            json::JsonValue::Array(values) => yaml::Yaml::Array(
                values
                    .into_iter()
                    .map(|value| YamlValue::from(JsonValue(value)).0)
                    .collect(),
            ),
        })
    }
}

impl From<YamlValue> for String {
    fn from(YamlValue(value): YamlValue) -> Self {
        use yaml::yaml::Yaml::*;
        match value {
            Real(value) => value,
            Integer(value) => value.to_string(),
            String(value) => value,
            Boolean(value) => value.to_string(),
            Null => "null".into(),
            Alias(_) => panic!("Can't convert Yaml value `Alias` to String"),
            Array(_) => panic!("Can't convert Yaml value `Array` to String"),
            Hash(_) => panic!("Can't convert Yaml value `Hash` to String"),
            BadValue => panic!("Can't convert Yaml value `BadValue` to String"),
        }
    }
}

impl From<YamlHash> for JsonObject {
    fn from(YamlHash(value): YamlHash) -> Self {
        Self(
            value
                .iter()
                .map(|(key, value)| {
                    (
                        String::from(YamlValue(key.clone())),
                        JsonValue::from(YamlValue(value.clone())).0,
                    )
                })
                .collect::<json::object::Object>(),
        )
    }
}

impl From<YamlValue> for JsonValue {
    fn from(YamlValue(value): YamlValue) -> Self {
        JsonValue(match value {
            yaml::yaml::Yaml::Real(value) => {
                json::JsonValue::Number(value.parse::<f64>().unwrap().into())
            }
            yaml::yaml::Yaml::Integer(value) => json::JsonValue::Number(value.into()),
            yaml::yaml::Yaml::String(value) => json::JsonValue::String(value),
            yaml::yaml::Yaml::Boolean(value) => json::JsonValue::Boolean(value),
            yaml::yaml::Yaml::Null => json::JsonValue::Null,
            yaml::yaml::Yaml::Hash(hash) => {
                json::JsonValue::Object(JsonObject::from(YamlHash(hash)).0)
            }
            yaml::yaml::Yaml::Array(values) => json::JsonValue::Array(
                values
                    .into_iter()
                    .map(|value| JsonValue::from(YamlValue(value)).0)
                    .collect(),
            ),

            yaml::yaml::Yaml::Alias(_) => panic!("`Yaml::Alias` is not yer supported"),
            yaml::yaml::Yaml::BadValue => panic!("`Yaml::BadValue` can not be converted"),
        })
    }
}
