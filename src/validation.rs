//! Identifier validation for scanf placeholders.
//!
//! This module handles validation of placeholder identifiers to ensure they
//! are valid Rust identifiers and not reserved keywords.

/// All Rust keywords that cannot be used as placeholder identifiers.
///
/// This list includes current keywords and reserved keywords for future use.
const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

/// Checks if a string is a valid Rust identifier.
///
/// A valid identifier must:
/// - Not be empty
/// - Not be a Rust keyword
/// - Start with an alphabetic character (including Unicode) or underscore
/// - Contain only alphanumeric characters (including Unicode) or underscores
///
/// # Note
///
/// This doesn't check for raw identifiers (r#name) as they're not needed
/// in placeholder context.
///
/// # Performance
///
/// This function is called at compile-time during macro expansion, so it's optimized
/// for correctness over runtime performance. The keyword check uses a simple slice
/// search which is acceptable for compile-time use.
#[inline]
pub fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Check if the identifier is a Rust keyword
    if RUST_KEYWORDS.contains(&s) {
        return false;
    }

    // Invariant: We already checked s.is_empty() above, so next() will return Some
    let mut chars = s.chars();
    let first = chars.next().unwrap();

    // First character: alphabetic (Unicode) or underscore, but not a digit
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    // Subsequent characters: alphanumeric (Unicode) or underscore
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_identifiers() {
        assert!(is_valid_identifier("variable"));
        assert!(is_valid_identifier("var_123"));
        assert!(is_valid_identifier("_private"));
        assert!(is_valid_identifier("número"));
        assert!(is_valid_identifier("VarName"));
        assert!(is_valid_identifier("x"));
    }

    #[test]
    fn test_invalid_identifiers() {
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123var"));
        assert!(!is_valid_identifier("var-name"));
        assert!(!is_valid_identifier("var.name"));
        assert!(!is_valid_identifier("var name"));
    }

    #[test]
    fn test_keywords_rejected() {
        assert!(!is_valid_identifier("let"));
        assert!(!is_valid_identifier("fn"));
        assert!(!is_valid_identifier("struct"));
        assert!(!is_valid_identifier("self"));
        assert!(!is_valid_identifier("Self"));
        assert!(!is_valid_identifier("async"));
        assert!(!is_valid_identifier("await"));
    }
}
