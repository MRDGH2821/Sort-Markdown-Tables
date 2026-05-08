use std::path::PathBuf;
use thiserror::Error;

/// SourceLocation wraps an optional path for error reporting
#[derive(Debug, Clone)]
pub struct SourceLocation(pub Option<PathBuf>);

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(path) => write!(f, "{}", path.display()),
            None => write!(f, "<stdin>"),
        }
    }
}

/// SmtError represents all error types in the smt tool
#[derive(Error, Debug)]
pub enum SmtError {
    // CLI errors
    #[error("--write cannot be used with multiple input files")]
    WriteWithMultipleFiles,

    #[error("--append requires --write")]
    AppendWithoutWrite,

    #[error("--in-place cannot be used with stdin")]
    InPlaceWithStdin,

    #[error("no files matched pattern \"{pattern}\"")]
    NoFilesMatched { pattern: String },

    // Parse errors
    #[error("{path}:{line}: smt comment is not followed by a table")]
    CommentWithoutTable { path: SourceLocation, line: usize },

    #[error("{path}:{line}: duplicate smt comment (previous at line {previous_line})")]
    DuplicateComment {
        path: SourceLocation,
        line: usize,
        previous_line: usize,
    },

    #[error("{path}:{line}: unknown option \"{key}\" in smt comment")]
    UnknownOption {
        path: SourceLocation,
        line: usize,
        key: String,
    },

    #[error(
        "{path}:{line}: invalid value \"{value}\" for option \"{key}\" (expected: {expected})"
    )]
    InvalidOptionValue {
        path: SourceLocation,
        line: usize,
        key: String,
        value: String,
        expected: String,
    },

    #[error("{path}:{line}: column must be >= 1 in smt comment")]
    ColumnZero { path: SourceLocation, line: usize },

    #[error("{path}:{line}: column must be a positive integer in smt comment")]
    ColumnNotInteger { path: SourceLocation, line: usize },

    #[error("{path}:{line}: column {column} is out of range (table has {actual} columns)")]
    ColumnOutOfRange {
        path: SourceLocation,
        line: usize,
        column: usize,
        actual: usize,
    },

    #[error("{path}:{line}: malformed table (missing separator row)")]
    MalformedTable { path: SourceLocation, line: usize },

    // I/O errors
    #[error("file not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
}

