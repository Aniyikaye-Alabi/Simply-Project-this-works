//! Show list, resolver and project file (PRD sections 4 and 8).
//!
//! This crate is pure logic. It must not depend on the OS, the UI or the network
//! (AGENTS.md section 4.1). The resolver lives here and is the source of truth
//! for what the audience sees.

mod cue_stack;
mod error;
mod item;
mod resolver;

pub use cue_stack::CueStack;
pub use error::UserError;
pub use item::{Item, ItemId, ItemState};
pub use resolver::{next_in_line, resolve, Controls, Program};
