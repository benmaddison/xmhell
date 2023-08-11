//! A less hellscape-ish XML reader.
//!
//! This library extends [quick-xml] by adding methods to read deeply nested XML
//! without endless nested loops of `match` statements.
//!
//! See [`Expect`] for details.
//!
//! # Example
//!
//! ``` rust
//! use xmhell::{quick_xml::Reader, Error, Expect};
//!
//! const IN: &str = r#"
//!     <root>
//!         <foo>
//!             <bar/>
//!             <bar/>
//!         </foo>
//!     </root>
//! "#;
//!
//! fn main() -> Result<(), Error> {
//!     let mut bars = 0;
//!
//!     let mut reader = Reader::from_str(IN);
//!     reader.trim_text(true);
//!
//!     reader.expect_element("root")?.read_inner(|reader| {
//!         reader.expect_element("foo")?.read_inner(|reader| {
//!             while let Ok(()) = reader.expect_empty("bar") {
//!                 bars += 1;
//!             }
//!             Ok(())
//!         })?;
//!         Ok(())
//!     })?;
//!     reader.expect_eof()?;
//!
//!     assert_eq!(bars, 2);
//!
//!     Ok(())
//! }
//! ```
//!
//! [quick-xml]: https://docs.rs/quick-xml
#![doc(html_root_url = "https://docs.rs/xmhell/0.1.0")]
// clippy lints
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![allow(clippy::redundant_pub_crate)]
// rustc lints
#![allow(box_pointers)]
#![warn(absolute_paths_not_starting_with_crate)]
#![warn(deprecated_in_future)]
#![warn(elided_lifetimes_in_paths)]
#![warn(explicit_outlives_requirements)]
#![warn(keyword_idents)]
#![warn(macro_use_extern_crate)]
#![warn(meta_variable_misuse)]
#![warn(missing_abi)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(noop_method_call)]
#![warn(pointer_structural_match)]
#![warn(rust_2021_incompatible_closure_captures)]
#![warn(rust_2021_incompatible_or_patterns)]
#![warn(rust_2021_prefixes_incompatible_syntax)]
#![warn(rust_2021_prelude_collisions)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(unstable_features)]
#![warn(unused_crate_dependencies)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_lifetimes)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]
// docs.rs build config
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub use quick_xml;

mod expect;
pub use self::expect::{ElementReader, Expect};

mod error;
pub use self::error::Error;

// silence unused dev-dependency warnings
#[cfg(test)]
mod deps {
    use version_sync as _;
}
