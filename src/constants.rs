//! Security limits and capacity constants for the scanf macros.
//!
//! This module centralizes all compile-time and runtime limits to prevent
//! denial-of-service attacks and optimize memory allocation.

// ============================================================================
// Security Limits (Compile-Time DoS Protection)
// ============================================================================

/// Maximum length of a format string in bytes.
///
/// This limit prevents compile-time DoS attacks via extremely long format strings
/// while allowing all legitimate use cases.
pub const MAX_FORMAT_STRING_LEN: usize = 10_000;

/// Maximum number of tokens in a format string.
///
/// This limit prevents excessive code generation and compile-time resource exhaustion.
pub const MAX_TOKENS: usize = 256;

/// Maximum length of an identifier in a placeholder.
///
/// This limit prevents DoS attacks via extremely long identifier names.
pub const MAX_IDENTIFIER_LEN: usize = 128;

// ============================================================================
// Memory Pre-Allocation Hints
// ============================================================================

/// Initial capacity hint for the token vector.
///
/// Most format strings have 2-4 tokens, so this avoids initial reallocations.
pub const TOKENS_INITIAL_CAPACITY: usize = 4;

/// Initial capacity hint for text segments between placeholders.
///
/// Typical separators are short (": ", " - ", etc.), so 16 bytes is sufficient.
pub const TEXT_SEGMENT_CAPACITY: usize = 16;

/// Initial capacity hint for placeholder identifier names.
///
/// Most identifiers are short (3-10 characters), so 8 bytes avoids reallocations.
pub const IDENTIFIER_CAPACITY: usize = 8;
