use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn read_env<R: BufRead>(reader: R) -> HashSet<String> {
    let mut env_variables = HashSet::new();

    for line in reader.lines() {
        let line = line.expect("Unable to read line");
        if let Some((key, _value)) = line.split_once('=') {
            env_variables.insert(key.to_string());
        }
    }

    env_variables
}

fn read_env_file(env_file: &Path) -> HashSet<String> {
    let file = fs::File::open(env_file)
        .expect(format!("Unable to open file {}", env_file.display()).as_str());
    let reader = BufReader::new(file);
    read_env(reader)
}

fn collect_src_files(src_dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(src_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .map(|entry| entry.into_path())
        .collect()
}

fn process_src_file<R: BufRead>(reader: R) -> Vec<String> {
    let re = Regex::new(r#"env::var\(\s*"([A-Za-z_][A-Za-z0-9_]*)"\s*\)"#).unwrap();
    let mut env_variables: Vec<String> = vec![];
    for line in reader.lines() {
        let line = line.expect("Unable to read line");
        if let Some(caps) = re.captures(&line) {
            env_variables.push(caps[1].to_string());
        }
    }
    env_variables
}

fn extract_env_variables(src_file: &Path) -> Vec<String> {
    let file = fs::File::open(src_file)
        .expect(format!("Unable to open file {}", src_file.display()).as_str());
    let reader = BufReader::new(file);
    process_src_file(reader)
}

pub struct AnalysisResult {
    pub unused: HashSet<String>,
    pub missing: HashSet<String>,
}

pub fn analyze(env_file: &Path, src_dir: &Path) -> AnalysisResult {
    let env_variables = read_env_file(env_file);
    let src_files = collect_src_files(src_dir);

    let mut src_env_variables: Vec<String> = vec![];
    for file in src_files {
        let mut env_variables = extract_env_variables(&file);
        src_env_variables.append(&mut env_variables);
    }

    let unique_src_env_variables: HashSet<String> = src_env_variables.into_iter().collect();
    let unused_env_variables = env_variables.difference(&unique_src_env_variables);
    let missing_env_variables = unique_src_env_variables.difference(&env_variables);

    AnalysisResult {
        unused: unused_env_variables.cloned().collect(),
        missing: missing_env_variables.cloned().collect(),
    }
}

pub fn run(env_file: &Path, src_dir: &Path) {
    let result = analyze(env_file, src_dir);

    for env_variable in &result.unused {
        println!("Unused env variable: {}", env_variable);
    }

    for env_variable in &result.missing {
        println!("Missing env variable: {}", env_variable);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_reads_env_vars_from_reader() {
        let input = "FOO=bar\nBAZ=qux\nINVALID_LINE\n";
        let reader = Cursor::new(input);

        let envs = read_env(reader);

        assert!(envs.contains("FOO"));
        assert!(envs.contains("BAZ"));
        assert!(!envs.contains("INVALID_LINE"));
    }

    #[test]
    fn test_extracts_env_variables_from_src_file() {
        let input = "fn main() {\n    env::var(\"FOO\");\n    env::var(\"BAR\");\n}";
        let reader = Cursor::new(input);

        let envs = process_src_file(reader);

        assert!(envs.contains(&"FOO".to_string()));
        assert!(envs.contains(&"BAR".to_string()));
        assert!(!envs.contains(&"INVALID_LINE".to_string()));
    }
}
