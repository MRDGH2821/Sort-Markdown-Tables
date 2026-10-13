use crate::{error::SmtError, parser::LineEnding};
use editorconfig_parser::{EditorConfig, EditorConfigProperty, EndOfLine};
use std::{fs, path::Path};

/// Resolve `end_of_line` for an output path, searching ancestor configs until
/// the first applicable value or `root = true`.
pub fn resolve_end_of_line(path: &Path) -> Result<Option<LineEnding>, SmtError> {
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|source| SmtError::Io { source })?
            .join(path)
    };
    let mut directory = absolute_path.parent();

    while let Some(config_dir) = directory {
        let config_path = config_dir.join(".editorconfig");
        if config_path.is_file() {
            let contents =
                fs::read_to_string(&config_path).map_err(|source| SmtError::Io { source })?;
            let config = EditorConfig::parse(&contents);
            let relative_path = absolute_path
                .strip_prefix(config_dir)
                .unwrap_or(&absolute_path);
            let mut eol = EditorConfigProperty::None;
            for section in config.sections() {
                if section
                    .matcher
                    .as_ref()
                    .is_some_and(|matcher| matcher.is_match(relative_path))
                    && !matches!(section.properties.end_of_line, EditorConfigProperty::None)
                {
                    eol = section.properties.end_of_line.clone();
                }
            }
            match eol {
                EditorConfigProperty::Value(value) => return Ok(Some(value.into())),
                EditorConfigProperty::Unset => return Ok(None),
                EditorConfigProperty::None if config.root() => return Ok(None),
                EditorConfigProperty::None => {}
            }
        }
        directory = config_dir.parent();
    }

    Ok(None)
}

impl From<EndOfLine> for LineEnding {
    fn from(value: EndOfLine) -> Self {
        match value {
            EndOfLine::Lf => Self::Lf,
            EndOfLine::Crlf => Self::CrLf,
            EndOfLine::Cr => Self::Cr,
        }
    }
}
