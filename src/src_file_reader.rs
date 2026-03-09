use std::io;
use std::io::BufRead;
use regex::Regex;

pub fn process_src_file<R: BufRead>(reader: R, re: &Regex) -> io::Result<Vec<String>> {
    let mut env_variables = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Some(caps) = re.captures(&line) {
            env_variables.push(caps[1].to_string());
        }
    }

    Ok(env_variables)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_extracts_env_variables_from_src_file() {
        let input = "fn main() {\n    env::var(\"FOO\");\n    env::var(\"BAR\");\n}";
        let reader = Cursor::new(input);
        let re = Regex::new(r#"env::var\(\s*"([A-Za-z_][A-Za-z0-9_]*)"\s*\)"#)
            .expect("hardcoded regex must be valid");

        let envs = process_src_file(reader, &re).unwrap();

        assert!(envs.contains(&"FOO".to_string()));
        assert!(envs.contains(&"BAR".to_string()));
        assert!(!envs.contains(&"INVALID_LINE".to_string()));
    }
}