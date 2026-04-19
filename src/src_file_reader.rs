//! This module provides functionality for scanning Rust source files for environment variable usage.
//!
//! It uses a regular expression to find calls to `env::var`, `std::env::var`, `env!`, etc.

use regex::Regex;
use std::io;
use std::io::BufRead;
use std::sync::OnceLock;

/// Represents an occurrence of an environment variable in a source file.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EnvOccurrence {
    /// The name of the environment variable.
    pub name: String,
    /// The path to the file where the occurrence was found.
    pub file_path: String,
    /// The line number (1-indexed) where the occurrence starts.
    pub line: usize,
    /// The column number (1-indexed) where the environment variable name starts.
    pub column: usize,
    /// The byte offset where the occurrence starts in the full file.
    pub span_start: usize,
    /// The byte offset of the variable name.
    pub span_len: usize,
}

fn src_env_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?:(?:std::)?env::(?:var|var_os)|var|env!|option_env!)\(\s*"([A-Za-z_][A-Za-z0-9_]*)"\s*\)"#)
            .expect("hardcoded regex must be valid")
    })
}

/// Processes a source file and extracts all environment variable occurrences.
///
/// # Errors
///
/// Returns an `std::io::Error` if reading from the `reader` fails.
pub fn process_src_file<R: BufRead>(
    mut reader: R,
    file_path: &str,
) -> io::Result<Vec<EnvOccurrence>> {
    let mut env_occurrences = Vec::new();
    let re = src_env_regex();

    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    let mut line_starts = vec![0];
    for (i, c) in content.char_indices() {
        if c == '\n' {
            line_starts.push(i + 1);
        }
    }

    for caps in re.captures_iter(&content) {
        let name_match = caps.get(1).unwrap();
        let span_start = name_match.start();
        let span_len = name_match.as_str().len();
        let name = name_match.as_str().to_string();

        let line = line_starts
            .binary_search(&span_start)
            .unwrap_or_else(|x| x - 1);
        let column = span_start - line_starts[line] + 1;

        env_occurrences.push(EnvOccurrence {
            name,
            file_path: file_path.to_string(),
            line: line + 1,
            column,
            span_start,
            span_len,
        });
    }

    Ok(env_occurrences)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn process(input: &str) -> Vec<EnvOccurrence> {
        let reader = Cursor::new(input);
        process_src_file(reader, "test.rs").unwrap()
    }

    #[test]
    fn test_extracts_env_variables_from_src_file() {
        let input = "fn main() {\n    env::var(\"FOO\");\n    env::var(\"BAR\");\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| e.name == "FOO"
            && e.line == 2
            && e.column == 15
            && e.span_start == 26
            && e.span_len == 3
            && e.file_path == "test.rs"));
        assert!(envs.iter().any(|e| e.name == "BAR"
            && e.line == 3
            && e.column == 15
            && e.span_start == 47
            && e.span_len == 3
            && e.file_path == "test.rs"));
    }

    #[test]
    fn test_extracts_env_variables_from_std_env_var_calls() {
        let input =
            "fn main() {\n    std::env::var(\"API_KEY\");\n    std::env::var(\"SECRET_KEY\");\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| {
            e.name == "API_KEY"
                && e.line == 2
                && e.column == 20
                && e.span_start == 31
                && e.span_len == 7
        }));
        assert!(envs.iter().any(|e| {
            e.name == "SECRET_KEY"
                && e.line == 3
                && e.column == 20
                && e.span_start == 61
                && e.span_len == 10
        }));
    }

    #[test]
    fn test_extracts_env_variables_from_direct_var_import_calls() {
        let input = "use std::env::var;\nfn main() {\n    var(\"API_KEY\");\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| {
            e.name == "API_KEY"
                && e.line == 3
                && e.column == 10
                && e.span_start == 40
                && e.span_len == 7
        }));
    }

    #[test]
    fn test_extracts_env_variables_from_var_os_calls() {
        let input =
            "fn main() {\n    env::var_os(\"API_KEY\");\n    std::env::var_os(\"SECRET_KEY\");\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| {
            e.name == "API_KEY"
                && e.line == 2
                && e.column == 18
                && e.span_start == 29
                && e.span_len == 7
        }));
        assert!(envs.iter().any(|e| {
            e.name == "SECRET_KEY"
                && e.line == 3
                && e.column == 23
                && e.span_start == 62
                && e.span_len == 10
        }));
    }

    #[test]
    fn test_extracts_env_variables_from_env_macro() {
        let input = "fn main() {\n    let _ = env!(\"API_KEY\");\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| {
            e.name == "API_KEY"
                && e.line == 2
                && e.column == 19
                && e.span_start == 30
                && e.span_len == 7
        }));
    }

    #[test]
    fn test_extracts_env_variables_from_option_env_macro() {
        let input = "fn main() {\n    let _ = option_env!(\"API_KEY\");\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| {
            e.name == "API_KEY"
                && e.line == 2
                && e.column == 26
                && e.span_start == 37
                && e.span_len == 7
        }));
    }

    #[test]
    fn test_extracts_env_variables_from_multiline_calls() {
        let input = "fn main() {\n    env::var(\n        \"API_KEY\"\n    );\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| {
            e.name == "API_KEY"
                && e.line == 3
                && e.column == 10
                && e.span_start == 35
                && e.span_len == 7
        }));
    }

    #[test]
    fn test_extracts_multiple_env_variables_from_same_statement() {
        let input = "fn main() {\n let (var, var2) = (\n env::var(\"MISSING_VAR_1\").unwrap(), env::var(\"MISSING_VAR_2\").unwrap()\n);\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| {
            e.name == "MISSING_VAR_1"
                && e.line == 3
                && e.column == 12
                && e.span_start == 44
                && e.span_len == 13
        }));
        assert!(envs.iter().any(|e| {
            e.name == "MISSING_VAR_2"
                && e.line == 3
                && e.column == 48
                && e.span_start == 80
                && e.span_len == 13
        }));
    }
}
