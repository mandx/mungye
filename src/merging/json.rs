use crate::merging::{ArrayMergeBehavior, DeepMerge};

pub(crate) use crate::conversions::JsonValue;

impl DeepMerge for JsonValue {
    fn deep_merge(self, with: Self, array_merge: ArrayMergeBehavior) -> Self {
        use json::JsonValue::*;
        JsonValue(match (self.0, with.0) {
            (Array(mut self_values), Array(with_values)) => match array_merge {
                ArrayMergeBehavior::Replace => Array(with_values),
                ArrayMergeBehavior::Concat => {
                    self_values.extend(with_values);
                    Array(self_values)
                }
            },
            (Object(mut self_obj), Object(with_obj)) => {
                for (key, with_value) in with_obj.iter().map(|(k, v)| (k, v.clone())) {
                    let original_value = self_obj.remove(key);
                    self_obj.insert(
                        key,
                        match original_value {
                            Some(self_value) => {
                                JsonValue(self_value)
                                    .deep_merge(JsonValue(with_value), array_merge)
                                    .0
                            }
                            None => with_value,
                        },
                    )
                }
                Object(self_obj)
            }
            (_, with) => with,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(r#"{"a":1}"#, r#"{"a":2}"#, r#"{"a":2}"#, ArrayMergeBehavior::Replace)]
    #[test_case(
        r#"{"z":[1]}"#,
        r#"{"z":2}"#,
        r#"{"z":2}"#,
        ArrayMergeBehavior::Replace
    )]
    #[test_case(
        r#"{"y":[1]}"#,
        r#"{"y":[2]}"#,
        r#"{"y":[2]}"#,
        ArrayMergeBehavior::Replace
    )]
    #[test_case(
        r#"{"a":1}"#,
        r#"{"b":2}"#,
        r#"{"a":1,"b":2}"#,
        ArrayMergeBehavior::Replace
    )]
    #[test_case(
        r#"{"a":1, "b":{"b1": 3}}"#,
        r#"{"b":2}"#,
        r#"{"a":1,"b":2}"#,
        ArrayMergeBehavior::Replace
    )]
    #[test_case(
        r#"{"a":1, "b":{"b1": 3}}"#,
        r#"{"b":{"b1": 4}}"#,
        r#"{"a":1,"b":{"b1": 4}}"#,
        ArrayMergeBehavior::Replace
    )]
    #[test_case(
        r#"{"a": 1, "b": {"b1": 3}}"#,
        r#"{        "b": {"b2": 5}}"#,
        r#"{"a": 1, "b": {"b1": 3, "b2": 5}}"#,
        ArrayMergeBehavior::Replace
    )]
    #[test_case(
        r#"{"a": 1, "b": {"b1": 3,          "b3": {"c1": 6          }}}"#,
        r#"{        "b": {         "b2": 5, "b3": {         "c2": 7 }}}"#,
        r#"{"a": 1, "b": {"b1": 3, "b2": 5, "b3": {"c1": 6, "c2": 7 }}}"#,
        ArrayMergeBehavior::Replace
    )]
    #[test_case(
        r#"{"z":[1]}"#,
        r#"{"z":[2]}"#,
        r#"{"z":[2]}"#,
        ArrayMergeBehavior::Replace;
        "meh"
    )]
    #[test_case(
        r#"{"z":[1]}"#,
        r#"{"z":[2]}"#,
        r#"{"z":[1, 2]}"#,
        ArrayMergeBehavior::Concat
    )]
    #[test_case(
        r#"{"z":[2]}"#,
        r#"{"z":[1]}"#,
        r#"{"z":[2, 1]}"#,
        ArrayMergeBehavior::Concat
    )]
    #[test_case(
        r#"{"z": {"z1": [2]}}"#,
        r#"{"z": {"z1": [1]}}"#,
        r#"{"z": {"z1": [2, 1]}}"#,
        ArrayMergeBehavior::Concat
    )]
    fn test_json_merge(current: &str, next: &str, expected: &str, array_merge: ArrayMergeBehavior) {
        assert_eq!(
            JsonValue(json::parse(current).unwrap())
                .deep_merge(JsonValue(json::parse(next).unwrap()), array_merge)
                .0,
            json::parse(expected).unwrap()
        );
    }
}
