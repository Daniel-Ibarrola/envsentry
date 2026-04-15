//! This module provides functionality for parsing `.env` files and extracting environment variable names.
//!
//! It supports standard `.env` file syntax, including:
//! - Lines in the format `KEY=VALUE`.
//! - Optional `export ` prefix for keys.
//! - Comments starting with `#`.
//! - Stripping of the UTF-8 Byte Order Mark (BOM).
//! - Validation of environment variable names.

use std::collections::HashSet;
use std::io;
use std::io::BufRead;

/// Checks if a string is a valid environment variable key.
///
/// A valid key must:
/// - Start with an ASCII letter (`a-z`, `A-Z`) or an underscore (`_`).
/// - Contain only ASCII alphanumeric characters (`a-z`, `A-Z`, `0-9`) or underscores (`_`).
fn is_valid_key(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {}
        _ => return false,
    }

    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

/// Represents a single environment variable definition in an environment file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvDefinition {
    /// The name of the environment variable.
    pub name: String,
    /// The line number (0-indexed) where the definition was found.
    pub line: usize,
    /// The byte offset where the variable name starts in the full file.
    pub span_start: usize,
    /// The byte length of the variable name.
    pub span_len: usize,
    /// The value of the environment variable.
    pub value: String,
}

/// Processes an environment file and returns a set of unique environment variable keys.
///
/// This function reads from a `BufRead` source line by line, identifies environment variable
/// definitions, and extracts their keys. It handles comments, empty lines, and the `export` prefix.
///
/// # Errors
///
/// Returns an `std::io::Error` if reading from the `reader` fails.
pub fn process_env_file<R: BufRead>(
    mut reader: R,
) -> io::Result<(HashSet<String>, Vec<EnvDefinition>)> {
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    let mut env_variables = HashSet::new();
    let mut env_definitions = Vec::new();
    let mut file_offset = 0usize;

    for (line_number, raw_line) in contents.split_inclusive('\n').enumerate() {
        let line = raw_line.strip_suffix('\n').unwrap_or(raw_line);
        let line_start_offset = file_offset;

        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            file_offset += raw_line.len();
            continue;
        }

        if let Some((key_part, value)) = line.split_once('=') {
            let normalized_key = key_part
                .trim_start_matches('\u{FEFF}')
                .trim_start_matches("export ")
                .trim();

            if is_valid_key(normalized_key) {
                if let Some(key_start_in_line) = line.find(normalized_key) {
                    let span_start = line_start_offset + key_start_in_line;
                    let span_len = normalized_key.len();

                    env_variables.insert(normalized_key.to_string());
                    env_definitions.push(EnvDefinition {
                        name: normalized_key.to_string(),
                        line: line_number,
                        span_start,
                        span_len,
                        value: value.trim_end_matches('\r').to_string(),
                    });
                }
            }
        }

        file_offset += raw_line.len();
    }

    Ok((env_variables, env_definitions))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_extracts_env_variables_from_env_file() {
        let input = "FOO=bar\nBAZ=qux\nINVALID_LINE\n";
        let reader = Cursor::new(input);

        let (envs, definitions) = process_env_file(reader).unwrap();

        assert!(envs.contains("FOO"));
        assert!(envs.contains("BAZ"));
        assert!(!envs.contains("INVALID_LINE"));

        assert_eq!(definitions.len(), 2);

        assert_eq!(definitions[0].name, "FOO");
        assert_eq!(definitions[0].line, 0);
        assert_eq!(definitions[0].span_start, 0);
        assert_eq!(definitions[0].span_len, 3);

        assert_eq!(definitions[1].name, "BAZ");
        assert_eq!(definitions[1].line, 1);
        assert_eq!(definitions[1].span_start, 8);
        assert_eq!(definitions[1].span_len, 3);
    }

    #[test]
    fn test_ignores_empty_lines() {
        let input = "FOO=bar\n\nBAZ=qux\n";
        let reader = Cursor::new(input);

        let (envs, definitions) = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 2);
        assert!(envs.contains("FOO"));
        assert!(envs.contains("BAZ"));

        assert_eq!(definitions.len(), 2);
        assert_eq!(definitions[0].line, 0);
        assert_eq!(definitions[0].span_start, 0);
        assert_eq!(definitions[0].span_len, 3);

        assert_eq!(definitions[1].line, 2);
        assert_eq!(definitions[1].span_start, 9);
        assert_eq!(definitions[1].span_len, 3);
    }

    #[test]
    fn test_ignores_comments() {
        let input = "# This is a comment\nFOO=bar\n";
        let reader = Cursor::new(input);

        let (envs, definitions) = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 1);
        assert!(envs.contains("FOO"));

        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].line, 1);
        assert_eq!(definitions[0].span_start, 20);
        assert_eq!(definitions[0].span_len, 3);
    }

    #[test]
    fn test_trims_whitespace() {
        let input = "FOO = bar\nBAZ = qux\n";
        let reader = Cursor::new(input);

        let (envs, definitions) = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 2);

        assert!(envs.contains("FOO"));
        assert_eq!(definitions[0].span_start, 0);
        assert_eq!(definitions[0].span_len, 3);

        assert!(envs.contains("BAZ"));
        assert_eq!(definitions[1].span_start, 10);
        assert_eq!(definitions[1].span_len, 3);
    }

    #[test]
    fn test_strips_export_prefix_from_keys() {
        let input = "export FOO=bar\nBAR=baz\n";
        let reader = Cursor::new(input);

        let (envs, definitions) = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 2);
        assert!(envs.contains("FOO"));
        assert!(envs.contains("BAR"));
        assert!(!envs.contains("export FOO"));

        assert_eq!(definitions[0].name, "FOO");
        assert_eq!(definitions[0].span_start, 7);
        assert_eq!(definitions[0].span_len, 3);

        assert_eq!(definitions[1].name, "BAR");
        assert_eq!(definitions[1].span_start, 15);
        assert_eq!(definitions[1].span_len, 3);
    }

    #[test]
    fn test_ignores_empty_key() {
        let input = "=bar\nFOO=baz\n";
        let reader = Cursor::new(input);

        let (envs, _) = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 1);
        assert!(envs.contains("FOO"));
        assert!(!envs.contains(""));
    }

    #[test]
    fn test_ignores_invalid_variable_names() {
        let input = "123FOO=bar\nFOO-BAR=baz\nGOOD_NAME=ok\n";
        let reader = Cursor::new(input);

        let (envs, _) = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 1);
        assert!(envs.contains("GOOD_NAME"));
        assert!(!envs.contains("123FOO"));
        assert!(!envs.contains("FOO-BAR"));
    }

    #[test]
    fn test_strips_utf8_bom_from_first_key() {
        let input = "\u{FEFF}FOO=bar\nBAR=baz\n";
        let reader = Cursor::new(input);

        let (envs, _) = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 2);
        assert!(envs.contains("FOO"));
        assert!(envs.contains("BAR"));
        assert!(!envs.contains("\u{FEFF}FOO"));
    }

    #[test]
    fn test_handles_duplicate_keys_once() {
        let input = "FOO=one\nFOO=two\nBAR=three\n";
        let reader = Cursor::new(input);

        let (envs, definitions) = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 2);
        assert!(envs.contains("FOO"));
        assert!(envs.contains("BAR"));

        assert_eq!(definitions.len(), 3);
    }

    #[test]
    fn test_accepts_empty_value() {
        let input = "FOO=\nBAR=baz\n";
        let reader = Cursor::new(input);

        let (envs, _) = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 2);
        assert!(envs.contains("FOO"));
        assert!(envs.contains("BAR"));
    }

    #[test]
    fn test_parses_key_when_value_contains_additional_equals() {
        let input = "DATABASE_URL=postgres://user:pass@host/db?x=y\nFOO=bar\n";
        let reader = Cursor::new(input);

        let (envs, _) = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 2);
        assert!(envs.contains("DATABASE_URL"));
        assert!(envs.contains("FOO"));
    }

    #[test]
    fn test_accepts_valid_keys() {
        for key in ["FOO", "_FOO", "FOO123"] {
            assert!(is_valid_key(key), "expected valid key: {key}");
        }
    }

    #[test]
    fn test_rejects_invalid_keys() {
        for key in ["", "123FOO", "FOO-BAR", "FOO BAR"] {
            assert!(!is_valid_key(key), "expected invalid key: {key}");
        }
    }
}
