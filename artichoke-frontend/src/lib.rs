#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(missing_docs, warnings, intra_doc_link_resolution_failure)]
#![doc(deny(warnings))]

//!  Crate artichoke-frontend provides binaries for interacting with the
//!  artichoke interpreter in the [`artichoke-backend`](artichoke_backend)
//!  crate.

pub mod parser;
pub mod repl;
pub mod ruby;
