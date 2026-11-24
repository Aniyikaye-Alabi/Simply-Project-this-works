//! Show list, resolver and project file (PRD sections 4 and 8).
//!
//! This crate is pure logic. It must not depend on the OS, the UI or the network
//! (AGENTS.md section 4.1). The resolver (M1) lives here and is the source of truth
//! for what the audience sees.

mod error;

pub use error::UserError;