impl SmtError {
    /// Return the exit code for this error
    /// All errors map to exit code 2
    pub fn exit_code(&self) -> i32 {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_with_multiple_files_message() {
        let err = SmtError::WriteWithMultipleFiles;
        assert_eq!(
            err.to_string(),
            "--write cannot be used with multiple input files"
        );
    }

    #[test]
    fn test_append_without_write_message() {
        let err = SmtError::AppendWithoutWrite;
        assert_eq!(err.to_string(), "--append requires --write");
    }

    #[test]
    fn test_in_place_with_stdin_message() {
        let err = SmtError::InPlaceWithStdin;
        assert_eq!(err.to_string(), "--in-place cannot be used with stdin");
    }

    #[test]
    fn test_no_files_matched_message() {
        let err = SmtError::NoFilesMatched {
            pattern: "*.nonexistent".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "no files matched pattern \"*.nonexistent\""
        );
    }

    #[test]
    fn test_comment_without_table_message() {
        let err = SmtError::CommentWithoutTable {
            path: SourceLocation(Some(PathBuf::from("test.md"))),
            line: 5,
        };
        assert_eq!(
            err.to_string(),
            "test.md:5: smt comment is not followed by a table"
        );
    }

    #[test]
    fn test_comment_without_table_stdin() {
        let err = SmtError::CommentWithoutTable {
            path: SourceLocation(None),
            line: 5,
        };
        assert_eq!(
            err.to_string(),
            "<stdin>:5: smt comment is not followed by a table"
        );
    }

    #[test]
    fn test_duplicate_comment_message() {
        let err = SmtError::DuplicateComment {
            path: SourceLocation(Some(PathBuf::from("test.md"))),
            line: 10,
            previous_line: 5,
        };
        assert_eq!(
            err.to_string(),
            "test.md:10: duplicate smt comment (previous at line 5)"
        );
    }

    #[test]
    fn test_unknown_option_message() {
        let err = SmtError::UnknownOption {
            path: SourceLocation(Some(PathBuf::from("test.md"))),
            line: 3,
            key: "colum".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "test.md:3: unknown option \"colum\" in smt comment"
        );
    }

    #[test]
    fn test_invalid_option_value_message() {
        let err = SmtError::InvalidOptionValue {
            path: SourceLocation(Some(PathBuf::from("test.md"))),
            line: 3,
            key: "order".to_string(),
            value: "ascending".to_string(),
            expected: "asc, desc".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "test.md:3: invalid value \"ascending\" for option \"order\" (expected: asc, desc)"
        );
    }

    #[test]
    fn test_column_zero_message() {
        let err = SmtError::ColumnZero {
            path: SourceLocation(Some(PathBuf::from("test.md"))),
            line: 3,
        };
        assert_eq!(
            err.to_string(),
            "test.md:3: column must be >= 1 in smt comment"
        );
    }

    #[test]
    fn test_column_not_integer_message() {
        let err = SmtError::ColumnNotInteger {
            path: SourceLocation(Some(PathBuf::from("test.md"))),
            line: 3,
        };
        assert_eq!(
            err.to_string(),
            "test.md:3: column must be a positive integer in smt comment"
        );
    }

    #[test]
    fn test_column_out_of_range_message() {
        let err = SmtError::ColumnOutOfRange {
            path: SourceLocation(Some(PathBuf::from("test.md"))),
            line: 3,
            column: 5,
            actual: 3,
        };
        assert_eq!(
            err.to_string(),
            "test.md:3: column 5 is out of range (table has 3 columns)"
        );
    }

    #[test]
    fn test_malformed_table_message() {
        let err = SmtError::MalformedTable {
            path: SourceLocation(Some(PathBuf::from("test.md"))),
            line: 3,
        };
        assert_eq!(
            err.to_string(),
            "test.md:3: malformed table (missing separator row)"
        );
    }

    #[test]
    fn test_file_not_found_message() {
        let err = SmtError::FileNotFound {
            path: PathBuf::from("missing.md"),
        };
        assert_eq!(err.to_string(), "file not found: missing.md");
    }

    #[test]
    fn test_permission_denied_message() {
        let err = SmtError::PermissionDenied {
            path: PathBuf::from("/restricted/file.md"),
        };
        assert_eq!(err.to_string(), "permission denied: /restricted/file.md");
    }

    #[test]
    fn test_io_error_message() {
        let err = SmtError::Io {
            source: std::io::Error::new(std::io::ErrorKind::Other, "mock failure"),
        };
        assert!(
            err.to_string().starts_with("I/O error:"),
            "{err}"
        );
    }

    #[test]
    fn test_all_errors_exit_code_2() {
        let errors = vec![
            SmtError::WriteWithMultipleFiles,
            SmtError::AppendWithoutWrite,
            SmtError::InPlaceWithStdin,
            SmtError::NoFilesMatched {
                pattern: "test".to_string(),
            },
            SmtError::CommentWithoutTable {
                path: SourceLocation(None),
                line: 1,
            },
            SmtError::DuplicateComment {
                path: SourceLocation(None),
                line: 1,
                previous_line: 1,
            },
            SmtError::UnknownOption {
                path: SourceLocation(None),
                line: 1,
                key: "test".to_string(),
            },
            SmtError::InvalidOptionValue {
                path: SourceLocation(None),
                line: 1,
                key: "test".to_string(),
                value: "test".to_string(),
                expected: "test".to_string(),
            },
            SmtError::ColumnZero {
                path: SourceLocation(None),
                line: 1,
            },
            SmtError::ColumnNotInteger {
                path: SourceLocation(None),
                line: 1,
            },
            SmtError::ColumnOutOfRange {
                path: SourceLocation(None),
                line: 1,
                column: 1,
                actual: 1,
            },
            SmtError::MalformedTable {
                path: SourceLocation(None),
                line: 1,
            },
            SmtError::FileNotFound {
                path: PathBuf::from("test"),
            },
            SmtError::PermissionDenied {
                path: PathBuf::from("test"),
            },
            SmtError::Io {
                source: std::io::Error::new(std::io::ErrorKind::Other, "mock I/O failure"),
            },
        ];

        for err in errors {
            assert_eq!(err.exit_code(), 2, "Error: {}", err);
        }
    }
}
