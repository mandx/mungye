#[derive(Debug, Copy, Clone)]
pub(crate) enum ArrayMergeBehavior {
    Replace,
    Concat,
    // TODO: Add `Extend`: Like concat, but will remove duplicates.
    // Extend,
    // TODO: Add `Zip`: Merge together elements in the same indices.
    //       For arrays of unequal lengths, simply use the value
    //       that's already there.
    // Zip,
}

// TODO: Find better names for `current` and `next` variables/symbols.

pub(crate) trait DeepMerge {
    fn deep_merge(self, with: Self, array_merge: ArrayMergeBehavior) -> Self;
}

pub(crate) use crate::conversions::JsonValue;

impl DeepMerge for JsonValue {
    fn deep_merge(self, with: Self, array_merge: ArrayMergeBehavior) -> JsonValue {
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
                for key in with_obj.iter().map(|(k, _)| k) {
                    match with_obj.get(key) {
                        // If we found two objects, merge them together.
                        Some(Object(with_value)) => {
                            if let Some(Object(self_value)) = self_obj.remove(key) {
                                self_obj.insert(
                                    key,
                                    JsonValue(Object(self_value.clone()))
                                        .deep_merge(
                                            JsonValue(Object(with_value.clone())),
                                            array_merge,
                                        )
                                        .0,
                                );
                            } else {
                                self_obj.insert(key, Object(with_value.clone()));
                            }
                        }

                        // Otherwise, as long as the second value is present, it will always win
                        Some(with_value) => {
                            self_obj.insert(key, with_value.clone());
                        }

                        // Since we are iterating over the keys of the second
                        // object we should never get to this point
                        None => unreachable!(),
                    }
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
    use json;
    use test_case::test_case;

    #[test_case(r#"{"a":1}"#, r#"{"a":2}"#, r#"{"a":2}"#)]
    #[test_case(r#"{"z":[1]}"#, r#"{"z":2}"#, r#"{"z":2}"#)]
    #[test_case(r#"{"y":[1]}"#, r#"{"y":[2]}"#, r#"{"y":[2]}"#)]
    #[test_case(r#"{"a":1}"#, r#"{"b":2}"#, r#"{"a":1,"b":2}"#)]
    #[test_case(r#"{"a":1, "b":{"b1": 3}}"#, r#"{"b":2}"#, r#"{"a":1,"b":2}"#)]
    #[test_case(
        r#"{"a":1, "b":{"b1": 3}}"#,
        r#"{"b":{"b1": 4}}"#,
        r#"{"a":1,"b":{"b1": 4}}"#
    )]
    #[test_case(
        r#"{"a":1, "b":{"b1": 3}}"#,
        r#"{"b":{"b2": 5}}"#,
        r#"{"a":1,"b":{"b1": 3, "b2": 5}}"#
    )]
    fn test_json_merge(current: &str, next: &str, expected: &str) {
        assert_eq!(
            JsonValue(json::parse(current).unwrap())
                .deep_merge(
                    JsonValue(json::parse(next).unwrap()),
                    ArrayMergeBehavior::Replace
                )
                .0,
            json::parse(expected).unwrap()
        );
    }
}
