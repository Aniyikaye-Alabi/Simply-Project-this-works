//! Choosing where to show, and keeping the choice safe (PRD OUT-1, OUT-3, OUT-5, OUT-6).

use sp_core::UserError;

use crate::display::{first_non_primary, Display, DisplayId};

/// What the app should preselect on launch (OUT-1, OUT-3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Preselection {
    /// A second screen exists: preselect it and ask for one-click confirmation.
    Chosen(DisplayId),
    /// Only one screen (or none): show the "Connect your projector or TV" card with
    /// the "Extend my screens" button (OUT-3, scenario A6).
    NoProjector,
}

/// Pick the display to preselect as "Show on": the first non-primary display, left to
/// right (OUT-1). With no second display the answer is [`Preselection::NoProjector`].
#[must_use]
pub fn preselect(displays: &[Display]) -> Preselection {
    match first_non_primary(displays) {
        Some(display) => Preselection::Chosen(display.id.clone()),
        None => Preselection::NoProjector,
    }
}

/// Whether it is safe to go live right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveCheck {
    /// Safe to go live.
    Ready,
    /// No screen has been chosen yet.
    NoDisplayChosen,
    /// The chosen screen is not connected.
    Disconnected,
    /// The chosen screen is the one this app is on (OUT-6).
    SameAsControlWindow,
}

impl LiveCheck {
    /// True only for [`LiveCheck::Ready`].
    #[must_use]
    pub fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }

    /// The plain-language reason going live is blocked, or `None` when it is safe.
    #[must_use]
    pub fn to_user_error(self) -> Option<UserError> {
        match self {
            Self::Ready => None,
            Self::NoDisplayChosen => Some(UserError::new(
                "no_display_chosen",
                "No screen is chosen yet.",
                "Connect your projector or TV, then pick it under Show on.",
            )),
            Self::Disconnected => Some(UserError::new(
                "display_disconnected",
                "The screen you picked is not connected.",
                "Plug it back in, or pick another screen.",
            )),
            Self::SameAsControlWindow => Some(UserError::new(
                "display_is_control_screen",
                "The audience would see this app.",
                "Pick another screen.",
            )),
        }
    }
}

/// Decide whether going live is allowed (OUT-6, scenario A6). Going live is blocked
/// unless a connected screen is chosen and it is not the screen this app is on.
#[must_use]
pub fn check_live(
    chosen: Option<&DisplayId>,
    control_window: Option<&DisplayId>,
    displays: &[Display],
) -> LiveCheck {
    let Some(chosen) = chosen else {
        return LiveCheck::NoDisplayChosen;
    };
    if !displays.iter().any(|display| &display.id == chosen) {
        return LiveCheck::Disconnected;
    }
    if control_window == Some(chosen) {
        return LiveCheck::SameAsControlWindow;
    }
    LiveCheck::Ready
}

/// What happened to the chosen screen after a refresh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetChange {
    /// Nothing to report.
    None,
    /// The chosen screen went away: pause the output and show a banner (OUT-5).
    Paused,
    /// The same screen is back: resume on it automatically (OUT-5).
    Resumed,
}

/// The remembered "Show on" choice. It is remembered by display id, so the same
/// projector is recognised after an unplug, a replug or a change in display order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OutputTarget {
    chosen: Option<DisplayId>,
    connected: bool,
}

impl OutputTarget {
    /// No screen chosen yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Choose a screen. Returns `false` and changes nothing if it is not connected.
    pub fn choose(&mut self, id: &DisplayId, displays: &[Display]) -> bool {
        if !displays.iter().any(|display| &display.id == id) {
            return false;
        }
        self.chosen = Some(id.clone());
        self.connected = true;
        true
    }

    /// The chosen screen, if any.
    #[must_use]
    pub fn chosen(&self) -> Option<&DisplayId> {
        self.chosen.as_ref()
    }

