use regex::Regex;
use std::io;
use std::io::BufRead;
use std::sync::OnceLock;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EnvOccurrence {
    pub name: String,
    pub line: usize,
    pub column: usize,
}

fn src_env_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?:(?:std::)?env::(?:var|var_os)|var|env!|option_env!)\(\s*"([A-Za-z_][A-Za-z0-9_]*)"\s*\)"#)
            .expect("hardcoded regex must be valid")
    })
}

pub fn process_src_file<R: BufRead>(mut reader: R) -> io::Result<Vec<EnvOccurrence>> {
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
        let offset = name_match.start();
        let name = name_match.as_str().to_string();

        let line = line_starts.binary_search(&offset).unwrap_or_else(|x| x - 1);
        let column = offset - line_starts[line] + 1;

        env_occurrences.push(EnvOccurrence {
            name,
            line: line + 1,
            column,
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
        process_src_file(reader).unwrap()
    }

    #[test]
    fn test_extracts_env_variables_from_src_file() {
        let input = "fn main() {\n    env::var(\"FOO\");\n    env::var(\"BAR\");\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| e.name == "FOO" && e.line == 2 && e.column == 15));
        assert!(envs.iter().any(|e| e.name == "BAR" && e.line == 3 && e.column == 15));
    }

    #[test]
    fn test_extracts_env_variables_from_std_env_var_calls() {
        let input =
            "fn main() {\n    std::env::var(\"API_KEY\");\n    std::env::var(\"SECRET_KEY\");\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| e.name == "API_KEY" && e.line == 2 && e.column == 20));
        assert!(envs.iter().any(|e| e.name == "SECRET_KEY" && e.line == 3 && e.column == 20));
    }

    #[test]
    fn test_extracts_env_variables_from_direct_var_import_calls() {
        let input = "use std::env::var;\nfn main() {\n    var(\"API_KEY\");\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| e.name == "API_KEY" && e.line == 3 && e.column == 10));
    }

    #[test]
    fn test_extracts_env_variables_from_var_os_calls() {
        let input =
            "fn main() {\n    env::var_os(\"API_KEY\");\n    std::env::var_os(\"SECRET_KEY\");\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| e.name == "API_KEY" && e.line == 2 && e.column == 18));
        assert!(envs.iter().any(|e| e.name == "SECRET_KEY" && e.line == 3 && e.column == 23));
    }

    #[test]
    fn test_extracts_env_variables_from_env_macro() {
        let input = "fn main() {\n    let _ = env!(\"API_KEY\");\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| e.name == "API_KEY" && e.line == 2 && e.column == 19));
    }

    #[test]
    fn test_extracts_env_variables_from_option_env_macro() {
        let input = "fn main() {\n    let _ = option_env!(\"API_KEY\");\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| e.name == "API_KEY" && e.line == 2 && e.column == 26));
    }

    #[test]
    fn test_extracts_env_variables_from_multiline_calls() {
        let input = "fn main() {\n    env::var(\n        \"API_KEY\"\n    );\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| e.name == "API_KEY" && e.line == 3 && e.column == 10));
    }

    #[test]
    fn test_extracts_multiple_env_variables_from_same_statement() {
        let input = "fn main() {\n let (var, var2) = (\n env::var(\"MISSING_VAR_1\").unwrap(), env::var(\"MISSING_VAR_2\").unwrap()\n);\n}";

        let envs = process(input);

        assert!(envs.iter().any(|e| e.name == "MISSING_VAR_1" && e.line == 3 && e.column == 12));
        assert!(envs.iter().any(|e| e.name == "MISSING_VAR_2" && e.line == 3 && e.column == 48));
    }
}
