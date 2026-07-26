//! Command-line interface: **`clap`** argument definitions, glob expansion,
//! stdin-vs-files routing (`InputSource`), and stdout / file / in-place targets
//! (`OutputTarget`).
//!
//! The process entrypoint parses with [`finalize_cli`] (or [`parse_args`] wrapping
//! [`Args::parse`]).
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
    File {
        path: PathBuf,
        append: bool,
    },
}

/// Maps parsed flags to the output destination (stdout, file, or in-place).
fn output_target_from_args(args: &Args) -> OutputTarget {
    if args.in_place {
        OutputTarget::InPlace
    } else if let Some(path) = args.write.clone() {
        OutputTarget::File {
            path,
            append: args.append,
        }
    } else {
        OutputTarget::Stdout
    }
}

/// Finalize routing and validation from an already-parsed [`Args`] value.
///
/// Used by [`parse_args`] and by **binary** tests (and `main`) via
/// [`Args::try_parse_from`].
pub fn finalize_cli(args: Args) -> Result<(InputSource, OutputTarget, bool, bool), SmtError> {
    let input_source = detect_input_source(args.inputs.clone())?;
    let output_target = output_target_from_args(&args);

    // Additional validation not expressible purely in clap attributes.
    //
    // * `--write` with multiple input files is not allowed.
    //
    // * `--in-place` with stdin is not allowed.
    match (&output_target, &input_source) {
        (OutputTarget::File { .. }, InputSource::Files(files)) if files.len() > 1 => {
            return Err(SmtError::WriteWithMultipleFiles);
        },
        (OutputTarget::InPlace, InputSource::Stdin) => {
            return Err(SmtError::InPlaceWithStdin);
        },
        _ => { },
    }
    Ok((input_source, output_target, args.check, args.verbose))
}

/// Parse command-line arguments and validate flag combinations
pub fn parse_args() -> Result<(InputSource, OutputTarget, bool, bool), SmtError> {
    finalize_cli(Args::parse())
}

/// Expand glob patterns to file paths
pub fn expand_globs(patterns: Vec<String>) -> Result<Vec<PathBuf>, SmtError> {
    use glob::glob as glob_expand;

    if patterns.is_empty() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for pattern in patterns {
        let glob_results =
            glob_expand(&pattern).map_err(|_| SmtError::NoFilesMatched { pattern: pattern.clone() })?;
        let mut pattern_files = Vec::new();
        for entry in glob_results {
            match entry {
                Ok(path) => pattern_files.push(path),
                Err(_) => {
                    return Err(SmtError::NoFilesMatched { pattern: pattern.clone() })
                },
            }
        }
        if pattern_files.is_empty() {
            return Err(SmtError::NoFilesMatched { pattern: pattern.clone() });
        }
        files.extend(pattern_files);
    }
    Ok(files)
}

/// Resolve positional arguments into [`InputSource`]: either
/// [`InputSource::Stdin`] when `inputs` is empty (TTY vs pipe is opaque here;
/// callers use [`should_print_help_when_stdin_tty`]).
pub fn detect_input_source(inputs: Vec<String>) -> Result<InputSource, SmtError> {
    use std::io::IsTerminal;

    if inputs.is_empty() {
        // No input files provided
        if std::io::stdin().is_terminal() {
            // TTY input: no inputs and stdin is a TTY (interactive) In this case, main.rs
            // should print help and exit 0 For now, return Stdin and let main.rs handle it
            Ok(InputSource::Stdin)
        } else {
            // Non-TTY input: read from stdin
            Ok(InputSource::Stdin)
        }
    } else {
        // Input files provided: expand globs
        let files = expand_globs(inputs)?;

        // Validate: if we're using --write, can't have multiple files (This is checked in
        // main.rs)
        Ok(InputSource::Files(files))
    }
}

