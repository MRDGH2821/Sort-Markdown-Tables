# AGENTS Instructions

This file provides guidance for AI coding assistants working with this project.

## MANDATORY: Action Logging

> **This is non-negotiable. Log before you start, log as you work, log when you finish.**

Every AI session MUST produce a log entry in `.agents/logs/YYYY-MM-DD.md`. This is not optional documentation — it is a **required action**, executed by the agent itself, not left to the human.

### Procedure

**Step 1 — Before touching any file:**

```bash
# Get today's filename
date '+%Y-%m-%d'   # e.g. 2026-03-16
```

- If `.agents/logs/YYYY-MM-DD.md` does not exist → create it with the header:
  ```markdown
  # AI Work Log - YYYY-MM-DD
  ```
- If it already exists → append to it (do NOT overwrite)

**Step 2 — Open your entry immediately:**

Append a new entry header with the current ISO timestamp and the user's prompt:

```markdown
## HH:MM:SS+TZ

### Prompt

> <exact user request, verbatim or faithfully paraphrased>

### Model

<model name and version> via <editor/tool> (e.g. Claude Sonnet 4.6 via opencode)
```

**Step 3 — Log each action as you perform it:**

After every meaningful action, append to the `### Actions` section. Do not batch everything at the end — if the session is interrupted, the log must still reflect what was done.

**Step 4 — Close the entry when done:**

Append the `### Outcome` section:

```markdown
### Outcome

<✅ / ⚠️ / ❌> <one-line summary of what was achieved or what failed>
```

---

### Log Format (AI-authored)

```markdown
## 2026-03-16T20:15:00+11:00

### Prompt

> Add tool-runner skill with Bun/Node fallback chains

### Model

Claude Sonnet 4.6 via opencode

### Actions

- Created `.agents/skills/tool-runner/SKILL.md` — main skill documentation with fallback patterns
- Created `.agents/skills/tool-runner/assets/tool-runner.sh` — standalone bash script for tool selection
- Created `.agents/skills/tool-runner/assets/validate-tools.sh` — validation script for tool availability
- Modified `AGENTS.md` — registered skill in Project Skills table
- Decision: used `command -v` over `which` for POSIX compliance across Linux/macOS/Windows

### Outcome

✅ Skill created and committed, all pre-commit hooks passed
```

---

### What Counts as a Loggable Action

**Always log:**

- Every file **created** — name, purpose, approximate scope
- Every file **modified** — name, what changed and why
- Every **decision** made — especially when choosing between alternatives
- Every **command run** with a non-trivial outcome (tool installs, test runs, linter results)
- Anything **rejected or changed** from the original approach, and the reason

**Do NOT log:**

- Trivial auto-fixes by pre-commit hooks (formatting, whitespace)
- Reading files for context (unless the read revealed something decision-relevant)
- Intermediate tool calls that produced no output or change

---

### Additional Materials

Place any other relevant documents (prompts, examples, references, generated docs) in the `.agents/` folder.

---

## MANDATORY: AI Co-authored-by Trailer

> **Every commit made with AI assistance MUST include a `Co-authored-by` trailer. No exceptions.**

**Format:**

```txt
Co-authored-by: <Model Name> via <Tool> <noreply@provider-domain>
```

**Provider noreply addresses:**

| Provider                | noreply address         |
| ----------------------- | ----------------------- |
| Anthropic (Claude)      | `noreply@anthropic.com` |
| OpenAI (GPT / o-series) | `noreply@openai.com`    |
| Google (Gemini)         | `noreply@google.com`    |
| Microsoft (Copilot)     | `noreply@microsoft.com` |
| Mistral                 | `noreply@mistral.ai`    |
| Meta (Llama)            | `noreply@meta.com`      |
| xAI (Grok)              | `noreply@x.ai`          |

**Examples:**

```txt
feat(precommit): add spell checking to commit messages

Co-authored-by: Claude Sonnet 4.6 via opencode <noreply@anthropic.com>
```

```txt
fix(cspell): resolve configuration issue

Co-authored-by: GPT-4o via Cursor <noreply@openai.com>
```

**Rules:**

