// use toml;
use json as jsonlib;
use yaml_rust as yamllib;

#[derive(Debug, PartialEq)]
pub(crate) struct YamlValue(pub yamllib::Yaml);
#[derive(Debug, PartialEq)]
pub(crate) struct JsonValue(pub jsonlib::JsonValue);
// pub(crate) struct TomlValue(toml::Value);

pub(crate) struct JsonObject(jsonlib::object::Object);
pub(crate) struct YamlHash(yamllib::yaml::Hash);

impl From<yamllib::Yaml> for YamlValue {
    fn from(value: yamllib::Yaml) -> Self {
        Self(value)
    }
}

impl Default for YamlValue {
    fn default() -> Self {
        Self(yamllib::Yaml::Null)
    }
}

impl From<jsonlib::JsonValue> for JsonValue {
    fn from(value: jsonlib::JsonValue) -> Self {
        Self(value)
    }
}

impl Default for JsonValue {
    fn default() -> Self {
        Self(jsonlib::JsonValue::Null)
    }
}

impl From<JsonObject> for YamlHash {
    fn from(JsonObject(value): JsonObject) -> Self {
        Self(
            value
                .iter()
                .map(|(key, value)| {
                    (
                        yamllib::Yaml::String(key.into()),
                        YamlValue::from(JsonValue(value.clone())).0,
                    )
                })
                .collect::<yamllib::yaml::Hash>(),
        )
    }
}

impl From<JsonValue> for YamlValue {
    fn from(JsonValue(value): JsonValue) -> Self {
        YamlValue(match value {
            jsonlib::JsonValue::Null => yamllib::Yaml::Null,
            jsonlib::JsonValue::Short(value) => yamllib::Yaml::String(value.into()),

            // TODO: Maybe figure out a way of not having to format a JSON number to string?
            jsonlib::JsonValue::Number(value) => yamllib::Yaml::Real(value.to_string()),
            jsonlib::JsonValue::String(value) => yamllib::Yaml::String(value),
            jsonlib::JsonValue::Boolean(value) => yamllib::Yaml::Boolean(value),
            jsonlib::JsonValue::Object(value) => {
                yamllib::Yaml::Hash(YamlHash::from(JsonObject(value)).0)
            }
            jsonlib::JsonValue::Array(values) => yamllib::Yaml::Array(
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
        use yamllib::Yaml::*;
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
                .collect::<jsonlib::object::Object>(),
        )
    }
}

impl From<YamlValue> for JsonValue {
    fn from(YamlValue(value): YamlValue) -> Self {
        JsonValue(match value {
            yamllib::Yaml::Real(value) => {
                match value.parse::<f64>() {
                    Ok(parsed) => jsonlib::JsonValue::Number(parsed.into()),
                    // In the rare case we can't parse the number, we just
                    // leave the original value as a string.
                    Err(_) => jsonlib::JsonValue::String(value),
                }
            }
            yamllib::Yaml::Integer(value) => jsonlib::JsonValue::Number(value.into()),
            yamllib::Yaml::String(value) => jsonlib::JsonValue::String(value),
            yamllib::Yaml::Boolean(value) => jsonlib::JsonValue::Boolean(value),
            yamllib::Yaml::Null => jsonlib::JsonValue::Null,
            yamllib::Yaml::Hash(hash) => {
                jsonlib::JsonValue::Object(JsonObject::from(YamlHash(hash)).0)
            }
            yamllib::Yaml::Array(values) => jsonlib::JsonValue::Array(
                values
                    .into_iter()
                    .map(|value| JsonValue::from(YamlValue(value)).0)
                    .collect(),
            ),

            yamllib::Yaml::Alias(_) => panic!("`Yaml::Alias` is not yer supported"),
            yamllib::Yaml::BadValue => panic!("`Yaml::BadValue` can not be converted"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        assert_eq!(YamlValue::default(), YamlValue(yamllib::Yaml::Null));
        assert_eq!(JsonValue::default(), JsonValue(jsonlib::JsonValue::Null));
    }
}
