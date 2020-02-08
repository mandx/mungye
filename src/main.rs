mod conversions;
mod documents;
mod merging;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::FromStr;

use structopt::StructOpt;

use documents::{Document, DocumentError, DocumentType};
use merging::ArrayMergeBehavior;

/// Command-line arguments for this tool
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct CliArgs {
    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,

    #[structopt(long = "arrays")]
    array_merge: ArrayMergeBehavior,

    #[structopt(long = "force-format")]
    force_format: Option<DocumentType>,
}

fn main() {
    let CliArgs {
        files: filenames,
        array_merge,
        force_format,
    } = CliArgs::from_args();

    let mut documents = filenames
        .into_iter()
        .filter_map(|filename| {
            filename
                .extension()
                .and_then(OsStr::to_str)
                .map(|extension| {
                    DocumentType::from_str(extension)
                        .map_err(|_| DocumentError::Skipped {
                            filename: filename.clone(),
                        })
                        .and_then(|doc_type| doc_type.load_from_path(&filename))
                })
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

    let destination = force_format
        .as_ref()
        .map(DocumentType::default_document)
        .unwrap_or_else(|| match documents.next() {
            Some(loaded) => loaded,
            None => {
                eprintln!("Got no documents to work with!");
                std::process::exit(1);
            }
        });

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
                    use yaml_rust as yamllib;
                    let mut out_str = String::new();
                    let mut emitter = yamllib::YamlEmitter::new(&mut out_str);
                    emitter.dump(&doc).unwrap();
                    out_str
                })
                .collect(),
        }
    );
}
