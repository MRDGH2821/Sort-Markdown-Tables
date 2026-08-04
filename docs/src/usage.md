# Usage & Configuration

This guide covers how to mark your markdown tables for sorting, customize sort options, and run `smt` via CLI.

---

## 1. Marking Tables for Sorting

To opt-in a table for sorting, place an `<!-- smt -->` comment **immediately** above the table:

```markdown
<!-- smt -->

| Name    | Score |
| ------- | ----- |
| Charlie | 88    |
| Alice   | 95    |
| Bob     | 78    |
```

!!! warning "Placement Rule"
The `<!-- smt -->` comment must immediately precede the table header row (no blank lines or text between the comment and table).

---

## 2. Sort Attributes Reference

You can customize sorting behavior by adding attributes inside the comment:

```markdown
<!-- smt type=numeric column=2 order=desc case=insensitive -->
```

| Attribute | Accepted Values                 | Default         | Description                                                       |
| :-------- | :------------------------------ | :-------------- | :---------------------------------------------------------------- |
| `type`    | `numeric`, `lexicographic`      | `lexicographic` | How column values are compared (`numeric` parses numbers safely). |
| `column`  | `1`, `2`, `3`...                | `1`             | 1-indexed column number to sort by.                               |
| `order`   | `asc`, `descending` (or `desc`) | `asc`           | Sort direction: ascending or descending.                          |
| `case`    | `sensitive`, `insensitive`      | `sensitive`     | Case sensitivity (applies to `lexicographic` mode).               |

---

## 3. Sort Configuration Examples

### Numeric Sorting (`type=numeric`)

Useful for scores, ages, prices, or version numbers. Handles floats, negative numbers, and `NaN` values safely.

=== "Before"

    ```markdown
    <!-- smt type=numeric column=2 order=asc -->
    | Item     | Price ($) |
    | -------- | --------- |
    | Laptop   | 1200.50   |
    | Mouse    | 25.00     |
    | Keyboard | 89.99     |
    ```

=== "After"

    ```markdown
    <!-- smt type=numeric column=2 order=asc -->
    | Item     | Price ($) |
    | -------- | --------- |
    | Mouse    | 25.00     |
    | Keyboard | 89.99     |
    | Laptop   | 1200.50   |
    ```

---

### Sorting by Specific Column (`column=N`)

Columns are 1-indexed (`column=1` is the first data column).

=== "Before"

    ```markdown
    <!-- smt column=2 order=asc -->
    | Project | Language  | Stars |
    | ------- | --------- | ----- |
    | smt     | Rust      | 150   |
    | app     | Go        | 300   |
    | web     | TypeScript| 200   |
    ```

=== "After"

    ```markdown
    <!-- smt column=2 order=asc -->
    | Project | Language  | Stars |
    | ------- | --------- | ----- |
    | app     | Go        | 300   |
    | smt     | Rust      | 150   |
    | web     | TypeScript| 200   |
    ```

---

### Case-Insensitive Alphabetical Sorting (`case=insensitive`)

=== "Before"

    ```markdown
    <!-- smt case=insensitive order=asc -->
    | Fruit   |
    | ------- |
    | apple   |
    | Banana  |
    | cherry  |
    | Apricot |
    ```

=== "After"

    ```markdown
    <!-- smt case=insensitive order=asc -->
    | Fruit   |
    | ------- |
    | apple   |
    | Apricot |
    | Banana  |
    | cherry  |
    ```

---

## 4. CLI Command Examples

### Sort in-place (`-i`)

Overwrites files directly with sorted tables:

```bash
smt -i README.md
```

### Check mode (`--check`)

Validates whether files are sorted without modifying them. Ideal for CI scripts:

```bash
smt --check docs/**/*.md
```

### Output to another file (`-o`)

Prints sorted result to a separate file:

```bash
smt input.md -o output.md
```

### Directory Recursion (`-r`)

Recursively scans directories for all `.md` files:

```bash
smt -r -i docs/
```

### Pipe via Stdin / Stdout

```bash
cat input.md | smt | tee output.md
```

---

## 5. Exit Codes

| Code | Status           | Meaning                                              |
| :--: | :--------------- | :--------------------------------------------------- |
| `0`  | **Success**      | All files are sorted (or were already sorted).       |
| `1`  | **Check Failed** | At least one file is unsorted (`--check` mode only). |
| `2`  | **User Error**   | Invalid arguments, non-existent files, or I/O error. |
