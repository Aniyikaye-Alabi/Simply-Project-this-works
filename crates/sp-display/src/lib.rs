//! Monitors, extend mode, hot-plug and the output window (PRD OUT-1 to OUT-6).
//!
//! OS access sits behind the [`DisplayProvider`] trait, with a fake for tests
//! ([`fake::FakeDisplayProvider`]). Everything else in this crate is pure logic that
//! works on plain [`Display`] values, so it is tested without hardware.
//!
//! Displays are identified by their device path, never by index (AGENTS.md section 10).

mod display;
mod events;
pub mod fake;
mod provider;
mod target;

pub use display::{identify_numbers, Display, DisplayId};
pub use events::{diff, DisplayEvent};
pub use provider::DisplayProvider;
pub use target::{check_live, preselect, LiveCheck, OutputTarget, Preselection, TargetChange};
