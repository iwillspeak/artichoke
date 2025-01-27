#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![cfg_attr(test, allow(clippy::non_ascii_literal))]
#![allow(unknown_lints)]
#![warn(broken_intra_doc_links)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![warn(rust_2018_idioms)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]
#![forbid(unsafe_code)]
// Enable feature callouts in generated documentation:
// https://doc.rust-lang.org/beta/unstable-book/language-features/doc-cfg.html
//
// This approach is borrowed from tokio.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_alias))]

//! Identifier for interned bytestrings and routines for manipulating the
//! underlying bytestrings.
//!
//! `Symbol` is a `Copy` type based on `u32`. `Symbol` is cheap to copy, store,
//! and compare. It is suitable for representing indexes into a string interner.
//!
//! # Artichoke integration
//!
//! This crate has an `artichoke` Cargo feature. When this feature is active,
//! this crate implements [the `Symbol` API from Ruby Core]. These APIs require
//! resolving the underlying bytes associated with the `Symbol` via a type that
//! implements `Intern` from `artichoke-core`.
//!
//! APIs that require this feature to be active are highlighted in the
//! documentation.
//!
//! This crate provides an `AllSymbols` iterator for walking all symbols stored
//! in an [`Intern`]er and an extension trait for constructing it which is
//! suitable for implementing [`Symbol::all_symbols`] from Ruby Core.
//!
//! This crate provides an `Inspect` iterator for converting `Symbol` byte
//! content to a debug representation suitable for implementing
//! [`Symbol#inspect`] from Ruby Core.
//!
//! # `no_std`
//!
//! This crate is `no_std` compatible when built without the `std` feature. This
//! crate does not depend on [`alloc`].
//!
//! # Crate features
//!
//! All features are enabled by default.
//!
//! - **artichoke** - Enables additional methods, functions, and types for
//!   implementing APIs from Ruby Core. Dropping this feature removes the
//!   `artichoke-core` and `focaccia` dependencies. Activating this feature also
//!   activates the **inspect** feature.
//! - **inspect** - Enables an iterator for generating debug output of a symbol
//!   bytestring. Activating this feature also activates the **ident-parser**
//!   feature.
//! - **ident-parser** - Enables a parser to determing the Ruby identifier type,
//!   if any, for a bytestring. Dropping this feature removes the `bstr` and
//!   `scolapasta-string-escape` dependencies.
//! - **std** - Enables a dependency on the Rust Standard Library. Activating
//!   this feature enables [`std::error::Error`] impls on error types in this
//!   crate.
//!
//! [the `Symbol` API from Ruby Core]: https://ruby-doc.org/core-2.6.3/Symbol.html
//! [`Symbol::all_symbols`]: https://ruby-doc.org/core-2.6.3/Symbol.html#method-c-all_symbols
//! [`Symbol#inspect`]: https://ruby-doc.org/core-2.6.3/Symbol.html#method-i-inspect
//! [`alloc`]: https://doc.rust-lang.org/alloc/
//! [`std::error::Error`]: https://doc.rust-lang.org/std/error/trait.Error.html

// `spinoso-symbol` is a `no_std` crate unless the `std` feature is enabled.
#![cfg_attr(not(feature = "std"), no_std)]

// Having access to `String` in tests is convenient to collect `Inspect`
// iterators for whole content comparisons.
#[cfg(test)]
extern crate alloc;

// Ensure code blocks in README.md compile
#[cfg(doctest)]
macro_rules! readme {
    ($x:expr) => {
        #[doc = $x]
        mod readme {}
    };
    () => {
        readme!(include_str!("../README.md"));
    };
}
#[cfg(doctest)]
readme!();

#[cfg(feature = "artichoke")]
use artichoke_core::intern::Intern;
use core::borrow::Borrow;
use core::fmt;
use core::mem::size_of;
use core::num::TryFromIntError;

#[doc(inline)]
#[cfg(feature = "artichoke")]
#[cfg_attr(docsrs, doc(cfg(feature = "artichoke")))]
pub use focaccia::{CaseFold, NoSuchCaseFoldingScheme};

// Spinoso symbol assumes symbols are `u32` and requires `usize` to be at least
// as big as `u32`.
//
// This const-evaluated expression will fail to compile if this invariant does
// not hold.
const _: () = [()][!(size_of::<usize>() >= size_of::<u32>()) as usize];

