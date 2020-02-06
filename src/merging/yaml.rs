use crate::merging::{ArrayMergeBehavior, DeepMerge};

pub(crate) use crate::conversions::YamlValue;

impl DeepMerge for YamlValue {
    fn deep_merge(self, with: Self, array_merge: ArrayMergeBehavior) -> Self {
        use yaml_rust::Yaml::*;

        YamlValue(match (self.0, with.0) {
            (Array(mut self_values), Array(with_values)) => match array_merge {
                ArrayMergeBehavior::Replace => Array(with_values),
                ArrayMergeBehavior::Concat => {
                    self_values.extend(with_values);
                    Array(self_values)
                }
            },
            (Hash(mut self_hash), Hash(with_hash)) => {
                for (key, with_value) in with_hash.iter().map(|(k, v)| (k.clone(), v.clone())) {
                    let original_value = self_hash.remove(&key);
                    self_hash.insert(
                        key,
                        match original_value {
                            Some(self_value) => {
                                YamlValue(self_value)
                                    .deep_merge(YamlValue(with_value), array_merge)
                                    .0
                            }
                            None => with_value,
                        },
                    );
                }

                Hash(self_hash)
            }
            (_, with) => with,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    use yaml_rust as yaml;

    #[test_case(
        r#"
a:
  foo: bar
"#,
        r#"
b:
  foo: bar
"#,
        r#"
a:
  foo: bar
b:
  foo: bar
"#,
        ArrayMergeBehavior::Replace
    )]
    #[test_case(
        r#"
key:
  first_value: a
"#,
        r#"
key:
  second_value: b
"#,
        r#"
key:
  first_value: a
  second_value: b
"#,
        ArrayMergeBehavior::Replace
    )]
    #[test_case(
        r#"
key:
  value: a
"#,
        r#"
key:
  value: b
"#,
        r#"
key:
  value: b
"#,
        ArrayMergeBehavior::Replace
    )]
    fn test_yaml_merge(current: &str, next: &str, expected: &str, array_merge: ArrayMergeBehavior) {
        let current_docs = yaml::YamlLoader::load_from_str(current).unwrap();
        let next_docs = yaml::YamlLoader::load_from_str(next).unwrap();
        let expected_docs = yaml::YamlLoader::load_from_str(expected).unwrap();

        assert_eq!(
            YamlValue(current_docs[0].clone())
                .deep_merge(YamlValue(next_docs[0].clone()), array_merge)
                .0,
            expected_docs[0]
        );
    }
}
