use regex::Regex;
use std::io;
use std::io::BufRead;
use std::sync::OnceLock;

// #[derive(Debug)]
// struct EnvOccurrence {
//     name: String,
//     file: std::path::PathBuf,
//     line: usize,
//     column: usize,
// }

fn src_env_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?:(?:std::)?env::(?:var|var_os)|var|env!|option_env!)\(\s*"([A-Za-z_][A-Za-z0-9_]*)"\s*\)"#)
            .expect("hardcoded regex must be valid")
    })
}

pub fn process_src_file<R: BufRead>(mut reader: R) -> io::Result<Vec<String>> {
    let mut env_variables = Vec::new();
    let re = src_env_regex();

    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    for caps in re.captures_iter(&content) {
        env_variables.push(caps[1].to_string());
    }

    Ok(env_variables)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn process(input: &str) -> Vec<String> {
        let reader = Cursor::new(input);
        process_src_file(reader).unwrap()
    }

    #[test]
    fn test_extracts_env_variables_from_src_file() {
        let input = "fn main() {\n    env::var(\"FOO\");\n    env::var(\"BAR\");\n}";

        let envs = process(input);

        assert!(envs.contains(&"FOO".to_string()));
        assert!(envs.contains(&"BAR".to_string()));
        assert!(!envs.contains(&"INVALID_LINE".to_string()));
    }

    #[test]
    fn test_extracts_env_variables_from_std_env_var_calls() {
        let input =
            "fn main() {\n    std::env::var(\"API_KEY\");\n    std::env::var(\"SECRET_KEY\");\n}";

        let envs = process(input);

        assert!(envs.contains(&"API_KEY".to_string()));
        assert!(envs.contains(&"SECRET_KEY".to_string()));
    }

    #[test]
    fn test_extracts_env_variables_from_direct_var_import_calls() {
        let input = "use std::env::var;\nfn main() {\n    var(\"API_KEY\");\n}";

        let envs = process(input);

        assert!(envs.contains(&"API_KEY".to_string()));
    }

    #[test]
    fn test_extracts_env_variables_from_var_os_calls() {
        let input =
            "fn main() {\n    env::var_os(\"API_KEY\");\n    std::env::var_os(\"SECRET_KEY\");\n}";

        let envs = process(input);

        assert!(envs.contains(&"API_KEY".to_string()));
        assert!(envs.contains(&"SECRET_KEY".to_string()));
    }

    #[test]
    fn test_extracts_env_variables_from_env_macro() {
        let input = "fn main() {\n    let _ = env!(\"API_KEY\");\n}";

        let envs = process(input);

        assert!(envs.contains(&"API_KEY".to_string()));
    }

    #[test]
    fn test_extracts_env_variables_from_option_env_macro() {
        let input = "fn main() {\n    let _ = option_env!(\"API_KEY\");\n}";

        let envs = process(input);

        assert!(envs.contains(&"API_KEY".to_string()));
    }

    #[test]
    fn test_extracts_env_variables_from_multiline_calls() {
        let input = "fn main() {\n    env::var(\n        \"API_KEY\"\n    );\n}";

        let envs = process(input);

        assert!(envs.contains(&"API_KEY".to_string()));
    }
}
