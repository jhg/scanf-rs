#![forbid(unsafe_code)]
#![allow(clippy::needless_return)]
#![doc = include_str!("../README.md")]

// Re-export the procedural macros
pub use scanf_proc_macro::{scanf, sscanf};
