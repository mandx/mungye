use std::error::Error;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use itertools::{EitherOrBoth, Itertools};

use crate::conversions::{JsonValue, YamlValue};
use crate::merging::{ArrayMergeBehavior, DeepMerge};

use json as jsonlib;
// use toml as tomllib;
use yaml_rust as yamllib;

#[derive(Debug, Clone, Copy)]
pub(crate) enum DocumentType {
    YAML,
    // TOML,
    JSON,
}

impl std::str::FromStr for DocumentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "yaml" | "yml" => Ok(Self::YAML),
            "json" => Ok(Self::JSON),
            // "toml" => Ok(Self::TOML),
            s => Err(format!("`{}` is not recognized as document typd", s)),
        }
    }
}

impl DocumentType {
    pub fn default_document(&self) -> Document {
        match self {
            Self::YAML => Document::YAML(vec![YamlValue::default().0]),
            Self::JSON => Document::JSON(vec![JsonValue::default().0]),
        }
    }

    pub fn load_from_path<P: AsRef<Path>>(&self, filename: P) -> Result<Document, DocumentError> {
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
            Self::YAML => yamllib::YamlLoader::load_from_str(content.as_ref())
                .map(Document::YAML)
                .map_err(|error| DocumentError::Loading {
                    filename: filename.as_ref().into(),
                    error: Box::new(error),
                }),
            Self::JSON => jsonlib::parse(content.as_ref())
                .map(|loaded| Document::JSON(vec![loaded]))
                .map_err(|error| DocumentError::Loading {
                    filename: filename.as_ref().into(),
                    error: Box::new(error),
                }),
        }
    }

    pub fn load_from_str<S: AsRef<str>, P: AsRef<Path>>(
        &self,
        content: S,
        filename: P,
    ) -> Result<Document, DocumentError> {
        match self {
            Self::YAML => yamllib::YamlLoader::load_from_str(content.as_ref())
                .map(Document::YAML)
                .map_err(|error| DocumentError::Loading {
                    filename: filename.as_ref().into(),
                    error: Box::new(error),
                }),
            Self::JSON => jsonlib::parse(content.as_ref())
                .map(|loaded| Document::JSON(vec![loaded]))
                .map_err(|error| DocumentError::Loading {
                    filename: filename.as_ref().into(),
                    error: Box::new(error),
                }),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Document {
    YAML(Vec<yamllib::Yaml>),
    // TOML(Vec<tomllib::Value>),
    JSON(Vec<jsonlib::JsonValue>),
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
            (Self::YAML(left), Self::YAML(right)) => Self::YAML(
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
            (Self::JSON(left), Self::JSON(right)) => Self::JSON(
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
            (Self::JSON(left), Self::YAML(right)) => Self::JSON(
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
            (Self::YAML(left), Self::JSON(right)) => Self::YAML(
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
