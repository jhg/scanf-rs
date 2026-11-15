//! Where the text to parse comes from: a string, a reader or stdin, and what each gives up once
//! a call has succeeded.

use crate::codegen::internal_span;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote_spanned};
use syn::Expr;

/// Where the text to parse comes from.
pub enum Source<'a> {
    /// `sscanf!`: the value of this expression, evaluated once.
    Expression(&'a Expr),
    /// `fscanf!`: one line from this `BufRead`, borrowed mutably like the writer of `write!`.
    Reader(&'a Expr),
    /// `scanf!`: one line from stdin, after flushing stdout so a prompt shows up first.
    Stdin,
}

impl Source<'_> {
    /// Code that defines `remaining: &str` and then runs `body` with it in scope.
    pub fn bind_remaining(&self, body: TokenStream2) -> TokenStream2 {
        let span = internal_span();
        match self {
            // The match keeps temporaries in the input expression alive for the whole body,
            // so `sscanf!(line().trim(), ..)` borrows from a value that is still around.
            Source::Expression(expr) => quote_spanned! {span=>
                match &(#expr) {
                    input => {
                        let mut remaining: &str = input;
                        #body
                    }
                }
            },
            Source::Reader(reader) => peek_line_into_remaining(&reader.to_token_stream(), body),
            Source::Stdin => {
                let flush = quote_spanned! {span=>
                    let _ = ::std::io::Write::flush(&mut ::std::io::stdout());
                };
                let stdin = quote_spanned! {span=> ::std::io::stdin().lock() };
                let read = peek_line_into_remaining(&stdin, body);
                quote_spanned! {span=> #flush #read }
            }
        }
    }

    /// What the source gives up once the call has succeeded: nothing for a string, and for a
    /// reader the bytes that matched, or the whole line unless the final placeholder
    /// (`partial`) may have left some of it for the next call.
    pub fn consume(&self, partial: bool) -> TokenStream2 {
        let span = internal_span();
        if matches!(self, Source::Expression(_)) {
            return TokenStream2::new();
        }
        let used = if partial {
            quote_spanned! {span=>
                text_len - remaining.len() + if remaining.is_empty() { tail } else { 0 }
            }
        } else {
            quote_spanned! {span=> text_len + tail }
        };
        quote_spanned! {span=>
            if peeked {
                let used = #used;
                reader.consume(used);
            }
        }
    }
}

/// Looks at the next line of `reader` without consuming it and defines `remaining: &str`
/// without its line ending, then runs `body`, which ends by consuming what matched (see
/// `Source::consume`). A failure returns before that, so it consumes nothing.
///
/// `BufRead` only shows what is buffered. A line whose end is not in the buffer cannot be
/// looked at as a whole, so it is read (and consumed) up to its end instead.
fn peek_line_into_remaining(reader: &TokenStream2, body: TokenStream2) -> TokenStream2 {
    let span = internal_span();
    quote_spanned! {span=>
        // A method call, so the reader is borrowed automatically (as in `write!`): a variable,
        // a `&mut` reference or a temporary all work without being declared `mut`.
        trait Reborrow {
            fn scanf_reborrow(&mut self) -> &mut Self;
            /// What is buffered, retrying a read a signal interrupted as `read_line` does.
            fn scanf_peek(&mut self) -> ::std::io::Result<&[u8]>;
        }
        impl<T: ::std::io::BufRead + ?::core::marker::Sized> Reborrow for T {
            fn scanf_reborrow(&mut self) -> &mut Self {
                self
            }
            fn scanf_peek(&mut self) -> ::std::io::Result<&[u8]> {
                // Checked first because returning the borrow from inside the loop is rejected.
                while let ::core::result::Result::Err(error) = self.fill_buf() {
                    if error.kind() != ::std::io::ErrorKind::Interrupted {
                        return ::core::result::Result::Err(error);
                    }
                }
                self.fill_buf()
            }
        }
        match (#reader).scanf_reborrow() {
            reader => {
                use ::std::io::BufRead as _;
                let newline = reader.scanf_peek()?.iter().position(|&byte| byte == b'\n');
                let peeked = newline.is_some();
                let mut owned = ::std::vec::Vec::new();
                let (bytes, line_ending): (&[u8], usize) = match newline {
                    ::core::option::Option::Some(end) => (&reader.scanf_peek()?[..end], 1),
                    ::core::option::Option::None => {
                        if reader.read_until(b'\n', &mut owned)? == 0 {
                            return ::core::result::Result::Err(::std::io::Error::new(
                                ::std::io::ErrorKind::UnexpectedEof,
                                "no input available",
                            ));
                        }
                        (owned.strip_suffix(b"\n").unwrap_or(&owned), 0)
                    }
                };
                let line = ::std::str::from_utf8(bytes).map_err(|error| {
                    ::std::io::Error::new(::std::io::ErrorKind::InvalidData, error)
                })?;
                // One line ending is dropped: `\n`, `\r\n` or a lone `\r`. Any other `\r` stays.
                let text = line.strip_suffix('\r').unwrap_or(line);
                let text_len = text.len();
                let tail = line.len() - text_len + line_ending;
                let mut remaining: &str = text;
                #body
            }
        }
    }
}
