#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]
#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub mod format;

// Re-export the procedural macros
pub use scanf_proc_macro::{scanf, sscanf};

#[cfg(test)]
mod tests {
    use super::*;
}
