//! This module provides the main functionality for analyzing environment variables
//! in a source code directory and comparing them with an environment file.

mod diagnostics;
mod env_file_reader;
mod src_file_reader;

use crate::diagnostics::{EmptyEnvError, MissingEnvError, UnusedEnvError};
use miette::{NamedSource, Report};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Cursor};
use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;

/// Represents the results of an environment variable analysis.
#[derive(Debug)]
pub struct AnalysisResult {
    /// Environment variables defined in the env file but not used in the source code.
    pub unused: Vec<env_file_reader::EnvDefinition>,
    // Environment variables defined in the env file but with empty values.
    pub empty_vars: Vec<env_file_reader::EnvDefinition>,
    /// Environment variables used in the source code but not defined in the env file.
    pub missing: Vec<src_file_reader::EnvOccurrence>,
    /// The raw contents of the env file.
    pub env_file_contents: Arc<String>,
    /// Cached source file contents keyed by file path.
    pub source_cache: HashMap<String, Arc<String>>,
}

/// Analyzes the source directory and compares environment variable usage with the provided env file.
///
/// # Errors
///
/// Returns an `io::Result` if there are issues reading files or walking the directory.
pub fn analyze(env_file: &Path, src_dir: &Path) -> io::Result<AnalysisResult> {
    let env_file_contents = Arc::new(fs::read_to_string(env_file).map_err(|e| {
        io::Error::new(e.kind(), format!("failed to open {}: {}", env_file.display(), e))
    })?);
    let (env_variables, env_definitions) =
        env_file_reader::process_env_file(Cursor::new(env_file_contents.as_bytes()))?;

    let mut src_env_occurrences = Vec::new();
    let mut source_cache: HashMap<String, Arc<String>> = HashMap::new();

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
            let path_str = path.to_str().unwrap_or("").to_string();
            let contents = Arc::new(fs::read_to_string(path).map_err(|e| {
                io::Error::new(e.kind(), format!("failed to open {}: {}", path.display(), e))
            })?);
            let mut envs = src_file_reader::process_src_file(
                Cursor::new(contents.as_bytes()),
                &path_str,
            )?;
            if !envs.is_empty() {
                src_env_occurrences.append(&mut envs);
                source_cache.insert(path_str, contents);
            }
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

    let empty_vars = env_definitions
        .iter()
        .filter(|def| def.value.is_empty())
        .cloned()
        .collect::<Vec<_>>();

    Ok(AnalysisResult {
        unused: unused_env_definitions,
        missing: missing_env_occurrences,
        empty_vars,
        env_file_contents,
        source_cache,
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
        let err = UnusedEnvError {
            name: env_variable.name.clone(),
            location: (env_variable.span_start, env_variable.span_len).into(),
            src: NamedSource::new(env_file.to_string_lossy(), result.env_file_contents.clone()),
        };
        eprintln!("{:?}", Report::new(err));
    }

    for env_variable in &result.empty_vars {
        let err = EmptyEnvError {
            name: env_variable.name.clone(),
            location: (env_variable.span_start, env_variable.span_len).into(),
            src: NamedSource::new(env_file.to_string_lossy(), result.env_file_contents.clone()),
        };
        eprintln!("{:?}", Report::new(err));
    }

    for occurrence in &result.missing {
        let contents = result.source_cache
            .get(&occurrence.file_path)
            .cloned()
            .unwrap_or_default();

        let err = MissingEnvError {
            name: occurrence.name.clone(),
            location: (occurrence.span_start, occurrence.span_len).into(),
            src: NamedSource::new(occurrence.file_path.clone(), contents),
        };

        eprintln!("{:?}", Report::new(err));
    }
    Ok(())
}
