//! Core types for scanf macro implementation.
//!
//! This module defines the fundamental types used during format string
//! tokenization and code generation.

/// Represents a placeholder in a format string.
///
/// Placeholders can be either named (e.g., `{variable}`) or anonymous (e.g., `{}`).
///
/// # Memory Layout
///
/// Uses `Box<str>` instead of `String` for Named variant to minimize memory overhead.
/// `Box<str>` is more memory-efficient for immutable strings (no capacity field):
/// - `String`: 24 bytes (pointer + length + capacity)
/// - `Box<str>`: 16 bytes (pointer + length)
///
/// This 33% memory saving is appropriate since placeholder names don't change after parsing.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Placeholder {
    /// A named placeholder that captures to a specific variable.
    ///
    /// Uses `Box<str>` for memory efficiency (no capacity overhead).
    Named(Box<str>),

    /// An anonymous placeholder that requires an explicit argument.
    Anonymous,
}

/// Token type for compile-time tokenization of format strings.
///
/// Represents either literal text or a placeholder in the format string.
///
/// # Design Note
///
/// `FormatToken` is only used during compile-time macro expansion, not in the generated
/// runtime code. Therefore, we prioritize code clarity over runtime performance.
#[derive(Debug, Clone)]
pub enum FormatToken {
    /// A literal text segment that must match exactly in the input.
    Text(String),

    /// A placeholder that captures a value from the input.
    Placeholder(Placeholder),
}
