use std::fs;
use std::time::Instant;

use tempfile::tempdir;

use envsentry::analyze;

/// Performance test for the `analyze` function with a large-scale project simulation.
///
/// This test creates a temporary project structure with:
/// - A `.env` file containing 2,000 environment variables.
/// - A `src` directory containing 1,000 Rust source files.
/// - Each source file contains 20 calls to `env::var`, referencing a subset of the environment variables.
///
/// The test measures the time taken by `analyze` to process the entire project and asserts
/// that it completes within a reasonable threshold (5 seconds).

#[test]
#[ignore = "Performance test; run manually"]
fn analyze_large_project_performance() {
    let dir = tempdir().unwrap();

    let env_file = dir.path().join(".env");
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    let env_count = 2_000;
    let file_count = 1000;

    let mut env_contents = String::new();
    for i in 0..env_count {
        env_contents.push_str(&format!("KEY_{i}=value_{i}\n"));
    }
    fs::write(&env_file, env_contents).unwrap();

    for file_index in 0..file_count {
        let file_path = src_dir.join(format!("file_{file_index}.rs"));

        let mut file_contents = String::from(
            r#"
use std::env;

fn check() {
"#,
        );

        for key_index in 0..20 {
            file_contents.push_str(&format!(
                r#"    let _ = env::var("KEY_{}");
"#,
                (file_index * 20 + key_index) % env_count
            ));
        }

        file_contents.push_str(
            r#"}
"#,
        );

        fs::write(file_path, file_contents).unwrap();
    }

    let start = Instant::now();
    let result = analyze(&env_file, &src_dir).unwrap();
    let elapsed = start.elapsed();

    println!("analyze() took: {:?}", elapsed);
    println!("unused: {}", result.unused.len());
    println!("missing: {}", result.missing.len());

    assert!(
        elapsed.as_secs_f64() < 5.0,
        "analyze() was too slow: {:?}",
        elapsed
    );
}
