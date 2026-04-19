//! Diagnostic types for environment variable analysis.
//!
//! This module defines the error structures used to report missing, unused,
//! or empty environment variables, integrating with `miette` for pretty-printed diagnostics.

use miette::{Diagnostic, NamedSource, SourceSpan};
use std::sync::Arc;
use thiserror::Error;

/// Error returned when an environment variable used in the source code is missing from the environment file.
#[derive(Debug, Error, Diagnostic)]
#[error("Missing env variable `{name}`")]
#[diagnostic(
    code(envsentry::missing_env),
    help("Define this variable in your .env file")
)]
pub struct MissingEnvError {
    /// The name of the missing environment variable.
    pub name: String,
    /// The location in the source code where the variable is used.
    #[label("used here")]
    pub location: SourceSpan,
    /// The source code where the usage was found.
    #[source_code]
    pub src: NamedSource<Arc<String>>,
}

/// Error returned when an environment variable defined in the environment file is not used in the source code.
#[derive(Debug, Error, Diagnostic)]
#[error("Unused env variable `{name}`")]
#[diagnostic(
    code(envsentry::unused_env),
    help("Remove this variable from your .env file")
)]
pub struct UnusedEnvError {
    /// The name of the unused environment variable.
    pub name: String,
    /// The location in the environment file where the variable is defined.
    #[label("defined here")]
    pub location: SourceSpan,
    /// The source content of the environment file.
    #[source_code]
    pub src: NamedSource<Arc<String>>,
}

/// Error returned when an environment variable is defined in the environment file but has no value.
#[derive(Debug, Error, Diagnostic)]
#[error("Empty env variable `{name}`")]
#[diagnostic(
    code(envsentry::empty_env),
    help("Add a value to this variable in your .env file, or remove it if it's not needed")
)]
pub struct EmptyEnvError {
    /// The name of the empty environment variable.
    pub name: String,
    /// The location in the environment file where the variable is defined.
    #[label("defined here")]
    pub location: SourceSpan,
    /// The source content of the environment file.
    #[source_code]
    pub src: NamedSource<Arc<String>>,
}
