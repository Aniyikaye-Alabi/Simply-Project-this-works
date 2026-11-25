//! The seam between this crate's logic and the operating system.

use sp_core::UserError;

use crate::display::{Display, DisplayId};

/// Asks the operating system about displays. The Windows implementation lands later;
/// tests use [`crate::fake::FakeDisplayProvider`].
pub trait DisplayProvider {
    /// All connected displays right now.
    ///
    /// # Errors
    /// Returns a plain-language [`UserError`] if the system cannot be asked.
    fn displays(&self) -> Result<Vec<Display>, UserError>;

    /// The display the control window is on, if known. Used to refuse going live on
    /// the same screen (OUT-6).
    fn control_window_display(&self) -> Option<DisplayId>;
}
