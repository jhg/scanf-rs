//! Core types for scanf macros.

/// Placeholder in format string: `{name}` or `{}`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Placeholder {
    Named(Box<str>),
    Anonymous,
}

/// Format string token: literal text or placeholder.
#[derive(Debug, Clone)]
pub enum FormatToken {
    Text(Box<str>),
    Placeholder(Placeholder),
}
