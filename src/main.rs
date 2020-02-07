mod conversions;
mod merging;

use std::error::Error;
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::PathBuf;

use structopt::StructOpt;
// use toml;
use itertools::{EitherOrBoth, Itertools};
use json;
use yaml_rust as yaml;

use conversions::{JsonValue, YamlValue};
use merging::{ArrayMergeBehavior, DeepMerge};

#[derive(Debug)]
pub(crate) enum Document {
    YAML(Vec<yaml::Yaml>),
    // TOML(Vec<toml::Value>),
    JSON(Vec<json::JsonValue>),
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

/// Command-line arguments for this tool
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct CliArgs {
    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,

    #[structopt(long = "arrays")]
    array_merge: ArrayMergeBehavior,
}

fn main() {
    let CliArgs {
        files: filenames,
        array_merge,
    } = CliArgs::from_args();

    let mut documents = filenames
        .into_iter()
        .filter_map(|filename| {
            let contents = match read_to_string(&filename) {
                Ok(contents) => contents,
                Err(error) => {
                    return Some(Err(DocumentError::Loading {
                        filename,
                        error: Box::new(error),
                    }))
                }
            };

            match filename.extension().and_then(OsStr::to_str) {
                Some("yml") | Some("yaml") => match yaml::YamlLoader::load_from_str(&contents) {
                    Ok(loaded) => Some(Ok(Document::YAML(loaded))),
                    Err(error) => Some(Err(DocumentError::Loading {
                        filename,
                        error: Box::new(error),
                    })),
                },
                Some("json") => match json::parse(&contents) {
                    Ok(loaded) => Some(Ok(Document::JSON(vec![loaded]))),
                    Err(error) => Some(Err(DocumentError::Loading {
                        filename,
                        error: Box::new(error),
                    })),
                },
                Some(_) => Some(Err(DocumentError::Skipped { filename })),
                None => None,
            }
        })
        .filter_map(|loaded| match loaded {
            Err(DocumentError::Skipped { filename }) => {
                eprintln!("Skipped {:?}", filename);
                None
            }
            Err(DocumentError::Loading { filename, error }) => {
                eprintln!("Error loading {:?}: {:?}", filename, error);
                None
            }
            Ok(document) => Some(document),
        });

    let destination = match documents.next() {
        Some(loaded) => loaded,
        None => {
            eprintln!("Got no documents to work with!");
            std::process::exit(1);
        }
    };

    let result = documents.fold(destination, |destination, document| {
        destination.deep_merge(document, array_merge)
    });

    println!(
        "{}",
        match result {
            Document::JSON(json) => json
                .into_iter()
                .map(|doc| doc.pretty(2))
                .collect::<String>(),
            Document::YAML(yaml) => yaml
                .into_iter()
                .map(|doc| {
                    let mut out_str = String::new();
                    let mut emitter = yaml::YamlEmitter::new(&mut out_str);
                    emitter.dump(&doc).unwrap();
                    out_str
                })
                .collect(),
        }
    );
}
