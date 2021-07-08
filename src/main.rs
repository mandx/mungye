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
    documents::{Document, DocumentError, DocumentType, NamespaceWith},
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

    #[structopt(long = "namespace")]
    namespace: Option<NamespaceWith>,
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
        namespace: wrap,
    } = CliArgs::from_args();

    let use_stdin = filenames
        .iter()
        .any(|filename| filename.to_str().map(|s| s == "-").unwrap_or(false));

    if use_stdin && stdin_format.is_none() {
        eprintln!("Error: `--stdin-format` must be set when `-` (stdin) is specified.");
        // TODO: Get rid of `exit()` since this doesn't allow destructors to
        // run properly. Change `main` to return a `Result<>`, also look into
        // https://github.com/sgrif/terminator
        std::process::exit(1);
    }

    let stdin_doc_result = match (use_stdin, stdin_format) {
        (true, Some(doc_type)) => {
            let mut buffer = String::new();
            stdin()
                .lock()
                .read_to_string(&mut buffer)
                .map_err(|error| DocumentError::Loading {
                    filename: "-".into(),
                    error: Box::new(error),
                })
                .and_then(|_| doc_type.load_from_str(&buffer, PathBuf::from("-")))
        }
        (true, None) => {
            eprintln!("Error: `--stdin-format` must be set when `-` (stdin) is specified.");
            // TODO: Get rid of `exit()` since this doesn't allow destructors to
            // run properly. Change `main` to return a `Result<>`, also look into
            // https://github.com/sgrif/terminator
            std::process::exit(1);
        }
        (false, _) => Err(DocumentError::Skipped {
            filename: "-".into(),
        }),
    };

    let mut documents = filenames.into_iter().filter_map(|filename| {
        // Check if is this is stdin's placeholder
        if filename.to_str().map(|s| s == "-").unwrap_or(false) {
            // stdin's result is a singleton, but unfortunately most errors are
            // not `Clone`, so we can't clone the entire result, which means
            // we need to handle stdin's processing right here.
            match stdin_doc_result.as_ref() {
                Ok(stdin_doc) => Some(stdin_doc.clone()),
                Err(DocumentError::Skipped { filename }) => {
                    eprintln!("Skipped {:?}", filename);
                    None
                }
                Err(DocumentError::Loading { filename, error }) => {
                    eprintln!("Error loading {:?}: {:?}", filename, error);
                    None
                }
            }
        } else {
            // Treat `filename` as a regular file
            match filename
                .extension()
                .and_then(OsStr::to_str)
                .map(|extension| {
                    DocumentType::from_str(extension)
                        .map_err(|_| DocumentError::Skipped {
                            filename: filename.clone(),
                        })
                        .and_then(|doc_type| doc_type.load_from_path(&filename))
                        .map(|doc| match wrap {
                            Some(using) => using.wrap(doc, &filename),
                            None => doc,
                        })
                }) {
                Some(Err(DocumentError::Skipped { filename })) => {
                    eprintln!("Skipped {:?}", filename);
                    None
                }
                Some(Err(DocumentError::Loading { filename, error })) => {
                    eprintln!("Error loading {:?}: {:?}", filename, error);
                    None
                }
                Some(Ok(document)) => Some(document),
                None => {
                    eprintln!("Skipped {:?}", filename);
                    None
                }
            }
        }
    });

    let destination = force_format
        .as_ref()
        .map(|doc_type| doc_type.default_document())
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
