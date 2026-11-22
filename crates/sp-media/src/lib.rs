//! Thumbnails, PDF page rendering and video preload (PRD SL-2, SL-8, SL-9, PRJ-5).
//!
//! Decoders are attack surface: every parser here must run on bounded resources and
//! have a fuzz target (AGENTS.md section 7.2).
