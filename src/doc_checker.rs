use chumsky::prelude::*;
use chumsky::span::SimpleSpan;

use crate::{
    helpers::{self},
    parser::{self, ComplexToken::*, FunctionInfo, StructInfo},
};

const RETURN_DOC_KEYWORD: &str = "Return";

pub struct DocChecker {}

impl DocChecker {
    pub fn new() -> Self {
        Self {}
    }

    pub fn check_documentation(&self, content: &str, print_tokens: bool) -> Result<(), String> {
        // Exit on empty input.
        if content.is_empty() {
            return Ok(());
        }

        // Parse tokens.
        let (tokens, errors) = parser::token_parser().parse(content).into_output_errors();

        // Show any errors.
        if !errors.is_empty() {
            if let Some(error) = errors.into_iter().next() {
                let (line, column) =
                    helpers::span_offset_to_line_and_column(error.span().start, content);
                let reason = error.reason();

                return Err(format!(
                    "token parser error at line {} column {}, reason: {}",
                    line, column, reason
                ));
            }
        }

        // Exit of no tokens returned (not an error).
        if tokens.is_none() {
            return Ok(());
        }
        let tokens: Vec<(parser::Token<'_>, SimpleSpan)> = tokens.unwrap();

        // Print tokens if needed.
        if print_tokens {
            println!("parsed tokens:");
            for token in &tokens {
                let (line, column) =
                    helpers::span_offset_to_line_and_column(token.1.start, content);
                println!("[line {}, column {}] {}", line, column, token.0);
            }
            println!("------------------------------------\n");
        }

        // Parse more stuff.
        let (complex_tokens, errors) = parser::complex_token_parser()
            .parse(tokens.spanned((tokens.len()..tokens.len()).into()))
            .into_output_errors();

        // Show any errors.
        if !errors.is_empty() {
            if let Some(error) = errors.into_iter().next() {
                let (line, column) =
                    helpers::span_offset_to_line_and_column(error.span().start, content);
                let reason = error.reason();
                return Err(format!(
                    "statement parser error at line {} column {}, reason: {}",
                    line, column, reason
                ));
            }
        }

        match complex_tokens {
            None => Ok(()), // nothing to do here
            Some(tokens) => {
                // Print tokens if needed.
                if print_tokens {
                    println!("parsed complex tokens:");
                    for token in &tokens {
                        let (line, column) =
                            helpers::span_offset_to_line_and_column(token.1.start, content);
                        println!("[line {}, column {}] {}", line, column, token.0);
                    }
                    println!("------------------------------------\n");
                }

                // Check.
                match self.check_complex_tokens(tokens) {
                    Ok(_) => Ok(()), // everything is fine
                    Err(msg) => Err(msg),
                }
            }
        }
    }

    fn check_complex_tokens(
        &self,
        complex_tokens: Vec<(parser::ComplexToken<'_>, SimpleSpan)>,
    ) -> Result<(), String> {
        for (complex_token, _) in complex_tokens {
            match complex_token {
                Struct(info) => {
                    // Check docs.
                    Self::check_struct_docs(&info)?;

                    // Check field docs.
                    Self::check_struct_field_docs(&info)?
                }
                Function(info) => {
                    // Check docs.
                    Self::check_function_docs(&info)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn check_function_docs(func_info: &FunctionInfo) -> Result<(), String> {
        // Make sure docs are not empty.
        if func_info.docs.is_empty() {
            return Err(format!(
                "expected to find documentation for the function \"{}\"",
                func_info.name
            ));
        }

        // Make sure docs are using ASCII characters since we will use `find` on bytes not chars.
        if !func_info.docs.is_ascii() {
            return Err(format!(
                "expected the documentation for the function \"{}\" to only use ASCII characters",
                func_info.name
            ));
        }

        // Check return docs.
        let return_doc_pos = func_info.docs.find(RETURN_DOC_KEYWORD);
        if !func_info.void_return_type {
            if return_doc_pos.is_none() {
                return Err(format!(
                    "expected to find a \"{}\" in the documentation that describes the return value for the function \"{}\"",
                    RETURN_DOC_KEYWORD, func_info.name
                ));
            }
        } else if return_doc_pos.is_some() {
            // Make sure there is no "return" docs (since it's void).
            return Err(format!(
                "found documentation of the VOID return value for the function \"{}\"",
                func_info.name
            ));
        }

        // Collect all args written in the docs.
        let param_keyword = "* `";
        let mut documented_args: Vec<String> = Vec::new();
        let found_arg_docs: Vec<_> = func_info.docs.match_indices(param_keyword).collect();
        let docs_as_bytes = func_info.docs.as_bytes();
        for (pos, _) in found_arg_docs {
            let mut current_pos = pos + param_keyword.len();
            let mut arg_name = String::new();

            while current_pos < docs_as_bytes.len() {
                let _char = docs_as_bytes[current_pos];
                if _char as char == '`' {
                    if arg_name.is_empty() {
                        current_pos += 1;
                        continue;
                    } else {
                        break;
                    }
                }

                arg_name += &(_char as char).to_string();
                current_pos += 1;
            }

            documented_args.push(arg_name);
        }

        // Check argument docs.
        for arg_name in &func_info.args {
            if *arg_name == "self" {
                continue;
            }

            if !documented_args.iter().any(|name| name == arg_name) {
                return Err(format!(
                    "expected to find documentation for the argument \"{}\" of the function \"{}\"",
                    arg_name, func_info.name
                ));
            }
        }

        // Check if there are argument comments that don't reference an actual argument.
        for doc_arg_name in documented_args {
            if !func_info
                .args
                .iter()
                .any(|arg_name| *arg_name == doc_arg_name.as_str())
            {
                return Err(format!(
                    "found documentation for a non-existing argument \"{}\" of the function \"{}\"",
                    doc_arg_name, func_info.name
                ));
            }
        }

        Ok(())
    }

    fn check_struct_docs(struct_info: &StructInfo) -> Result<(), String> {
        // Make sure docs are not empty.
        if struct_info.docs.is_empty() {
            return Err(format!(
                "expected to find documentation for the struct \"{}\"",
                struct_info.name
            ));
        }

        Ok(())
    }

    /// Checks that the documentation for fields of the specified struct are written correctly.
    ///
    /// # Return
    /// `Ok` if docs are correct, otherwise `Err` with a meaningful message about incorrect docs.
    fn check_struct_field_docs(struct_info: &StructInfo) -> Result<(), String> {
        for info in &struct_info.fields {
            // Make sure docs are not empty.
            if info.docs.is_empty() {
                return Err(format!(
                    "expected to find documentation for the struct field \"{}\"",
                    info.name
                ));
            }
        }

        Ok(())
    }
}
