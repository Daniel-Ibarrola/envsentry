use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::{Path};
use walkdir::WalkDir;

fn process_env_file<R: BufRead>(reader: R) -> io::Result<HashSet<String>> {
    let mut env_variables = HashSet::new();

    for line in reader.lines() {
        let line = line?;
        if let Some((key, _value)) = line.split_once('=') {
            env_variables.insert(key.to_string());
        }
    }

    Ok(env_variables)
}

fn get_file_reader(path: &Path) -> io::Result<BufReader<fs::File>> {
    let file = fs::File::open(path).map_err(|e| {
        io::Error::new(e.kind(), format!("failed to open {}: {}", path.display(), e))
    })?;
    Ok(BufReader::new(file))
}

fn process_src_file<R: BufRead>(reader: R) -> io::Result<Vec<String>> {
    let re = Regex::new(r#"env::var\(\s*"([A-Za-z_][A-Za-z0-9_]*)"\s*\)"#)
        .expect("hardcoded regex must be valid");

    let mut env_variables = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Some(caps) = re.captures(&line) {
            env_variables.push(caps[1].to_string());
        }
    }

    Ok(env_variables)
}

pub struct AnalysisResult {
    pub unused: HashSet<String>,
    pub missing: HashSet<String>,
}

pub fn analyze(env_file: &Path, src_dir: &Path) -> io::Result<AnalysisResult> {
    let env_variables = process_env_file(get_file_reader(env_file)?)?;
    let mut src_env_variables = Vec::new();

    for entry in WalkDir::new(src_dir) {
        let entry = entry.map_err(|e| {
            io::Error::other(format!(
                "failed to walk directory {}: {}",
                src_dir.display(),
                e
            ))
        })?;

        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            let mut envs = process_src_file(get_file_reader(path)?)?;
            src_env_variables.append(&mut envs);
        }
    }

    let unique_src_env_variables: HashSet<String> = src_env_variables.into_iter().collect();
    let unused_env_variables = env_variables.difference(&unique_src_env_variables);
    let missing_env_variables = unique_src_env_variables.difference(&env_variables);

    Ok(AnalysisResult {
        unused: unused_env_variables.cloned().collect(),
        missing: missing_env_variables.cloned().collect(),
    })
}

pub fn run(env_file: &Path, src_dir: &Path) -> io::Result<()> {
    let result = analyze(env_file, src_dir)?;

    for env_variable in &result.unused {
        println!("Unused env variable: {}", env_variable);
    }

    for env_variable in &result.missing {
        println!("Missing env variable: {}", env_variable);
    }

    Ok(())
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
    fn test_extracts_env_variables_from_src_file() {
        let input = "fn main() {\n    env::var(\"FOO\");\n    env::var(\"BAR\");\n}";
        let reader = Cursor::new(input);

        let envs = process_src_file(reader).unwrap();

        assert!(envs.contains(&"FOO".to_string()));
        assert!(envs.contains(&"BAR".to_string()));
        assert!(!envs.contains(&"INVALID_LINE".to_string()));
    }
}
