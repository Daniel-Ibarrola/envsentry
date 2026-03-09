use std::collections::HashSet;
use std::io;
use std::io::BufRead;

pub fn process_env_file<R: BufRead>(reader: R) -> io::Result<HashSet<String>> {
    let mut env_variables = HashSet::new();

    for line in reader.lines() {
        let line = line?;
        if let Some((key, _value)) = line.split_once('=') {
            env_variables.insert(key.to_string());
        }
    }

    Ok(env_variables)
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
}
