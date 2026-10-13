# Sort Markdown Tables

The language used to describe marked Markdown tables and their row order.
These terms apply to both command-line use and project documentation.

## Language

**SMT comment**:
An HTML comment that starts with `smt` and marks a table for sorting.
It can contain sort options, as in `<!-- smt column=2 -->`.
_Avoid_: Sort directive, table annotation.

**Marked table**:
A Markdown table associated with an SMT comment.
_Avoid_: Sorted table when referring to whether a table is marked.

**Unmarked table**:
A Markdown table without an associated SMT comment.

**Header row**:
The first row of a table, which names its columns.

**Separator row**:
The row between the header and the data rows, with dashes and optional column alignment markers.

**Data row**:
A table row below the separator row whose position can change during sorting.

**Sort column**:
The column whose cell values determine the order of data rows.
Column numbers start at one.

**Sort key**:
The comparison value derived from a data row's cell in the sort column.

**Sort options**:
The choices in an SMT comment that specify the sort column, direction, case sensitivity, and sort type.

**Numeric sort**:
Row ordering based on numeric values in the sort column, with a rule for cells that are not numbers.

**Lexicographic sort**:
Row ordering based on cell text, with the selected case sensitivity.
_Avoid_: Alphabetical sort when describing the exact ordering rule.

**Stable row order**:
The original relative order of data rows that have equal sort keys.

**Check mode**:
A read-only check of whether every marked table follows its sort options.
An unsorted table is a check result, separate from an input or processing error.

**In-place output**:
Sorted Markdown that replaces its input file.
_Avoid_: In-place sort when referring to writing a file.

**In-place batch**:
The complete set of input files selected for in-place output in one invocation.

**Line-ending override**:
The effective EditorConfig `end_of_line` value for the relevant output path.
An absent or `unset` value leaves the original line terminators in use.