#[cfg(feature = "artichoke")]
mod all_symbols;
#[cfg(feature = "artichoke")]
mod casecmp;
mod convert;
mod eq;
#[cfg(feature = "ident-parser")]
mod ident;
#[cfg(feature = "inspect")]
mod inspect;

#[cfg(test)]
mod fixtures;

#[cfg(feature = "artichoke")]
pub use all_symbols::{AllSymbols, InternerAllSymbols};
#[cfg(feature = "artichoke")]
pub use casecmp::{ascii_casecmp, unicode_case_eq};
#[cfg(feature = "ident-parser")]
pub use ident::{IdentifierType, ParseIdentifierError};
#[cfg(feature = "inspect")]
pub use inspect::Inspect;

/// Error returned when a symbol identifier overflows.
///
/// Spinoso symbol uses `u32` identifiers for symbols to save space. If more
/// than `u32::MAX` symbols are stored in the underlying table, no more
/// identifiers can be generated.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolOverflowError {
    _private: (),
}

impl SymbolOverflowError {
    /// The maximum identifier of a `Symbol`.
    pub const MAX_IDENTIFIER: usize = u32::MAX as usize;

    /// Construct a new, default `SymbolOverflowError`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

impl From<TryFromIntError> for SymbolOverflowError {
    #[inline]
    fn from(err: TryFromIntError) -> Self {
        let _ = err;
        Self::new()
    }
}

impl fmt::Display for SymbolOverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Symbol overflow")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SymbolOverflowError {}

/// Identifier bound to an interned bytestring.
///
/// A `Symbol` allows retrieving a reference to the original interned
/// bytestring.  Equivalent `Symbol`s will resolve to an identical bytestring.
///
/// `Symbol`s are based on a `u32` index. They are cheap to compare and cheap to
/// copy.
///
/// `Symbol`s are not constrained to the interner which created them.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol(u32);

impl Borrow<u32> for Symbol {
    fn borrow(&self) -> &u32 {
        &self.0
    }
}

impl Symbol {
    /// Construct a new `Symbol` from the given `u32`.
    ///
    /// `Symbol`s constructed manually may fail to resolve to an underlying
    /// bytesstring.
    ///
    /// `Symbol`s are not constrained to the interner which created them.
    /// No runtime checks ensure that the underlying interner is called with a
    /// `Symbol` that the interner itself issued.
    ///
    /// # Examples
    ///
    /// ```
    /// # use spinoso_symbol::Symbol;
    /// let sym = Symbol::new(263);
    /// assert_eq!(sym.id(), 263);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Return the `u32` identifier from this `Symbol`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use spinoso_symbol::Symbol;
    /// let sym = Symbol::new(263);
    /// assert_eq!(sym.id(), 263);
    /// assert_eq!(u32::from(sym), 263);
    /// ```
    #[inline]
    #[must_use]
    pub const fn id(self) -> u32 {
        self.0
    }

    /// Returns whether the symbol is the empty byteslice `b""` in the
    /// underlying interner.
    ///
    /// If there symbol does not exist in the underlying interner or there is an
    /// error looking up the symbol in the underlying interner, `true` is
    /// returned.
    #[inline]
    #[must_use]
    #[cfg(feature = "artichoke")]
    #[cfg_attr(docsrs, doc(cfg(feature = "artichoke")))]
    pub fn is_empty<T, U>(self, interner: &T) -> bool
    where
        T: Intern<Symbol = U>,
        U: Copy + From<Symbol>,
    {
        if let Ok(Some(bytes)) = interner.lookup_symbol(self.into()) {
            bytes.is_empty()
        } else {
            true
        }
    }

    /// Returns the length of the byteslice associated with the symbol in the
    /// underlying interner.
    ///
    /// If there symbol does not exist in the underlying interner or there is an
    /// error looking up the symbol in the underlying interner, `0` is returned.
    #[inline]
    #[must_use]
    #[cfg(feature = "artichoke")]
    #[cfg_attr(docsrs, doc(cfg(feature = "artichoke")))]
    pub fn len<T, U>(self, interner: &T) -> usize
    where
        T: Intern<Symbol = U>,
        U: Copy + From<Symbol>,
    {
        if let Ok(Some(bytes)) = interner.lookup_symbol(self.into()) {
            bytes.len()
        } else {
            0_usize
        }
    }