- Use the **exact model name and version** you are running as (e.g. `Claude Sonnet 4.6`, not just `Claude`)
- Use the **tool name** as it is commonly known (e.g. `opencode`, `Cursor`, `Copilot`, `Zed`)
- If the model version is unknown, use the model family name (e.g. `Claude Sonnet`)
- One trailer per AI model involved
- **Never omit this trailer** when the commit was AI-assisted — this is how git history stays honest

## Project Context

- **Project Type**: Project generated from copier-mr-minimal
- **Key Technologies**: pre-commit hooks, MegaLinter, prek
- **Purpose**: Provides a standardized starting point for new projects with quality checks

## General Guidelines

### Communication

- Explain what you're doing and why before making changes
- Ask for clarification when requirements are ambiguous
- Provide context for decisions, especially when multiple approaches exist

### Code Quality

- Follow existing code style and conventions in the project
- Run linters and formatters before committing changes
- Ensure all changes pass pre-commit hooks

### File Operations

- Always check if a file exists before attempting to modify it
- Use appropriate tools to search for files rather than guessing paths
- Preserve file formatting and structure unless explicitly asked to change it

## AI Usage and Transparency

**IMPORTANT**: This project maintains full transparency about AI assistance.

### AI Assistance Guidelines

**AI can help with**:

- Boilerplate code and scaffolding
- Documentation and comments
- Test cases and test data
- Refactoring suggestions
- Bug fixes and debugging
- Code review and optimization suggestions
- Research and best practices

**Human must**:

