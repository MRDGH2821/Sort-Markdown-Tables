# Usage

## Basic: Sort a file and print to stdout

```bash
smt documents/file.md
```

## Sort in-place (modifies file)

```bash
smt -i documents/file.md
```

## Check if files are sorted (don't modify)

```bash
smt --check documents/file.md
echo $?  # Exit 0 = sorted, 1 = unsorted, 2 = error
```

## Save output to a different file

```bash
smt documents/file.md -o documents/file-sorted.md
```

## Process multiple files with glob patterns

```bash
smt -i "docs/**/*.md"
```

## Scan a directory recursively

```bash
smt -r -i docs/
```

## Use with stdin/stdout

```bash
cat documents/file.md | smt | tee output.md
```

## Table Format

To opt-in to sorting, place `<!-- smt -->` immediately before a table:

```markdown
Some introduction text.

<!-- smt -->

| Name    | Age | Score |
| ------- | --- | ----- |
| Alice   | 30  | 95    |
| Bob     | 25  | 87    |
| Charlie | 28  | 92    |

Rest of the document...
```

### Sort Configurations

Use attributes in the comment to configure sorting:

```markdown
<!-- smt type=numeric column=1 order=asc case=insensitive -->

| A   | B   |
| --- | --- |
| 1   | x   |
| 3   | z   |
```

| Attribute | Options                    | Default         | Notes                                |
| --------- | -------------------------- | --------------- | ------------------------------------ |
| `type`    | `numeric`, `lexicographic` | `lexicographic` | Numeric handles decimals, NaN safely |
| `column`  | `1`, `2`, `3`...           | `1`             | 1-indexed; first data column only    |
| `order`   | `asc`, `descending`        | `asc`           | Sort order                           |
| `case`    | `sensitive`, `insensitive` | `sensitive`     | Only affects lexicographic mode      |

### Tables without `<!-- smt -->` are untouched

```markdown
| This table | is left | alone |
| ---------- | ------- | ----- |
| 3          | z       |
| 1          | x       |
```

## Exit Codes

| Code | Meaning                                              |
| ---- | ---------------------------------------------------- |
| `0`  | Success (file(s) sorted or already sorted)           |
| `1`  | Check failed (file is unsorted, `--check` mode only) |
| `2`  | Error (invalid arguments, file not found, I/O error) |