    /// True while the chosen screen is unplugged. The output is paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.chosen.is_some() && !self.connected
    }

    /// Feed in a fresh list of displays. Reports a pause when the chosen screen
    /// disappears and a resume when that same screen returns (OUT-5).
    pub fn update(&mut self, displays: &[Display]) -> TargetChange {
        let Some(chosen) = &self.chosen else {
            return TargetChange::None;
        };
        let present = displays.iter().any(|display| &display.id == chosen);
        let change = match (self.connected, present) {
            (true, false) => TargetChange::Paused,
            (false, true) => TargetChange::Resumed,
            _ => TargetChange::None,
        };
        self.connected = present;
        change
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::test_support::display;
    use crate::fake::FakeDisplayProvider;
    use crate::provider::DisplayProvider;

    fn id(path: &str) -> DisplayId {
        DisplayId(path.to_owned())
    }

    // --- OUT-1, OUT-3: preselect --------------------------------------------------

    #[test]
    fn out_1_preselects_the_non_primary_display() {
        let displays = [
            display("LAPTOP", 0, true),
            display("PROJECTOR", 1920, false),
        ];
        assert_eq!(preselect(&displays), Preselection::Chosen(id("PROJECTOR")));
    }

    #[test]
    fn out_1_preselect_does_not_depend_on_list_order() {
        let displays = [
            display("PROJECTOR", 1920, false),
            display("LAPTOP", 0, true),
        ];
        assert_eq!(preselect(&displays), Preselection::Chosen(id("PROJECTOR")));
    }

    #[test]
    fn out_1_with_two_extra_displays_picks_the_leftmost() {
        let displays = [
            display("LAPTOP", 0, true),
            display("FAR", 3840, false),
            display("NEAR", 1920, false),
        ];
        assert_eq!(preselect(&displays), Preselection::Chosen(id("NEAR")));
    }

    #[test]
    fn a6_one_display_only_means_no_projector() {
        assert_eq!(
            preselect(&[display("LAPTOP", 0, true)]),
            Preselection::NoProjector
        );
    }

    #[test]
    fn out_3_no_displays_at_all_means_no_projector() {
        assert_eq!(preselect(&[]), Preselection::NoProjector);
    }

    // --- OUT-6: never go live on the control window's screen ---------------------

    #[test]
    fn out_6_same_screen_as_the_control_window_blocks_live() {
        let displays = [
            display("LAPTOP", 0, true),
            display("PROJECTOR", 1920, false),
        ];
        let check = check_live(Some(&id("LAPTOP")), Some(&id("LAPTOP")), &displays);
        assert_eq!(check, LiveCheck::SameAsControlWindow);
        assert!(!check.is_ready());
        let err = check.to_user_error();
        assert_eq!(
            err.as_ref().map(|e| e.message.as_str()),
            Some("The audience would see this app.")
        );
        assert_eq!(
            err.as_ref().map(|e| e.suggested_fix.as_str()),
            Some("Pick another screen.")
        );
    }

    #[test]
    fn out_6_a_different_connected_screen_is_ready() {
        let displays = [
            display("LAPTOP", 0, true),
            display("PROJECTOR", 1920, false),
        ];
        let check = check_live(Some(&id("PROJECTOR")), Some(&id("LAPTOP")), &displays);
        assert_eq!(check, LiveCheck::Ready);
        assert!(check.to_user_error().is_none());
    }

    #[test]
    fn a6_nothing_chosen_blocks_live() {
        let displays = [display("LAPTOP", 0, true)];
        let check = check_live(None, Some(&id("LAPTOP")), &displays);
        assert_eq!(check, LiveCheck::NoDisplayChosen);
        assert!(check.to_user_error().is_some());
    }

    #[test]
    fn out_5_chosen_screen_not_connected_blocks_live() {
        let displays = [display("LAPTOP", 0, true)];
        let check = check_live(Some(&id("PROJECTOR")), Some(&id("LAPTOP")), &displays);
        assert_eq!(check, LiveCheck::Disconnected);
        assert!(check.to_user_error().is_some());
    }

    #[test]
    fn out_6_unknown_control_screen_does_not_block_a_connected_choice() {
        let displays = [
            display("LAPTOP", 0, true),
            display("PROJECTOR", 1920, false),
        ];
        assert_eq!(
            check_live(Some(&id("PROJECTOR")), None, &displays),
            LiveCheck::Ready
        );
    }

    #[test]
    fn out_6_every_block_reason_is_plain_language_with_a_fix() {
        for check in [
            LiveCheck::NoDisplayChosen,
            LiveCheck::Disconnected,
            LiveCheck::SameAsControlWindow,
        ] {
            let err = check.to_user_error();
            assert!(err.is_some());
            if let Some(err) = err {
                assert!(!err.message.is_empty());
                assert!(!err.suggested_fix.is_empty());
            }
        }
    }

    // --- OUT-5: hot-plug ---------------------------------------------------------

    #[test]
    fn out_5_choose_refuses_a_screen_that_is_not_connected() {
        let displays = [display("LAPTOP", 0, true)];
        let mut target = OutputTarget::new();
        assert!(!target.choose(&id("PROJECTOR"), &displays));
        assert_eq!(target.chosen(), None);
    }

    #[test]
    fn out_5_unplug_pauses_and_replug_resumes_on_the_same_display() {
        let laptop = display("LAPTOP", 0, true);
        let projector = display("PROJECTOR", 1920, false);
        let mut os =
            FakeDisplayProvider::new(vec![laptop.clone(), projector.clone()], Some(id("LAPTOP")));
        let mut target = OutputTarget::new();
        let shown = os.displays().unwrap_or_default();
        assert!(target.choose(&id("PROJECTOR"), &shown));
        assert!(!target.is_paused());

        // Unplug the projector.
        os.set_displays(vec![laptop.clone()]);
        let shown = os.displays().unwrap_or_default();
        assert_eq!(target.update(&shown), TargetChange::Paused);
        assert!(target.is_paused());
        assert_eq!(
            check_live(target.chosen(), Some(&id("LAPTOP")), &shown),
            LiveCheck::Disconnected
        );
        // Refreshing again while it is still gone reports nothing new.
        assert_eq!(target.update(&shown), TargetChange::None);

        // Plug it back in, listed first this time and at a different position.
        let mut back = projector;
        back.x = -1920;
        os.set_displays(vec![back, laptop]);
        let shown = os.displays().unwrap_or_default();
        assert_eq!(target.update(&shown), TargetChange::Resumed);
        assert!(!target.is_paused());
        assert_eq!(target.chosen(), Some(&id("PROJECTOR")));
    }

    #[test]
    fn out_5_updates_with_no_choice_report_nothing() {
        let mut target = OutputTarget::new();
        assert_eq!(
            target.update(&[display("LAPTOP", 0, true)]),
            TargetChange::None
        );
        assert_eq!(target.update(&[]), TargetChange::None);
        assert!(!target.is_paused());
    }

    #[test]
    fn out_4_a_resolution_change_does_not_pause_the_output() {
        let mut projector = display("PROJECTOR", 1920, false);
        let mut target = OutputTarget::new();
        assert!(target.choose(&id("PROJECTOR"), std::slice::from_ref(&projector)));
        projector.width = 1280;
        projector.height = 720;
        assert_eq!(target.update(&[projector]), TargetChange::None);
        assert!(!target.is_paused());
    }

    #[test]
    fn out_5_all_displays_gone_pauses_instead_of_crashing() {
        let projector = display("PROJECTOR", 0, true);
        let mut target = OutputTarget::new();
        assert!(target.choose(&id("PROJECTOR"), std::slice::from_ref(&projector)));
        assert_eq!(target.update(&[]), TargetChange::Paused);
    }
}
