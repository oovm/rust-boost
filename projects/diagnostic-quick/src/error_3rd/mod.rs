use std::error::Error;

#[cfg(feature = "async-walkdir")]
pub use fs_tools::AsyncWalker as AsyncWalkDir;
#[cfg(feature = "email_address")]
pub use email_address::EmailAddress;
#[cfg(feature = "globset")]
pub use globset::{Glob, GlobSet, GlobSetBuilder};
#[cfg(feature = "mime")]
pub use mime::Mime;
#[cfg(feature = "num")]
pub use num::{traits as num_traits, BigInt, BigRational, BigUint};
#[cfg(feature = "semver")]
pub use semver::Version;
#[cfg(feature = "serde_json")]
pub use serde_json::Value as Json;
#[cfg(feature = "toml")]
pub use toml::Value as Toml;
#[cfg(feature = "url")]
pub use url::Url;
#[cfg(feature = "walkdir")]
pub use fs_tools::Walker as WalkDir;

use crate::{IOError, QError, QErrorKind, RuntimeError, SyntaxError};

pub use self::for_ast::NodeLocation;
#[cfg(feature = "rust_decimal")]
pub use self::for_rust_decimal::*;

// #[cfg(feature = "lsp-types")]
// mod for_lsp;
#[cfg(feature = "num")]
mod for_num;
#[cfg(feature = "peginator")]
mod for_peginator;
#[cfg(feature = "ropey")]
mod for_ropey;
#[cfg(feature = "ucd-trie")]
mod for_ucd_trie;

#[cfg(feature = "url")]
mod for_url;

#[cfg(feature = "sled")]
mod for_sled;

#[cfg(feature = "rust_decimal")]
mod for_rust_decimal;
#[cfg(feature = "serde-binary")]
mod for_serde_binary;

#[cfg(feature = "tl")]
mod for_tl;

#[cfg(feature = "globset")]
mod for_globset;

#[cfg(feature = "serde")]
mod for_serde;

#[cfg(any(feature = "walkdir", feature = "async-walkdir"))]
mod for_walkdir;

#[cfg(feature = "toml")]
mod for_toml;

#[cfg(feature = "chrono")]
mod for_chrono;
#[cfg(feature = "email_address")]
mod for_email_address;
#[cfg(feature = "mime")]
mod for_mime;
#[cfg(feature = "semver")]
mod for_semver;
#[cfg(feature = "serde_json")]
mod for_serde_json;

#[cfg(feature = "font-kit")]
mod for_font_kit;

#[cfg(feature = "image")]
mod for_image;

#[cfg(feature = "imageproc")]
mod for_imageproc;

mod for_ast;

#[allow(unused)]
impl QError {
    #[inline]
    pub(crate) fn fast_runtime_error(error: impl Error + 'static) -> QError {
        QError {
            error: Box::new(QErrorKind::Runtime(RuntimeError::from(&error))),
            level: Default::default(),
            source: Some(Box::new(error)),
        }
    }
    #[inline]
    pub(crate) fn fast_syntax_error(error: impl Error + 'static) -> QError {
        QError {
            error: Box::new(QErrorKind::Syntax(SyntaxError::from(&error))),
            level: Default::default(),
            source: Some(Box::new(error)),
        }
    }
    #[inline]
    pub(crate) fn fast_io_error(error: impl Error + 'static) -> QError {
        QError {
            error: Box::new(QErrorKind::IO(IOError { message: error.to_string(), file: Default::default() })),
            level: Default::default(),
            source: Some(Box::new(error)),
        }
    }
}
