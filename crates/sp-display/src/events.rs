//! Hot-plug events, found by comparing two snapshots (PRD OUT-5).

use crate::display::{Display, DisplayId};

/// Something that changed between two lists of displays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayEvent {
    /// A display appeared (plugged in, woke up).
    Added(DisplayId),
    /// A display went away (unplugged, asleep).
    Removed(DisplayId),
    /// Same display, new size, scale or position. The output re-lays out.
    Changed(DisplayId),
}

impl DisplayEvent {
    fn id(&self) -> &DisplayId {
        match self {
            Self::Added(id) | Self::Removed(id) | Self::Changed(id) => id,
        }
    }
}

/// Compare the displays before and after. Events come out sorted by id so the result
/// is deterministic. Pure: the caller decides when to take each snapshot.
#[must_use]
pub fn diff(before: &[Display], after: &[Display]) -> Vec<DisplayEvent> {
    let mut events = Vec::new();
    for old in before {
        match after.iter().find(|new| new.id == old.id) {
            None => events.push(DisplayEvent::Removed(old.id.clone())),
            Some(new) if new != old => events.push(DisplayEvent::Changed(old.id.clone())),
            Some(_) => {}
        }
    }
    for new in after {
        if !before.iter().any(|old| old.id == new.id) {
            events.push(DisplayEvent::Added(new.id.clone()));
        }
    }
    events.sort_by(|a, b| a.id().cmp(b.id()));
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::test_support::display;

    #[test]
    fn out_5_unplug_is_a_removed_event() {
        let before = [display("P", 0, true), display("TV", 1920, false)];
        let after = [display("P", 0, true)];
        assert_eq!(
            diff(&before, &after),
            vec![DisplayEvent::Removed(DisplayId("TV".into()))]
        );
    }

    #[test]
    fn out_5_replug_is_an_added_event() {
        let before = [display("P", 0, true)];
        let after = [display("P", 0, true), display("TV", 1920, false)];
        assert_eq!(
            diff(&before, &after),
            vec![DisplayEvent::Added(DisplayId("TV".into()))]
        );
    }

    #[test]
    fn out_4_resolution_or_scale_change_is_a_changed_event() {
        let before = [display("TV", 0, true)];
        let mut tv = display("TV", 0, true);
        tv.scale_percent = 150;
        assert_eq!(
            diff(&before, &[tv]),
            vec![DisplayEvent::Changed(DisplayId("TV".into()))]
        );
    }

    #[test]
    fn out_5_no_change_means_no_events() {
        let both = [display("P", 0, true), display("TV", 1920, false)];
        assert!(diff(&both, &both).is_empty());
        assert!(diff(&[], &[]).is_empty());
    }

    #[test]
    fn out_5_events_are_sorted_by_id() {
        let after = [display("B", 0, true), display("A", 1920, false)];
        assert_eq!(
            diff(&[], &after),
            vec![
                DisplayEvent::Added(DisplayId("A".into())),
                DisplayEvent::Added(DisplayId("B".into())),
            ]
        );
    }
}
