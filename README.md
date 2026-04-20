# Envsentry

A command-line tool for analyzing environment variables in Rust source code and comparing them with an environment file.

## Features

- **Find Unused Variables:** Identifies environment variables defined in your `.env` file that are not used in your source code.
- **Identify Missing Variables:** Detects calls to `env::var`, `std::env::var`, `env!`, etc., in your code where the corresponding variable is missing from your `.env` file.
- **Detailed Reports:** Provides precise file paths, line numbers, and column numbers for all occurrences and definitions.
- **Fast and Efficient:** Uses regular expressions for rapid source code scanning and `walkdir` for efficient file traversal.

## Installation

You can install `envsentry` directly from [crates.io](https://crates.io) using `cargo`:

```bash
cargo install envsentry
```

Alternatively, you can build it from source:

```bash
git clone https://github.com/yourusername/envsentry.git
cd envsentry
cargo install --path .
```

## Usage

To run `envsentry`, you need to provide the path to your environment file and the directory containing your source code.

```bash
envsentry --env-file .env --src-dir ./src
```

### Options

- `-e, --env-file <ENV_FILE>`: Path to the environment file (e.g., `.env`) containing all the variables to check against.
- `-s, --src-dir <SRC_DIR>`: Path to the directory containing the source code (it recursively scans for `.rs` files).
- `-h, --help`: Display help information.
- `-V, --version`: Display version information.

## Supported Syntax

`envsentry` scans for the following environment variable access patterns:

- `std::env::var("KEY")`
- `env::var("KEY")`
- `std::env::var_os("KEY")`
- `env::var_os("KEY")`
- `env!("KEY")`
- `option_env!("KEY")`
- `var("KEY")` (if imported)

It supports standard `.env` file formats, including:
- `KEY=VALUE`
- `export KEY=VALUE`
- Comments starting with `#`

## Example Output

```text
Running envsentry...
Environment file: .env
Source directory: ./src

Unused env variable: 
	DEBUG_MODE (.env:5)

Missing env variable: 
	API_SECRET (./src/auth.rs:42:25)
```

## Testing

To run the standard tests:

```bash
cargo test
```

### Performance Tests

The project includes a performance test suite to ensure efficiency with large projects. These tests are ignored by default and must be run manually:

```bash
cargo test --test run_performance -- --ignored
```

To see the performance metrics during execution:

```bash
cargo test --test run_performance -- --ignored --nocapture
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
