use crate::error::SmtError;
use clap::Parser;
use std::path::PathBuf;

/// Args struct for CLI argument parsing with clap
#[derive(Parser, Debug)]
#[command(name = "smt")]
#[command(about = "Sort Markdown Tables", long_about = None)]
pub struct Args {
    /// Input files or glob patterns
    pub inputs: Vec<String>,

    /// Sort files in-place
    #[arg(short, long, conflicts_with_all = ["write", "check"])]
    pub in_place: bool,

    /// Write output to a specific file
    #[arg(short, long, conflicts_with_all = ["in_place", "check"])]
    pub write: Option<PathBuf>,

    /// Append to output file (requires --write)
    #[arg(long, requires = "write")]
    pub append: bool,

    /// Check if tables are sorted without modifying files
    #[arg(long, conflicts_with_all = ["in_place", "write"])]
    pub check: bool,

    /// Print verbose output
    #[arg(long)]
    pub verbose: bool,
}

/// InputSource represents where input comes from
#[derive(Debug, Clone)]
pub enum InputSource {
    Stdin,
    Files(Vec<PathBuf>),
}

/// OutputTarget represents where output goes
#[derive(Debug, Clone)]
pub enum OutputTarget {
    Stdout,
    InPlace,
    File { path: PathBuf, append: bool },
}

/// Parse command-line arguments and validate flag combinations
pub fn parse_args() -> Result<(InputSource, OutputTarget, bool, bool), SmtError> {
    let args = Args::parse();

    // Determine input source
    let input_source = detect_input_source(args.inputs.clone())?;

    // Determine output target
    let output_target = if args.in_place {
        OutputTarget::InPlace
    } else if let Some(path) = args.write {
        OutputTarget::File {
            path,
            append: args.append,
        }
    } else {
        OutputTarget::Stdout
    };

    Ok((input_source, output_target, args.check, args.verbose))
}

/// Expand glob patterns to file paths
pub fn expand_globs(patterns: Vec<String>) -> Result<Vec<PathBuf>, SmtError> {
    use glob::glob as glob_expand;

    if patterns.is_empty() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();

    for pattern in patterns {
        let glob_results = glob_expand(&pattern).map_err(|_| SmtError::NoFilesMatched {
            pattern: pattern.clone(),
        })?;

        let mut pattern_files = Vec::new();
        for entry in glob_results {
            match entry {
                Ok(path) => pattern_files.push(path),
                Err(_) => {
                    return Err(SmtError::NoFilesMatched {
                        pattern: pattern.clone(),
                    })
                }
            }
        }

        if pattern_files.is_empty() {
            return Err(SmtError::NoFilesMatched {
                pattern: pattern.clone(),
            });
        }

        files.extend(pattern_files);
    }

    Ok(files)
}

/// Detect input source from command-line arguments
pub fn detect_input_source(inputs: Vec<String>) -> Result<InputSource, SmtError> {
    use std::io::IsTerminal;

    if inputs.is_empty() {
        // No input files provided
        if std::io::stdin().is_terminal() {
            // TTY input: no inputs and stdin is a TTY (interactive)
            // In this case, main.rs should print help and exit 0
            // For now, return Stdin and let main.rs handle it
            Ok(InputSource::Stdin)
        } else {
            // Non-TTY input: read from stdin
            Ok(InputSource::Stdin)
        }
    } else {
        // Input files provided: expand globs
        let files = expand_globs(inputs)?;

        // Validate: if we're using --write, can't have multiple files
        // (This is checked in main.rs)

        Ok(InputSource::Files(files))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing_help() {
        // We can't easily test help output, but we can test basic parsing
        // This would normally require running the binary itself
    }

    #[test]
    fn test_expand_globs_empty() {
        let result = expand_globs(vec![]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_expand_globs_single_pattern() {
        // Create a test file temporarily
        let test_dir = tempfile::TempDir::new().unwrap();
        let test_file = test_dir.path().join("test.md");
        std::fs::write(&test_file, "# Test").unwrap();

        let pattern = format!("{}/*.md", test_dir.path().display());
        let result = expand_globs(vec![pattern]);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_expand_globs_no_match() {
        let result = expand_globs(vec!["nonexistent_*.md".to_string()]);
        assert!(result.is_err());
        match result.unwrap_err() {
            SmtError::NoFilesMatched { pattern } => {
                assert_eq!(pattern, "nonexistent_*.md");
            }
            _ => panic!("Expected NoFilesMatched error"),
        }
    }

    #[test]
    fn test_detect_input_source_no_inputs() {
        // This test is tricky because it depends on whether stdin is a TTY
        // We can only test the logic path where inputs are provided
        let result = detect_input_source(vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_input_source_with_files() {
        let test_dir = tempfile::TempDir::new().unwrap();
        let test_file = test_dir.path().join("test.md");
        std::fs::write(&test_file, "# Test").unwrap();

        let pattern = format!("{}/*.md", test_dir.path().display());
        let result = detect_input_source(vec![pattern]);
        assert!(result.is_ok());
        match result.unwrap() {
            InputSource::Files(files) => {
                assert!(!files.is_empty());
            }
            InputSource::Stdin => panic!("Expected Files variant"),
        }
    }

    #[test]
    fn test_input_source_enum() {
        let source = InputSource::Stdin;
        match source {
            InputSource::Stdin => {
                // Success
            }
            InputSource::Files(_) => panic!("Expected Stdin"),
        }
    }

    #[test]
    fn test_output_target_stdout() {
        let target = OutputTarget::Stdout;
        match target {
            OutputTarget::Stdout => {
                // Success
            }
            _ => panic!("Expected Stdout"),
        }
    }

    #[test]
    fn test_output_target_in_place() {
        let target = OutputTarget::InPlace;
        match target {
            OutputTarget::InPlace => {
                // Success
            }
            _ => panic!("Expected InPlace"),
        }
    }

    #[test]
    fn test_output_target_file() {
        let target = OutputTarget::File {
            path: PathBuf::from("output.md"),
            append: false,
        };
        match target {
            OutputTarget::File { path, append } => {
                assert_eq!(path, PathBuf::from("output.md"));
                assert!(!append);
            }
            _ => panic!("Expected File"),
        }
    }

    #[test]
    fn test_output_target_file_append() {
        let target = OutputTarget::File {
            path: PathBuf::from("output.md"),
            append: true,
        };
        match target {
            OutputTarget::File { path, append } => {
                assert_eq!(path, PathBuf::from("output.md"));
                assert!(append);
            }
            _ => panic!("Expected File with append=true"),
        }
    }
}
