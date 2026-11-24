//! Monitors, extend mode, hot-plug and the output window (PRD OUT-1 to OUT-6).
//!
//! OS access sits behind the `DisplayProvider` and `OutputRenderer` traits, added in
//! M1, each with a Windows implementation and a fake for tests.