    /// Returns the interned byteslice associated with the symbol in the
    /// underlying interner.
    ///
    /// If there symbol does not exist in the underlying interner or there is an
    /// error looking up the symbol in the underlying interner, `&[]` is
    /// returned.
    #[inline]
    #[must_use]
    #[cfg(feature = "artichoke")]
    #[cfg_attr(docsrs, doc(cfg(feature = "artichoke")))]
    pub fn bytes<T, U>(self, interner: &T) -> &[u8]
    where
        T: Intern<Symbol = U>,
        U: Copy + From<Symbol>,
    {
        if let Ok(Some(bytes)) = interner.lookup_symbol(self.into()) {
            bytes
        } else {
            &[]
        }
    }

    /// Returns an iterator that yields a debug representation of the interned
    /// byteslice associated with the symbol in the underlying interner.
    ///
    /// This iterator produces [`char`] sequences like `:spinoso` and
    /// `:"invalid-\xFF-utf8"`.
    ///
    /// If there symbol does not exist in the underlying interner or there is an
    /// error looking up the symbol in the underlying interner, a default
    /// iterator is returned.
    #[inline]
    #[cfg(feature = "artichoke")]
    #[cfg_attr(docsrs, doc(cfg(feature = "artichoke")))]
    pub fn inspect<T, U>(self, interner: &T) -> Inspect<'_>
    where
        T: Intern<Symbol = U>,
        U: Copy + From<Symbol>,
    {
        if let Ok(Some(bytes)) = interner.lookup_symbol(self.into()) {
            Inspect::from(bytes)
        } else {
            Inspect::default()
        }
    }

    /// Write an [`Inspect`] iterator into the given destination using the debug
    /// representation of the interned byteslice associated with the symbol in
    /// the underlying interner.
    ///
    /// This formatter writes content like `:spinoso` and `:"invalid-\xFF-utf8"`.
    /// To see example output of the underlying iterator, see the [`Inspect`]
    /// documentation.
    ///
    /// To write binary output, use [`write_inspect_into`], which requires the
    /// `std` feature to be activated.
    ///
    /// If there symbol does not exist in the underlying interner or there is an
    /// error looking up the symbol in the underlying interner, a default
    /// inspect representation is written.
    ///
    /// # Errors
    ///
    /// If the given writer returns an error as it is being written to, that
    /// error is returned.
    ///
    /// [`write_inspect_into`]: Self::write_inspect_into
    #[inline]
    #[cfg(feature = "artichoke")]
    #[cfg_attr(docsrs, doc(cfg(feature = "artichoke")))]
    pub fn format_inspect_into<T, U, W>(self, interner: &T, mut dest: W) -> fmt::Result
    where
        T: Intern<Symbol = U>,
        U: Copy + From<Symbol>,
        W: fmt::Write,
    {
        let inspect = self.inspect(interner);
        for ch in inspect {
            dest.write_char(ch)?;
        }
        Ok(())
    }

    /// Write an [`Inspect`] iterator into the given destination using the debug
    /// representation of the interned byteslice associated with the symbol in
    /// the underlying interner.
    ///
    /// This formatter writes content like `:spinoso` and `:"invalid-\xFF-utf8"`.
    /// To see example output of the underlying iterator, see the [`Inspect`]
    /// documentation.
    ///
    /// To write to a `String` formatter, use [`format_inspect_into`].
    ///
    /// If there symbol does not exist in the underlying interner or there is an
    /// error looking up the symbol in the underlying interner, a default
    /// inspect representation is written.
    ///
    /// # Errors
    ///
    /// If the given writer returns an error as it is being written to, that
    /// error is returned.
    ///
    /// [`format_inspect_into`]: Self::format_inspect_into
    #[inline]
    #[cfg(all(feature = "artichoke", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "artichoke", feature = "std"))))]
    pub fn write_inspect_into<T, U, W>(self, interner: &T, mut dest: W) -> std::io::Result<()>
    where
        T: Intern<Symbol = U>,
        U: Copy + From<Symbol>,
        W: std::io::Write,
    {
        let inspect = self.inspect(interner);
        let mut buf = [0; 4];
        for ch in inspect {
            let utf8 = ch.encode_utf8(&mut buf);
            dest.write_all(utf8.as_bytes())?;
        }
        Ok(())
    }
}
