# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in scanf-rs, please report it by creating a new issue on our [GitHub repository](https://github.com/jhg/scanf-rs/issues) or by contacting the maintainer directly at jesushdez@protonmail.com.

We take security seriously and will respond to vulnerability reports as quickly as possible.

## Security Advisories

### Fixed Vulnerabilities

#### SCANF-2025-001: MAX_TOKENS Bypass Vulnerability

**Severity**: HIGH  
**Fixed in**: Commit 12c119a99f8a018e7ba4082040d18fd6934e3416  
**Status**: FIXED  
**Date**: 2025-11-03

**Description**:  
The MAX_TOKENS security limit (256 tokens) could be completely bypassed by using only placeholder tokens without text separators in the format string. This vulnerability allowed an attacker to cause compile-time DoS through unbounded token vector growth.

**Attack Vector**:  
A malicious format string consisting of `"{}"` repeated 257 or more times would push 257+ placeholder tokens without any validation, completely bypassing the MAX_TOKENS limit designed to prevent compile-time resource exhaustion.

**Root Cause**:  
The vulnerable code only checked the MAX_TOKENS limit after pushing text tokens, but not when pushing:
- Anonymous placeholder tokens
- Named placeholder tokens  
- Final text tokens

This meant 3 out of 4 token push sites had no security validation.

**Impact**:  
- **Compile-time DoS**: Excessive memory usage during macro expansion
- **Build failure**: Could cause builds to hang or crash
- **CI/CD disruption**: Could be used to disrupt automated build systems

**Fix**:  
Introduced a safe `push_token()` helper function that validates the MAX_TOKENS limit BEFORE every token push operation. The check was applied to all 4 token push sites in the codebase:
1. Text tokens (middle of parsing)
2. Anonymous placeholder tokens
3. Named placeholder tokens
4. Final text token

Additionally fixed an off-by-one error by changing the check from `>` to `>=` to ensure exactly 256 tokens maximum.

**Affected Versions**:  
All versions prior to the fix commit 12c119a99f8a018e7ba4082040d18fd6934e3416.

**Recommendation**:  
Users should ensure they are using a version that includes the security fix. Check your dependency version and update if necessary.

**Discovered and Fixed by**:  
Claude (Anthropic) during a security review.

## Security Best Practices

When using scanf-rs:

1. **Limit format string size**: The library enforces a MAX_FORMAT_STRING_LEN of 10,000 bytes
2. **Token limit**: Maximum 256 tokens per format string
3. **Identifier length**: Maximum 128 characters for placeholder identifiers
4. **Validate input**: Always validate and sanitize user input before processing
5. **Keep updated**: Regularly update to the latest version to receive security fixes

## Supported Versions

We recommend always using the latest version of scanf-rs. Security fixes are applied to the main development branch.

| Version | Supported          |
| ------- | ------------------ |
| 2.0.x   | :white_check_mark: |
| < 2.0   | :x:                |
