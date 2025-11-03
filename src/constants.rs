//! Security limits and memory pre-allocation hints.

// Security limits (DoS protection)
/// Max format string length (bytes).
pub const MAX_FORMAT_STRING_LEN: usize = 10_000;
/// Max tokens in format string.
pub const MAX_TOKENS: usize = 256;
/// Max identifier length (chars).
pub const MAX_IDENTIFIER_LEN: usize = 128;

// Memory pre-allocation hints
/// Initial token vector capacity.
pub const TOKENS_INITIAL_CAPACITY: usize = 4;
/// Initial text segment capacity.
pub const TEXT_SEGMENT_CAPACITY: usize = 16;
/// Initial identifier capacity.
pub const IDENTIFIER_CAPACITY: usize = 8;
