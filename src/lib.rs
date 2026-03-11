//! This module provides the main functionality for analyzing environment variables
//! in a source code directory and comparing them with an environment file.

mod env_file_reader;
mod src_file_reader;

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

/// Represents the results of an environment variable analysis.
#[derive(Debug)]
pub struct AnalysisResult {
    /// Environment variables defined in the env file but not used in the source code.
    pub unused: Vec<env_file_reader::EnvDefinition>,
    /// Environment variables used in the source code but not defined in the env file.
    pub missing: Vec<src_file_reader::EnvOccurrence>,
}

/// Analyzes the source directory and compares environment variable usage with the provided env file.
///
/// # Errors
///
/// Returns an `io::Result` if there are issues reading files or walking the directory.
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
            let mut envs = src_file_reader::process_src_file(
                get_file_reader(path)?,
                path.to_str().unwrap_or(""),
            )?;
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

    let mut unused_env_definitions: Vec<env_file_reader::EnvDefinition> = Vec::new();
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

/// Runs the analysis and prints the results to standard output.
///
/// # Errors
///
/// Returns an `io::Result` if there are issues during the analysis process.
pub fn run(env_file: &Path, src_dir: &Path) -> io::Result<()> {
    let result = analyze(env_file, src_dir)?;

    for env_variable in &result.unused {
        println!(
            "Unused env variable: \n\t{} ({}:{})",
            env_variable.name,
            env_file.display(),
            env_variable.line + 1
        );
    }
    println!();

    for occurrence in &result.missing {
        println!(
            "Missing env variable: \n\t{} ({}:{}:{})",
            occurrence.name, occurrence.file_path, occurrence.line, occurrence.column
        );
    }

    Ok(())
}
