//! A fake display provider for tests: no hardware, fully scriptable.

use sp_core::UserError;

use crate::display::{Display, DisplayId};
use crate::provider::DisplayProvider;

/// A [`DisplayProvider`] whose displays the test controls.
#[derive(Debug, Clone, Default)]
pub struct FakeDisplayProvider {
    displays: Vec<Display>,
    control: Option<DisplayId>,
    failing: bool,
}

impl FakeDisplayProvider {
    /// A provider with these displays and the control window on `control`.
    #[must_use]
    pub fn new(displays: Vec<Display>, control: Option<DisplayId>) -> Self {
        Self {
            displays,
            control,
            failing: false,
        }
    }

    /// Replace the connected displays (simulates plug, unplug, resolution change).
    pub fn set_displays(&mut self, displays: Vec<Display>) {
        self.displays = displays;
    }

    /// Move the control window to another display.
    pub fn set_control(&mut self, control: Option<DisplayId>) {
        self.control = control;
    }

    /// Make `displays()` fail, to test the error path.
    pub fn set_failing(&mut self, failing: bool) {
        self.failing = failing;
    }
}

impl DisplayProvider for FakeDisplayProvider {
    fn displays(&self) -> Result<Vec<Display>, UserError> {
        if self.failing {
            return Err(UserError::new(
                "displays_unavailable",
                "We couldn't check which screens are connected.",
                "Check the cable, then press Refresh.",
            ));
        }
        Ok(self.displays.clone())
    }

    fn control_window_display(&self) -> Option<DisplayId> {
        self.control.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::test_support::display;

    #[test]
    fn fake_reports_what_it_was_given_and_what_changes() {
        let mut fake = FakeDisplayProvider::new(vec![display("P", 0, true)], None);
        assert_eq!(fake.displays().map(|d| d.len()), Ok(1));
        fake.set_displays(vec![]);
        assert_eq!(fake.displays().map(|d| d.len()), Ok(0));
        fake.set_control(Some(DisplayId("P".into())));
        assert_eq!(fake.control_window_display(), Some(DisplayId("P".into())));
    }

    #[test]
    fn fake_failure_is_a_plain_language_user_error() {
        let mut fake = FakeDisplayProvider::default();
        fake.set_failing(true);
        let err = fake.displays().err();
        assert!(err.is_some());
        if let Some(err) = err {
            assert_eq!(err.code, "displays_unavailable");
            assert!(!err.message.is_empty() && !err.suggested_fix.is_empty());
        }
    }
}
