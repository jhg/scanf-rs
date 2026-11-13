# Changelog

## Unreleased (3.0.0)

### Added
- `fscanf!(reader, "format")` reads from any `BufRead`, borrowing it like the writer of `write!`.
- `sscanf!`, `fscanf!` and `scanf!` return the parsed values when the placeholders are empty:
  `let (name, age): (String, u32) = sscanf!(input, "{}: {}").unwrap();`.
- Named placeholders in every macro: `sscanf!(input, "{name}: {age}")` assigns your variables.
- Feature `ignore-trailing-input`, off by default, to accept text left after the last literal.
- `fscanf!` and `scanf!` consume only what matched: text after a final value that parses
  (`12 abc` read as a number) stays for the next call.

### Changed (breaking)
- Input left after the last literal of the format is an error (`"{} units"` no longer accepts
  `"5 units extra"`). Use the `ignore-trailing-input` feature for the old behaviour.
- A call writes to your variables only when every placeholder parses; before, the ones parsed
  before a failure were already written.
- A failed `fscanf!` or `scanf!` consumes nothing from the reader. Skip the line yourself
  before retrying, or the same input fails again.
- `fscanf!` needs a `BufRead`: pass `stdin().lock()`, not `stdin()`.
- `scanf!` holds the stdin lock during the call, so it must not be used while the same thread
  holds `stdin().lock()` (use `fscanf!` on that lock).

### Migrating from 2.x
- Calls with `&mut` arguments keep working. If you relied on trailing input being ignored,
  enable `ignore-trailing-input`; if you read lines in a loop and retried after errors, skip the
  bad line explicitly.
