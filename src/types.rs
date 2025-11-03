//! Core types for scanf macros.

/// Placeholder in format string: `{name}` or `{}`.
///
/// Named uses `Box<str>` (16 bytes) vs `String` (24 bytes) for 33% memory saving.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Placeholder {
    Named(Box<str>),
    Anonymous,
}

/// Format string token: literal text or placeholder.
#[derive(Debug, Clone)]
pub enum FormatToken {
    Text(String),
    Placeholder(Placeholder),
}
