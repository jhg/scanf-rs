# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security
- **HIGH SEVERITY FIX**: Fixed MAX_TOKENS bypass vulnerability (SCANF-2025-001) in commit 12c119a99f8a018e7ba4082040d18fd6934e3416
  - Vulnerability allowed format strings with 257+ consecutive placeholders to bypass the 256 token limit
  - Added safe `push_token()` helper that validates token limit before every push operation
  - Applied validation to all 4 token push sites (previously only 1 was checked)
  - Fixed off-by-one error in boundary check (changed `>` to `>=`)
  - See SECURITY.md for full details

### Added
- Added `test_max_tokens_consecutive_placeholders` regression test to prevent reintroduction of the vulnerability
- Added SECURITY.md documenting security policies and advisories
- Added CHANGELOG.md to track project changes

### Changed
- Improved error message clarity in token limit validation (now shows "would exceed X")
- Refactored to use more specific imports instead of wildcard imports
- Applied rustfmt formatting for code consistency

### Fixed
- Removed unnecessary `#![allow(clippy::needless_return)]` in benchmarks
- Removed needless explicit return in benchmark helper function

## [2.0.0] - Prior Release

Initial major version release with enhanced features.

### Added
- Automatic variable capture in format strings (enhanced syntax)
- Support for named placeholders: `{variable_name}`
- Support for anonymous placeholders: `{}`
- Mixed syntax support (both named and anonymous in same format)
- Escaped brace support: `{{` and `}}`
- Comprehensive security limits:
  - MAX_FORMAT_STRING_LEN: 10,000 bytes
  - MAX_TOKENS: 256 tokens
  - MAX_IDENTIFIER_LEN: 128 characters

### Changed
- Major API redesign for better ergonomics
- Improved error messages with detailed context

[Unreleased]: https://github.com/jhg/scanf-rs/compare/v2.0.0...HEAD
[2.0.0]: https://github.com/jhg/scanf-rs/releases/tag/v2.0.0
