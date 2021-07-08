use std::error::Error;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use itertools::{EitherOrBoth, Itertools};
use strum_macros::{Display, EnumString, EnumVariantNames};

use crate::conversions::{JsonValue, YamlValue};
use crate::merging::{ArrayMergeBehavior, DeepMerge};

use json as jsonlib;
// use toml as tomllib;
use yaml_rust as yamllib;

#[derive(Debug, Clone, Copy)]
pub(crate) enum NamespaceWith {
    Path,
    Filename,
    Basename,
}

impl std::str::FromStr for NamespaceWith {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "path" => Ok(NamespaceWith::Path),
            "filename" => Ok(NamespaceWith::Filename),
            "basename" => Ok(NamespaceWith::Basename),
            _ => Err("Invalid wrap with option"),
        }
    }
}

impl NamespaceWith {
    pub(crate) fn wrap<P: AsRef<Path>>(&self, document: Document, path: P) -> Document {
        let namespace: String = match self {
            NamespaceWith::Path => path.as_ref().to_string_lossy().into(),
            NamespaceWith::Filename => {
                let path_ref = path.as_ref();
                path_ref
                    .file_name()
                    .unwrap_or_else(|| path_ref.as_os_str())
                    .to_string_lossy()
                    .into()
            }
            NamespaceWith::Basename => {
                let mut path = path.as_ref().to_owned();
                if path.set_extension("") {
                    path.file_name()
                        .unwrap_or_else(|| path.as_os_str())
                        .to_string_lossy()
                        .into()
                } else {
                    path.as_os_str().to_string_lossy().into()
                }
            }
        };

        match document {
            Document::Yaml(yaml_doc) => {
                let mut namespace_hash = yamllib::yaml::Hash::new();
                namespace_hash.insert(
                    yamllib::Yaml::String(namespace),
                    match &yaml_doc[..] {
                        [] => yamllib::Yaml::Null,
                        [v] => v.clone(),
                        _ => yamllib::Yaml::Array(yaml_doc),
                    },
                );
                Document::Yaml(vec![yamllib::Yaml::Hash(namespace_hash)])
            }
            Document::Json(json_value) => {
                // todo!()
                let mut namespace_obj = jsonlib::object::Object::new();
                namespace_obj.insert(
                    &namespace,
                    match &json_value[..] {
                        [] => jsonlib::JsonValue::Null,
                        [v] => v.clone(),
                        _ => jsonlib::JsonValue::Array(json_value),
                    },
                );
                Document::Json(vec![jsonlib::JsonValue::Object(namespace_obj)])
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString, EnumVariantNames)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum DocumentType {
    Yaml,
    // TOML,
    Json,
}

impl DocumentType {
    pub fn default_document(self) -> Document {
        match self {
            Self::Yaml => Document::Yaml(vec![YamlValue::default().0]),
            Self::Json => Document::Json(vec![JsonValue::default().0]),
        }
    }

    pub fn load_from_path<P: AsRef<Path>>(self, filename: P) -> Result<Document, DocumentError> {
        let content = match read_to_string(filename.as_ref()) {
            Ok(contents) => contents,
            Err(error) => {
                return Err(DocumentError::Loading {
                    filename: filename.as_ref().into(),
                    error: Box::new(error),
                });
            }
        };

        match self {
            Self::Yaml => yamllib::YamlLoader::load_from_str(content.as_ref())
                .map(Document::Yaml)
                .map_err(|error| DocumentError::Loading {
                    filename: filename.as_ref().into(),
                    error: Box::new(error),
                }),
            Self::Json => jsonlib::parse(content.as_ref())
                .map(|loaded| Document::Json(vec![loaded]))
                .map_err(|error| DocumentError::Loading {
                    filename: filename.as_ref().into(),
                    error: Box::new(error),
                }),
        }
    }

    pub fn load_from_str<S: AsRef<str>, P: AsRef<Path>>(
        self,
        content: S,
        filename: P,
    ) -> Result<Document, DocumentError> {
        match self {
            Self::Yaml => yamllib::YamlLoader::load_from_str(content.as_ref())
                .map(Document::Yaml)
                .map_err(|error| DocumentError::Loading {
                    filename: filename.as_ref().into(),
                    error: Box::new(error),
                }),
            Self::Json => jsonlib::parse(content.as_ref())
                .map(|loaded| Document::Json(vec![loaded]))
                .map_err(|error| DocumentError::Loading {
                    filename: filename.as_ref().into(),
                    error: Box::new(error),
                }),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Document {
    Yaml(Vec<yamllib::Yaml>),
    // TOML(Vec<tomllib::Value>),
    Json(Vec<jsonlib::JsonValue>),
}

#[derive(Debug)]
pub(crate) enum DocumentError {
    Skipped {
        filename: PathBuf,
    },
    Loading {
        filename: PathBuf,
        error: Box<dyn Error>,
    },
}

impl Document {
    pub fn deep_merge(self, with: Self, array_merge: ArrayMergeBehavior) -> Self {
        match (self, with) {
            (Self::Yaml(left), Self::Yaml(right)) => Self::Yaml(
                left.into_iter()
                    .zip_longest(right.into_iter())
                    .map(|zipped| match zipped {
                        EitherOrBoth::Both(left, right) => {
                            YamlValue(left).deep_merge(YamlValue(right), array_merge).0
                        }
                        EitherOrBoth::Left(left) => left,
                        EitherOrBoth::Right(right) => right,
                    })
                    .collect(),
            ),
            (Self::Json(left), Self::Json(right)) => Self::Json(
                left.into_iter()
                    .zip_longest(right.into_iter())
                    .map(|zipped| match zipped {
                        EitherOrBoth::Both(left, right) => {
                            JsonValue(left).deep_merge(JsonValue(right), array_merge).0
                        }
                        EitherOrBoth::Left(left) => left,
                        EitherOrBoth::Right(right) => right,
                    })
                    .collect(),
            ),
            (Self::Json(left), Self::Yaml(right)) => Self::Json(
                left.into_iter()
                    .zip_longest(
                        right
                            .into_iter()
                            .map(|yml| JsonValue::from(YamlValue(yml)).0),
                    )
                    .map(|zipped| match zipped {
                        EitherOrBoth::Both(left, right) => {
                            JsonValue(left).deep_merge(JsonValue(right), array_merge).0
                        }
                        EitherOrBoth::Left(left) => left,
                        EitherOrBoth::Right(right) => right,
                    })
                    .collect(),
            ),
            (Self::Yaml(left), Self::Json(right)) => Self::Yaml(
                left.into_iter()
                    .zip_longest(
                        right
                            .into_iter()
                            .map(|yml| YamlValue::from(JsonValue(yml)).0),
                    )
                    .map(|zipped| match zipped {
                        EitherOrBoth::Both(left, right) => {
                            YamlValue(left).deep_merge(YamlValue(right), array_merge).0
                        }
                        EitherOrBoth::Left(left) => left,
                        EitherOrBoth::Right(right) => right,
                    })
                    .collect(),
            ),
        }
    }
}
