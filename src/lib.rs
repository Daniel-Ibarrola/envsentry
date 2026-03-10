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
    pub missing: HashSet<String>,
}

pub fn analyze(env_file: &Path, src_dir: &Path) -> io::Result<AnalysisResult> {
    let (env_variables, env_definitions) =
        env_file_reader::process_env_file(get_file_reader(env_file)?)?;
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
            let mut envs = src_file_reader::process_src_file(get_file_reader(path)?)?;
            src_env_variables.append(&mut envs);
        }
    }

    let unique_src_env_variables: HashSet<String> = src_env_variables.into_iter().collect();
    let unused_env_variables = env_variables.difference(&unique_src_env_variables);
    let missing_env_variables = unique_src_env_variables.difference(&env_variables);

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
        missing: missing_env_variables.cloned().collect(),
    })
}

pub fn run(env_file: &Path, src_dir: &Path) -> io::Result<()> {
    let result = analyze(env_file, src_dir)?;

    for env_variable in &result.unused {
        println!(
            "Unused env variable: {} ({}:{})",
            env_variable.name,
            env_file.display(),
            env_variable.line
        );
    }
    println!();

    for env_variable in &result.missing {
        println!("Missing env variable: {}", env_variable);
    }

    Ok(())
}
