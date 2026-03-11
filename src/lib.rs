mod env_file_reader;
mod src_file_reader;

use crate::env_file_reader::EnvDefinition;
use std::collections::HashSet;
use std::fs;
use std::io::{self, BufReader};
use std::path::Path;
use walkdir::WalkDir;

fn get_file_reader(path: &Path) -> io::Result<BufReader<fs::File>> {
    let file = fs::File::open(path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("failed to open {}: {}", path.display(), e),
        )
    })?;
    Ok(BufReader::new(file))
}

#[derive(Debug)]
pub struct AnalysisResult {
    pub unused: Vec<EnvDefinition>,
    pub missing: Vec<src_file_reader::EnvOccurrence>,
}

pub fn analyze(env_file: &Path, src_dir: &Path) -> io::Result<AnalysisResult> {
    let (env_variables, env_definitions) =
        env_file_reader::process_env_file(get_file_reader(env_file)?)?;
    let mut src_env_occurrences = Vec::new();

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
            let mut envs = src_file_reader::process_src_file(get_file_reader(path)?)?;
            src_env_occurrences.append(&mut envs);
        }
    }

    let mut missing_env_occurrences = Vec::new();
    let mut unique_src_env_variables = HashSet::new();
    for occurrence in src_env_occurrences {
        if !env_variables.contains(&occurrence.name) {
            missing_env_occurrences.push(occurrence.clone());
        }
        unique_src_env_variables.insert(occurrence.name);
    }

    let unused_env_variables = env_variables.difference(&unique_src_env_variables);

    let mut unused_env_definitions: Vec<EnvDefinition> = Vec::new();
    for env_variable in unused_env_variables {
        let definition = env_definitions.iter().find(|def| def.name == *env_variable);
        match definition {
            Some(def) => unused_env_definitions.push(def.clone()),
            None => continue,
        }
    }

    Ok(AnalysisResult {
        unused: unused_env_definitions,
        missing: missing_env_occurrences,
    })
}

pub fn run(env_file: &Path, src_dir: &Path) -> io::Result<()> {
    let result = analyze(env_file, src_dir)?;

    for env_variable in &result.unused {
        println!(
            "Unused env variable: {} ({}:{})",
            env_variable.name,
            env_file.display(),
            env_variable.line + 1
        );
    }
    println!();

    for occurrence in &result.missing {
        println!(
            "Missing env variable: {} (at line {}, column {})",
            occurrence.name, occurrence.line, occurrence.column
        );
    }

    Ok(())
}
