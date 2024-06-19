#![deny(warnings)]

use doc_checker::DocChecker;
use std::process::ExitCode;

mod doc_checker;
mod helpers;
mod parser;
mod tests;

const PRINT_TOKENS_ARG: &str = "--print-tokens";

fn main() -> ExitCode {
    // Make sure a path is specified.
    if std::env::args().len() == 1 {
        println!("rust-doc-checker (v{})", env!("CARGO_PKG_VERSION"));
        println!();
        println!("expected a path to be specified\n");
        return ExitCode::FAILURE;
    }

    // Get path.
    let Some(path) = std::env::args().nth(1) else {
        println!("expected a path to be specified");
        return ExitCode::FAILURE;
    };

    // Make sure it's a file.
    let path = std::path::PathBuf::from(path);
    if !path.is_file() {
        println!("expected \"{}\" to point to a file", path.to_string_lossy());
        return ExitCode::FAILURE;
    }

    // See if we need to print tokens.
    let print_tokens = if let Some(additional_option) = std::env::args().nth(2) {
        additional_option == PRINT_TOKENS_ARG
    } else {
        false
    };

    // Read file.
    let file_content = match std::fs::read_to_string(path.clone()) {
        Ok(content) => content,
        Err(error) => {
            println!("failed to read the file, error: {}", error);
            return ExitCode::FAILURE;
        }
    };

    // Check code.
    let doc_checker = DocChecker::new();
    match doc_checker.check_documentation(&file_content, print_tokens) {
        Ok(_) => {}
        Err(msg) => {
            println!("{}", msg);
            return ExitCode::FAILURE;
        }
    };

    ExitCode::SUCCESS
}
