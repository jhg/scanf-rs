//! Identifier validation for placeholders.

/// Rust keywords (current + reserved for future).
const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

/// Whether `s` can be used as a variable name in a placeholder.
///
/// Delegates the character rules to the Rust lexer through `syn`, so it accepts exactly what
/// the compiler does (XID characters, no lone `_`), and then rules out what a placeholder
/// must not contain: keywords, raw identifiers and surrounding whitespace.
pub fn is_valid_identifier(s: &str) -> bool {
    if RUST_KEYWORDS.contains(&s) || s.contains('#') {
        return false;
    }
    syn::parse_str::<syn::Ident>(s).is_ok_and(|ident| ident == s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ordinary_identifiers() {
        for name in [
            "variable", "var_123", "_private", "VarName", "x", "a1", "_a", "__",
        ] {
            assert!(is_valid_identifier(name), "{name:?} should be valid");
        }
    }

    #[test]
    fn accepts_unicode_identifiers() {
        for name in ["número", "é", "ñame", "x\u{0660}", "\u{2160}x"] {
            assert!(is_valid_identifier(name), "{name:?} should be valid");
        }
    }

    #[test]
    fn rejects_malformed_identifiers() {
        for name in [
            "", "123var", "1a", "var-name", "a-b", "var.name", "var name", " x", "x ", "r#x", "'a",
        ] {
            assert!(!is_valid_identifier(name), "{name:?} should be invalid");
        }
    }

    #[test]
    fn rejects_a_lone_underscore() {
        assert!(!is_valid_identifier("_"));
    }

    #[test]
    fn rejects_characters_that_are_alphanumeric_but_not_identifier_characters() {
        for name in ["x\u{b2}", "\u{b2}x", "x\u{0660}\u{2160}\u{00B2}", "x\u{bd}"] {
            assert!(!is_valid_identifier(name), "{name:?} should be invalid");
        }
    }

    #[test]
    fn rejects_every_listed_keyword() {
        for keyword in RUST_KEYWORDS {
            assert!(
                !is_valid_identifier(keyword),
                "{keyword:?} should be rejected"
            );
        }
    }

    #[test]
    fn accepts_gen_since_it_is_only_reserved_from_edition_2024() {
        // The macro cannot see the edition of the calling crate. In 2024 rustc itself rejects
        // the variable, and in earlier editions it is a perfectly good name.
        assert!(is_valid_identifier("gen"));
    }

    #[test]
    fn rejects_common_keywords() {
        for keyword in ["let", "fn", "struct", "self", "Self", "async", "await"] {
            assert!(
                !is_valid_identifier(keyword),
                "{keyword:?} should be rejected"
            );
        }
    }
}
