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
pub(crate) enum Loaded {
    Skipped(String),
    Error(Box<dyn Error>),
    YAML(Vec<yaml::Yaml>),
    // TOML(Vec<toml::Value>),
    JSON(Vec<json::JsonValue>),
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

    cli_args
        .files
        .iter()
        .filter_map(|filename| {
            let contents = match read_to_string(filename) {
                Ok(contents) => contents,
                Err(error) => return Some(Loaded::Error(Box::new(error))),
            };

            match filename.extension().and_then(OsStr::to_str) {
                Some("yml") | Some("yaml") => match yaml::YamlLoader::load_from_str(&contents) {
                    Ok(loaded) => Some(Loaded::YAML(loaded)),
                    Err(error) => Some(Loaded::Error(Box::new(error))),
                },
                // Some("toml") => match &contents.parse::<toml::Value>() {
                //     Ok(loaded) => Some(Loaded::TOML(vec![loaded.clone()])),
                //     Err(error) => Some(Loaded::Error(Box::new(error.clone()))),
                // },
                Some("json") => match json::parse(&contents) {
                    Ok(loaded) => Some(Loaded::JSON(vec![loaded])),
                    Err(error) => Some(Loaded::Error(Box::new(error))),
                },
                Some(_) => Some(Loaded::Skipped(format!("Skipped {:?}", filename))),
                None => None,
            }
        })
        .for_each(|loaded| {
            println!("{:?}", loaded);
        });
}
