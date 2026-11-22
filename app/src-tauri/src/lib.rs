//! SimplyProject desktop shell.
//!
//! This crate only wires windows and IPC to the `sp-*` crates. Logic belongs in those
//! crates, not here (AGENTS.md section 4.1).

/// Starts the app. A start-up failure is reported and the process exits cleanly,
/// because panics must never reach the user (AGENTS.md section 4.2).
pub fn run() {
    let result = tauri::Builder::default().run(tauri::generate_context!());
    if let Err(err) = result {
        eprintln!("SimplyProject could not start: {err}");
        std::process::exit(1);
    }
}
