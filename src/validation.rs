//! Identifier validation for placeholders.

/// Rust keywords (current + reserved for future).
const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

/// Check if string is valid Rust identifier.
#[inline]
pub fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() || RUST_KEYWORDS.contains(&s) {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap(); // OK to unwrap: checked is_empty above

    if !first.is_alphabetic() && first != '_' {
        return false;
    }

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
