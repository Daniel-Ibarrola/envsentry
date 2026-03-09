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

/// Processes an environment file and returns a set of unique environment variable keys.
///
/// This function reads from a `BufRead` source line by line, identifies environment variable 
/// definitions, and extracts their keys. It handles comments, empty lines, and the `export` prefix.
///
/// # Errors
///
/// Returns an `std::io::Error` if reading from the `reader` fails.
pub fn process_env_file<R: BufRead>(reader: R) -> io::Result<HashSet<String>> {
    let mut env_variables = HashSet::new();

    for line in reader.lines() {
        let line = line?;

        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((key, _value)) = trimmed.split_once('=') {
            let normalized_key = key
                .trim_start_matches('\u{FEFF}')
                .trim_start_matches("export ")
                .trim();

            if is_valid_key(normalized_key) {
                env_variables.insert(normalized_key.to_string());
            }
        }
    }

    Ok(env_variables)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_extracts_env_variables_from_env_file() {
        let input = "FOO=bar\nBAZ=qux\nINVALID_LINE\n";
        let reader = Cursor::new(input);

        let envs = process_env_file(reader).unwrap();

        assert!(envs.contains("FOO"));
        assert!(envs.contains("BAZ"));
        assert!(!envs.contains("INVALID_LINE"));
    }

    #[test]
    fn test_ignores_empty_lines() {
        let input = "FOO=bar\n\nBAZ=qux\n";
        let reader = Cursor::new(input);

        let envs = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 2);
        assert!(envs.contains("FOO"));
        assert!(envs.contains("BAZ"));
    }

    #[test]
    fn test_ignores_comments() {
        let input = "# This is a comment\nFOO=bar\n";
        let reader = Cursor::new(input);

        let envs = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 1);
        assert!(envs.contains("FOO"));
    }

    #[test]
    fn test_trims_whitespace() {
        let input = "FOO = bar\nBAZ = qux\n";
        let reader = Cursor::new(input);

        let envs = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 2);
        assert!(envs.contains("FOO"));
        assert!(envs.contains("BAZ"));
    }

    #[test]
    fn test_ignores_comments_with_leading_whitespace() {
        let input = "   # This is a comment\nFOO=bar\n";
        let reader = Cursor::new(input);

        let envs = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 1);
        assert!(envs.contains("FOO"));
    }

    #[test]
    fn test_strips_export_prefix_from_keys() {
        let input = "export FOO=bar\nBAR=baz\n";
        let reader = Cursor::new(input);

        let envs = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 2);
        assert!(envs.contains("FOO"));
        assert!(envs.contains("BAR"));
        assert!(!envs.contains("export FOO"));
    }

    #[test]
    fn test_ignores_empty_key() {
        let input = "=bar\nFOO=baz\n";
        let reader = Cursor::new(input);

        let envs = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 1);
        assert!(envs.contains("FOO"));
        assert!(!envs.contains(""));
    }

    #[test]
    fn test_ignores_invalid_variable_names() {
        let input = "123FOO=bar\nFOO-BAR=baz\nGOOD_NAME=ok\n";
        let reader = Cursor::new(input);

        let envs = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 1);
        assert!(envs.contains("GOOD_NAME"));
        assert!(!envs.contains("123FOO"));
        assert!(!envs.contains("FOO-BAR"));
    }

    #[test]
    fn test_strips_utf8_bom_from_first_key() {
        let input = "\u{FEFF}FOO=bar\nBAR=baz\n";
        let reader = Cursor::new(input);

        let envs = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 2);
        assert!(envs.contains("FOO"));
        assert!(envs.contains("BAR"));
        assert!(!envs.contains("\u{FEFF}FOO"));
    }

    #[test]
    fn test_handles_duplicate_keys_once() {
        let input = "FOO=one\nFOO=two\nBAR=three\n";
        let reader = Cursor::new(input);

        let envs = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 2);
        assert!(envs.contains("FOO"));
        assert!(envs.contains("BAR"));
    }

    #[test]
    fn test_accepts_empty_value() {
        let input = "FOO=\nBAR=baz\n";
        let reader = Cursor::new(input);

        let envs = process_env_file(reader).unwrap();

        assert_eq!(envs.len(), 2);
        assert!(envs.contains("FOO"));
        assert!(envs.contains("BAR"));
    }

    #[test]
    fn test_parses_key_when_value_contains_additional_equals() {
        let input = "DATABASE_URL=postgres://user:pass@host/db?x=y\nFOO=bar\n";
        let reader = Cursor::new(input);

        let envs = process_env_file(reader).unwrap();

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
