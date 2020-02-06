mod conversions;
mod merging;

use std::error::Error;
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::PathBuf;

use structopt::StructOpt;
// use toml;
use json;
use yaml_rust as yaml;

#[derive(Debug)]
pub(crate) enum Document {
    YAML(Vec<yaml::Yaml>),
    // TOML(Vec<toml::Value>),
    JSON(Vec<json::JsonValue>),
}

#[derive(Debug)]
pub(crate) enum DocumentError {
    Skipped { filename: PathBuf },
    Loading { filename: PathBuf, error: Box<dyn Error> },
}

impl Document {
    pub fn merge(self, with: Self) -> Self {
        todo!()
    }
}

/// Command-line arguments for this tool
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct CliArgs {
    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    let cli_args = CliArgs::from_args();

    let mut documents = cli_args
        .files
        .into_iter()
        .filter_map(|filename| {
            let contents = match read_to_string(&filename) {
                Ok(contents) => contents,
                Err(error) => return Some(Err(DocumentError::Loading { filename, error: Box::new(error) })),
            };

            match filename.extension().and_then(OsStr::to_str) {
                Some("yml") | Some("yaml") => match yaml::YamlLoader::load_from_str(&contents) {
                    Ok(loaded) => Some(Ok(Document::YAML(loaded))),
                    Err(error) => Some(Err(DocumentError::Loading { filename, error: Box::new(error) })),
                },
                Some("json") => match json::parse(&contents) {
                    Ok(loaded) => Some(Ok(Document::JSON(vec![loaded]))),
                    Err(error) => Some(Err(DocumentError::Loading { filename, error: Box::new(error) })),
                },
                Some(_) => Some(Err(DocumentError::Skipped { filename })),
                None => None,
            }
        })
        .filter(|loaded| {
            match loaded {
                Err(DocumentError::Skipped { filename }) => {
                    eprintln!("Skipped {:?}", filename);
                    false
                },
                Err(DocumentError::Loading { filename, error }) => {
                    eprintln!("Error loading {:?}: {:?}", filename, error);
                    false
                }
                Ok(_) => true,
            }
        });

    let destination = match documents.next() {
        Some(loaded) => loaded,
        None => {
            eprintln!("Got no documents to work with!");
            std::process::exit(1);
        }
    };
}
