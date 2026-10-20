//! McMarcel Studio core: country packs, cohort engine, CEA and validation.
//!
//! This crate is a clean-room Rust implementation of the McMarcel model
//! structure. It keeps the existing R implementation as a validation oracle.
//! See the repository README and `docs/` for evidence and licensing rules.

pub mod analysis;
pub mod engine;
pub mod error;
pub mod model;
pub mod pack;
pub mod questionnaire;
pub mod rng;
pub mod types;
pub mod validate;

pub use error::{Error, Result};
pub use types::*;

/// Human-readable engine version used in API responses and run manifests.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
