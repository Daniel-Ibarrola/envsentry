use std::fs;
use tempfile::tempdir;

use envsentry::analyze;

#[test]
fn finds_unused_and_missing_env_vars() {
    let dir = tempdir().unwrap();

    let env_file = dir.path().join(".env");
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(&env_file, "USED_KEY=value\nUNUSED_KEY=value\n").unwrap();

    fs::write(
        src_dir.join("main.rs"),
        r#"
        use std::env;

        fn main() {
            let _ = env::var("USED_KEY");
            let _ = env::var("MISSING_KEY");
        }
        "#,
    )
    .unwrap();

    let result = analyze(&env_file, &src_dir).unwrap();

    assert_eq!(result.unused.len(), 1);
    assert_eq!(result.unused[0].name, "UNUSED_KEY");
    assert_eq!(result.unused[0].line, 1);

    assert!(result.missing.iter().any(|occ| occ.name == "MISSING_KEY"));
    assert!(result
        .missing
        .iter()
        .any(|occ| occ.name == "MISSING_KEY" && occ.file_path.contains("main.rs")));
    assert!(!result.missing.iter().any(|occ| occ.name == "USED_KEY"));
}
