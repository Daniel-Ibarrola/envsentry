use std::fs;
use std::collections::HashSet;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use regex::Regex;

fn read_env_file(env_file: String) -> HashSet<String> {
    let file = fs::File::open(env_file).expect("Unable to open file");
    let reader = BufReader::new(file);

    let mut env_variables: HashSet<String> = HashSet::new();
    for line in reader.lines() {
        let line = line.expect("Unable to read line");
        let parts: Option<(&str, &str)> = line.split_once('=');
        if parts.is_some() {
           env_variables.insert(parts.unwrap().0.to_string());
        }
    }
    env_variables
}

fn collect_src_files(src_dir: String) -> Vec<PathBuf> {
    WalkDir::new(src_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .map(|entry| entry.into_path())
        .collect()
}

fn extract_env_variables(src_file: PathBuf) -> Vec<String> {
    let mut env_variables: Vec<String> = vec![];
    let re = Regex::new(r#"env::var\(\s*"([A-Za-z_][A-Za-z0-9_]*)"\s*\)"#).unwrap();

    for line in fs::read_to_string(src_file).expect("Unable to read file").lines() {
        if let Some(caps) = re.captures(&line) {
            env_variables.push(caps[1].to_string());
        }
    }
    env_variables
}

pub fn run(env_file: String, src_dir: String) {
    if !Path::new(&env_file).exists() {
        println!("The env file does not exist");
        return;
    }

    if !Path::new(&src_dir).exists() {
        println!("The src dir does not exist");
        return;
    }

    let env_variables = read_env_file(env_file);
    let src_files = collect_src_files(src_dir);

    println!("{:?}", env_variables);
    println!("{:?}", src_files);

    let mut src_env_variables: Vec<String> = vec![];
    for file in src_files {
        let mut env_variables = extract_env_variables(file);
        src_env_variables.append(&mut env_variables);
    }

    let unique_src_env_variables: HashSet<String> = src_env_variables.into_iter().collect();
    println!("{:?}", unique_src_env_variables);

    let unused_env_variables = env_variables.difference(&unique_src_env_variables);
    for env_variable in unused_env_variables {
        println!("Unused env variable: {}", env_variable);
    }

    let missing_env_variables = unique_src_env_variables.difference(&env_variables);
    for env_variable in missing_env_variables {
        println!("Missing env variable: {}", env_variable);
    }
}

#[cfg(test)]
mod tests {
    use super::*;


}