/// Returns true when the process should print help and exit successfully.
///
/// This applies when argv produced no explicit input files (`InputSource::Stdin`)
/// **and** the stdin handle appears to be a terminal (interactive use with no
/// piped content).
///
/// Fully unit-testable path for TTY-vs-pipe branching; cover end-to-end behavior
/// with a PTY in integration tests.
#[must_use]
pub fn should_print_help_when_stdin_tty(input_source: &InputSource, stdin_is_tty: bool) -> bool {
    matches!(input_source, InputSource::Stdin) && stdin_is_tty
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse_args_from(argv: &[&str]) -> Result<Args, clap::Error> {
        Args::try_parse_from(argv.iter().copied())
    }

    #[test]
    fn parse_flags_defaults_and_verbose() {
        let a = parse_args_from(&["smt"]).unwrap();
        assert!(a.inputs.is_empty());
        assert!(!a.in_place);
        assert!(a.write.is_none());
        assert!(!a.append);
        assert!(!a.check);
        assert!(!a.verbose);
        let a = parse_args_from(&["smt", "--verbose"]).unwrap();
        assert!(a.verbose);
    }

    #[test]
    fn parse_flags_check_exclusive_with_place_and_write() {
        assert!(parse_args_from(&["smt", "--check", "-i"]).is_err());
        assert!(parse_args_from(&["smt", "--check", "-w", "out.md"]).is_err());
    }

    #[test]
    fn parse_flags_in_place_conflicts_with_write() {
        assert!(parse_args_from(&["smt", "-i", "-w", "out.md"]).is_err());
    }

    #[test]
    fn parse_flags_append_requires_write() {
        assert!(parse_args_from(&["smt", "--append"]).is_err());
        let a = parse_args_from(&["smt", "-w", "out.md", "--append"]).unwrap();
        assert!(a.append);
        assert_eq!(a.write.as_ref().unwrap(), &PathBuf::from("out.md"));
    }

    #[test]
    fn parse_flags_meaningful_combinations_succeed() {
        let a = parse_args_from(&["smt", "a.md", "-i"]).unwrap();
        assert_eq!(a.inputs, vec!["a.md"]);
        assert!(a.in_place);
        let a = parse_args_from(&["smt", "x.md", "-w", "out.md"]).unwrap();
        assert_eq!(a.inputs, vec!["x.md"]);
        assert_eq!(a.write.as_ref().unwrap(), &PathBuf::from("out.md"));
        let a = parse_args_from(&["smt", "x.md", "-w", "out.md", "--append"]).unwrap();
        assert!(a.append);
        let a = parse_args_from(&["smt", "t.md", "--check"]).unwrap();
        assert!(a.check);
    }

    #[test]
    fn output_target_maps_stdout_write_in_place_append() {
        let a = parse_args_from(&["smt"]).unwrap();
        assert!(matches!(output_target_from_args(&a), OutputTarget::Stdout));
        let a = parse_args_from(&["smt", "-i", "f.md"]).unwrap();
        assert!(matches!(output_target_from_args(&a), OutputTarget::InPlace));
        let a = parse_args_from(&["smt", "f.md", "-w", "o.md"]).unwrap();
        match output_target_from_args(&a) {
            OutputTarget::File { path, append } => {
                assert_eq!(path, PathBuf::from("o.md"));
                assert!(!append);
            },
            _ => panic!("expected File"),
        }
        let a = parse_args_from(&["smt", "f.md", "-w", "o.md", "--append"]).unwrap();
        match output_target_from_args(&a) {
            OutputTarget::File { path, append } => {
                assert_eq!(path, PathBuf::from("o.md"));
                assert!(append);
            },
            _ => panic!("expected File append"),
        }
    }

    #[test]
    fn finalize_errors_in_place_with_stdin_no_inputs() {
        let args = parse_args_from(&["smt", "-i"]).unwrap();
        let err = finalize_cli(args).unwrap_err();
        assert!(matches!(err, SmtError::InPlaceWithStdin));
    }

    #[test]
    fn finalize_errors_write_with_multiple_expanded_files() {
        let dir = tempfile::TempDir::new().unwrap();
        let f1 = dir.path().join("a.md");
        let f2 = dir.path().join("b.md");
        std::fs::write(&f1, "# a").unwrap();
        std::fs::write(&f2, "# b").unwrap();
        let pat = format!("{}/*.md", dir.path().display());
        let argv = vec!["smt", pat.as_str(), "-w", "out.md"];
        let args = Args::try_parse_from(argv).unwrap();
        let err = finalize_cli(args).unwrap_err();
        assert!(matches!(err, SmtError::WriteWithMultipleFiles));
    }

    #[test]
    fn finalize_ok_single_file_with_write() {
        let dir = tempfile::TempDir::new().unwrap();
        let f = dir.path().join("only.md");
        std::fs::write(&f, "# x").unwrap();
        let argv = vec!["smt", f.to_str().unwrap(), "-w", "out.md"];
        let args = Args::try_parse_from(argv).unwrap();
        let (src, out, check, verbose) = finalize_cli(args).unwrap();
        assert!(!check && !verbose);
        match (src, out) {
            (InputSource::Files(files), OutputTarget::File { path, append }) => {
                assert_eq!(files.len(), 1);
                assert_eq!(files[0], f);
                assert_eq!(path, PathBuf::from("out.md"));
                assert!(!append);
            },
            _ => panic!("unexpected routing"),
        }
    }

    #[test]
    fn expand_globs_literal_path_with_dashes_and_spaces() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("my-file (copy).md");
        std::fs::write(&path, "# x").unwrap();
        let got = expand_globs(vec![path.to_string_lossy().into_owned()]).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0], path);
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
            },
            _ => panic!("Expected NoFilesMatched error"),
        }
    }

    #[test]
    fn test_detect_input_source_no_inputs() {
        // Empty inputs always resolve to `InputSource::Stdin` (TTY vs non-TTY is not
        // asserted here).
        let result = detect_input_source(vec![]).unwrap();
        assert!(matches!(result, InputSource::Stdin));
    }

    #[test]
    fn stdin_tty_help_predicate_matches_routing_expectations() {
        assert!(should_print_help_when_stdin_tty(&InputSource::Stdin, true));
        assert!(!should_print_help_when_stdin_tty(&InputSource::Stdin, false));
        assert!(!should_print_help_when_stdin_tty(&InputSource::Files(vec![PathBuf::from("x.md")]), true,));
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
            },
            InputSource::Stdin => panic!("Expected Files variant"),
        }
    }

    #[test]
    fn test_input_source_enum() {
        let source = InputSource::Stdin;
        match source {
            InputSource::Stdin => {
                // Success
            },
            InputSource::Files(_) => panic!("Expected Stdin"),
        }
    }

    #[test]
    fn test_output_target_stdout() {
        let target = OutputTarget::Stdout;
        match target {
            OutputTarget::Stdout => {
                // Success
            },
            _ => panic!("Expected Stdout"),
        }
    }

    #[test]
    fn test_output_target_in_place() {
        let target = OutputTarget::InPlace;
        match target {
            OutputTarget::InPlace => {
                // Success
            },
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
            },
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
            },
            _ => panic!("Expected File with append=true"),
        }
    }
}