- Review all AI-generated code thoroughly
- Test all functionality comprehensively
- Make final decisions on architecture and approach
- Approve all changes before committing
- Understand the code (never commit code you don't understand)

**Always**:

- Validate AI suggestions against project architecture (if such a document is present)
- Follow best coding practices and idioms
- Ensure code passes all tests and linters
- Document every action in `.agents/logs/` as described above

**Never**:

- Skip testing because "AI wrote it"
- Forget to write the log entry
- Rely solely on AI for architectural decisions

## Dev Environment Tips

- Use `--help` or `help` subcommand to get help on a command. It can even reveal hints on how to proceed ahead or optimize the number of steps.
- Check tool documentation before asking the user for configuration details

## Linting and Formatting

### MegaLinter

- Configuration is in `.mega-linter.yml`
- Run locally with: `bunx mega-linter-runner`
- Check reports in `megalinter-reports/` directory
- Not all linters need to pass - some are informational

### CSpell (Spell Checking)

- Configuration is in `.cspell.json`
- Add project-specific words to the `words` array
- Don't disable spell checking without good reason
- Both file content and commit messages are spell-checked

### treefmt

- Run `treefmt -vv` before every commit to format all supported file types (markdown, JSON, YAML, etc.)
- Must be run manually — it is not a pre-commit hook

## Commit Messages

### Format

- Follow Conventional Commits format: `<type>(<scope>): <description>` as given here - <https://www.conventionalcommits.org/en/v1.0.0/>
- Valid types: `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert`, `style`, `test`
- For valid scopes, refer to the `scopes` array in `cog.toml` — it is the source of truth.

### Examples

```txt
feat(precommit): add spell checking to commit messages
fix(cspell): resolve configuration issue
docs: update AGENTS.md with guidelines
chore(cspell): add technical terms to dictionary
```

## Troubleshooting

### Common Issues

**Pre-commit hooks failing on commit:**

- Read the error message — it usually points directly to the fix
- Try to fix the issue and retry the commit; do not skip hooks
- Fix formatting issues first (treefmt, whitespace)
- Then address spell checking and linting

**Spell check failures:**

- Add legitimate technical terms to `.cspell.json` `words` array
- Use proper capitalization for proper nouns
- Don't add obvious typos to the dictionary

**Template syntax errors:**

- Ensure template syntax is valid before committing
- Check for missing closing tags or brackets
- Test template rendering if applicable

### Getting Help

- Review existing configuration files for examples

## Best Practices

### Before Making Changes

1. Understand the current state of the project
2. Check if similar functionality already exists
3. Review relevant configuration files
4. Consider impact on users who will use this template

### When Adding Dependencies

- Prefer tools that don't require heavy installation
- Document installation steps clearly
- Consider cross-platform compatibility
- Update relevant configuration files

### Testing Changes

- Verify the project structure is correct
- Test on a clean environment if possible
- Ensure documentation is updated

Project specific instructions will be documented here.

## Project: `smt` (Sort Markdown Tables)

A Rust CLI tool that sorts markdown tables opted-in via `<!-- smt -->` HTML comments.

### Key Documents

- `.ai/PLAN.md` — Full project plan with all requirements, CLI interface, sorting behavior, error handling, testing strategy
- `.ai/ARCHITECTURE.md` — Module dependency diagram, data flow, data structures (Rust code), parser state machine, error type hierarchy

### Architecture Overview

```
src/
├── main.rs      # Entry point, orchestrates pipeline
├── cli.rs       # Clap args, validation, glob expansion
├── parser.rs    # Markdown parsing, comment detection, table extraction
├── sorter.rs    # Sort logic (numeric, lexicographic, case, direction)
├── writer.rs    # Output: stdout, file, in-place (atomic writes)
└── error.rs     # SmtError enum with thiserror
```

### Critical Rules

- **Atomicity**: On ANY error, no files are modified (two-phase: parse+sort all, then write all)
- **Stable sort**: Use `sort_by`, never `sort_unstable_by`
- **Exit codes**: 0=success, 1=unsorted (--check only), 2=user error
- **Comment must immediately precede table** (no blank lines between)
- **Tables without `<!-- smt -->` are left untouched**

### Crate Dependencies

- `clap` (4.x, derive) — CLI parsing
- `thiserror` (2.x) — Error types
- `anyhow` (1.x) — Error propagation
- `glob` (0.3.x) — File globbing
- `tempfile` — Atomic writes for `-i` mode
- NO `regex` — comment parsing is hand-rolled with `str` methods

### Testing

- Unit tests per module (parser, sorter, writer)
- Integration tests with `assert_cmd` + `predicates`
- Test fixtures in `tests/fixtures/` with `.md` + `.expected.md` pairs

### Git Workflow: Commit Scopes

**CRITICAL**: Before committing code, ensure the commit scope exists in `cog.toml`.

**Process** (required before EVERY feature commit):

1. Check `cog.toml` scopes list for your commit scope
2. If missing → add it alphabetically to the scopes array
3. Commit `cog.toml` first with message: `chore(cocogitto): add {scope} scope`
4. Then commit your code changes with the proper scope

**Example**:

```bash
# 1. Add scope to cog.toml if missing
edit cog.toml  # add "sorter" to scopes list

# 2. Commit scope update
git add cog.toml
git commit -m "chore(cocogitto): add sorter scope"

# 3. Commit actual code
git add src/sorter.rs
git commit -m "feat(sorter): implement table sorting..."
```

**Current scopes** (in `cog.toml`):

- ai, cocogitto, copier, cspell, github, jscpd, megalinter, parser, pre-commit, prettier, sorter, smt, treefmt, version, vscode, zed

Before implementing **writer** or **main**, ensure scopes are added.

### Commit Footer: AI Model Signoff

**REQUIRED**: Every commit created by an AI agent MUST include a trailer footer with the AI model name.

**Format** (using git trailer):

```bash
git commit --trailer="AI-Model: {model-name}" -m "feat(scope): description..."
```

**Git trailer output format**:

```
feat(sorter): implement table sorting logic

- Detailed description of changes
- More bullet points as needed

AI-Model: claude-haiku-4.5
```

**Purpose**: Provides transparency about which AI model contributed to each commit (for compliance, attribution, and debugging).

**Example**:

```bash
git commit --trailer="AI-Model: claude-haiku-4.5" -m "feat(sorter): implement table sorting with numeric/lexicographic comparators

- Numeric mode parses as f64 with proper NaN handling
- Lexicographic comparison with case-sensitivity toggle
- Stable sort guarantee using sort_by (never sort_unstable_by)
- 34 unit tests covering all scenarios"
```

Git automatically formats the trailer at the end of the commit message.
