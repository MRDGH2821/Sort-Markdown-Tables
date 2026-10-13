# Rust EditorConfig resolver findings

**Question:** Does `editor-config` (package `editor-config`, Rust crate `editor_config`) resolve effective EditorConfig rules for the output path as requested in [issue #1](https://github.com/MRDGH2821/Sort-Markdown-Tables/issues/1)?

## Finding

**No.** `editor-config` parses one supplied `.editorconfig` file and looks up values by exact section and key. It does not locate ancestor files, match section globs, or merge nested configs.
It accepts `end_of_line = lf`, `crlf`, and `cr`, but not `unset`, so it cannot resolve the effective setting for an output path by itself. [Crate docs](https://docs.rs/editor-config/latest/editor_config/) · [parser source](https://docs.rs/editor-config/latest/src/editor_config/parser.rs.html)

## Requirement-by-requirement

- **Ancestor search and `root = true`: No.** `from_file` reads only its explicit file; the parser has no ancestor traversal for `root` to control. [Crate docs](https://docs.rs/editor-config/latest/editor_config/) · [parser source](https://docs.rs/editor-config/latest/src/editor_config/parser.rs.html)
- **Nested config merge and precedence: No.** Parsed sections and properties belong to one file; there is no multi-file merge. [Parser source](https://docs.rs/editor-config/latest/src/editor_config/parser.rs.html)
- **Section glob matching: No.** `get_property(section, property)` looks up the exact supplied section string. [Crate docs](https://docs.rs/editor-config/latest/editor_config/)
- **`end_of_line` values: Partial.** The parser accepts `lf`, `crlf`, and `cr`, but does not accept `unset`. [Parser source](https://docs.rs/editor-config/latest/src/editor_config/parser.rs.html)

The crate docs describe it as a parser that leaves application behavior to its caller. Its example retrieves the literal `*` section rather than resolving a target path. [Crate docs](https://docs.rs/editor-config/latest/editor_config/)

## Specification behavior relevant to the issue

The specification searches from the target file's directory toward filesystem root, stopping only at `root = true`. Closer configs override ancestors; later sections override earlier sections. [File processing](https://spec.editorconfig.org/#file-processing)

Sections use filepath globs, `unset` removes a property's prior effect, and `end_of_line` accepts `lf`, `cr`, and `crlf`. [Glob expressions](https://spec.editorconfig.org/#glob-expressions) · [Supported pairs](https://spec.editorconfig.org/#supported-pairs)

Using `editor-config` would require implementing path lookup, glob matching, and merge precedence outside the crate. That resolver must also preserve per-line input endings when no effective EOL value remains. [Issue #1](https://github.com/MRDGH2821/Sort-Markdown-Tables/issues/1)

## Alternatives

- **`editorconfig` 1.0.0 (`editorconfig` crate): closer fit for filesystem resolution.** Its `get_config(&Path)` walks ancestors, matches sections, merges nearer configs, and stops at `root = true`. [API](https://docs.rs/editorconfig/1.0.0/editorconfig/) · [source](https://docs.rs/editorconfig/1.0.0/src/editorconfig/lib.rs.html). **Caveat:** it treats `unset` as an ordinary value instead of clearing the previous setting. [Source](https://docs.rs/editorconfig/1.0.0/src/editorconfig/lib.rs.html)
- **`editorconfig-parser` 0.0.4 (`editorconfig_parser` crate): useful for per-config parsing and glob matching, but not filesystem resolution.** It exposes typed `Unset`, but `EditorConfig::resolve(path)` collapses `Unset` to `None`; inspect matching section properties directly to preserve cross-file unset precedence. It does not locate or merge ancestor configs. [API/source](https://docs.rs/editorconfig-parser/latest/src/editorconfig_parser/lib.rs.html) · [README](https://github.com/oxc-project/editorconfig-parser)

## Recommendation

Use `editorconfig-parser` for per-config parsing and section matching. Implement ancestor traversal and cross-file precedence in the application, retaining the distinction between `unset` and an absent property.
