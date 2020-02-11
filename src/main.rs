mod conversions;
mod documents;
mod merging;

use std::{
    ffi::OsStr,
    io::{self, stdin, Read, Write},
    path::PathBuf,
    str::FromStr,
};

use structopt::StructOpt;
use strum::VariantNames;

use crate::{
    documents::{Document, DocumentError, DocumentType},
    merging::ArrayMergeBehavior,
};

/// Command-line arguments for this tool
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct CliArgs {
    /// Files to process. Formats are inferred from the filename extension.
    /// A `-` (dash) can be used to indicate `stdin`, however two conditions apply:
    /// 1. The `--stdin-format` argument has to be specified (so we know how to parse the incoming stream).
    /// 2. The dash can only be present at most once in the arguments list (because stdin can only be used once).
    #[structopt(name = "FILE", parse(from_os_str), required = true)]
    filenames: Vec<PathBuf>,

    /// How to handle arrays merging
    #[structopt(long = "arrays", default_value, possible_values = &ArrayMergeBehavior::VARIANTS)]
    array_merge: ArrayMergeBehavior,

    /// Force output to be in a specific format, otherwise the format of
    /// first file in the arguments is used.
    #[structopt(long = "force-format", possible_values = &DocumentType::VARIANTS)]
    force_format: Option<DocumentType>,

    /// Defines the format for stdin data. This is required if the dash
    /// (`-`, the stdin placeholder) is specified as a file argument.
    /// Otherwise it is ignored.
    #[structopt(long = "stdin-format", possible_values = &DocumentType::VARIANTS)]
    stdin_format: Option<DocumentType>,
}

fn handle_stdout_error<T>(result: io::Result<T>) {
    if let Err(original_error) = result {
        if let Err(stderr_error) = writeln!(
            io::stderr().lock(),
            "Error writing to stdout: {}",
            original_error
        ) {
            // If we can't log errors to stderr...
            // Well not much we can do now...
            drop(stderr_error);
        }
    }
}

fn main() {
    let CliArgs {
        filenames,
        array_merge,
        force_format,
        stdin_format,
    } = CliArgs::from_args();

    let mut stdin_found = false;

    if filenames
        .iter()
        .any(|filename| filename.to_str().map(|s| s == "-").unwrap_or(false))
        && stdin_format.is_none()
    {
        eprintln!("Error: `--stdin-format` must be set when `-` (stdin) is specified.");
        std::process::exit(1);
    }

    let mut documents = filenames
        .into_iter()
        .filter_map(|filename| {
            let is_stdin = filename.to_str().map(|s| s == "-").unwrap_or(false);

            if is_stdin {
                if stdin_found {
                    return None;
                }
                stdin_found = true;

                stdin_format.map(|doc_type| {
                    let mut buffer = String::new();
                    stdin()
                        .lock()
                        .read_to_string(&mut buffer)
                        .map_err(|error| DocumentError::Loading {
                            filename: filename.clone(),
                            error: Box::new(error),
                        })
                        .and_then(|_| doc_type.load_from_str(&buffer, &filename))
                })
            } else {
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

    let stdout = io::stdout();

    match result {
        Document::JSON(json) => json.into_iter().map(|doc| doc.pretty(2)).for_each(|out| {
            handle_stdout_error(writeln!(stdout.lock(), "{}", &out));
        }),
        Document::YAML(yaml) => yaml
            .into_iter()
            .map(|doc| {
                use yaml_rust as yamllib;
                let mut out_str = String::new();
                let mut emitter = yamllib::YamlEmitter::new(&mut out_str);
                emitter.dump(&doc).unwrap();
                out_str
            })
            .for_each(|out| {
                handle_stdout_error(writeln!(stdout.lock(), "{}", &out));
            }),
    }
}
