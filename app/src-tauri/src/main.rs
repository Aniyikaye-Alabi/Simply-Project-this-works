//! Executable entry point. All logic lives in the library crate.

// Hide the console window in release builds on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    simplyproject_app_lib::run();
}
