//! The resolver: decides what the audience sees (PRD section 4, AGENTS.md section 3).
//!
//! ```text
//! program(output) =
//!   if blackout            -> BLACK
//!   else if freeze         -> last_frame
//!   else first item in cue_stack where visible == true and state == ready
//!   else                   -> standby
//! ```
//!
//! [`resolve`] is a pure function: no I/O, no clock, no randomness. The same input
//! always gives the same output, and it never fails.

use crate::cue_stack::CueStack;
use crate::item::Item;

/// The operator's override switches.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Controls {
    /// Show black, ignoring the list. Beats everything, including freeze.
    pub blackout: bool,
    /// Hold the last frame while the operator changes things.
    pub freeze: bool,
}

/// What the output shows right now (PRD section 4: `Program`).
///
/// `F` is the caller's frame type. The resolver never looks inside it.
#[derive(Debug, PartialEq, Eq)]
pub enum Program<'a, F> {
    /// Solid black (blackout).
    Black,
    /// The held last frame (freeze).
    Frozen(&'a F),
    /// This item is on screen.
    Item(&'a Item),
    /// Nothing is eligible. The output shows the standby screen, never a desktop.
    Standby,
}

/// Decide what the output shows (SL-5, SL-6, PRD section 4).
///
/// - Blackout beats everything, including freeze.
/// - Freeze holds `last_frame`. If there is no frame to hold yet, freeze has nothing
///   to show, so the normal rule applies. The audience never sees an error.
/// - Otherwise the first item with its eye on that is ready wins. Items that are on
///   but not ready (missing, unavailable, reconnecting, loading) are skipped.
/// - If nothing is eligible, or the list is empty, the result is standby.
#[must_use]
pub fn resolve<'a, F>(
    stack: &'a CueStack,
    controls: Controls,
    last_frame: Option<&'a F>,
) -> Program<'a, F> {
    if controls.blackout {
        return Program::Black;
    }
    if controls.freeze {
        if let Some(frame) = last_frame {
            return Program::Frozen(frame);
        }
    }
    match stack.items().iter().find(|item| item.is_eligible()) {
        Some(item) => Program::Item(item),
        None => Program::Standby,
    }
}

/// The first eligible item that is not the one on screen. The row for it shows
/// "Next in line" (SL-5).
#[must_use]
pub fn next_in_line<'a>(stack: &'a CueStack, on_screen: &Item) -> Option<&'a Item> {
    stack
        .items()
        .iter()
        .filter(|item| item.is_eligible())
        .find(|item| item.id != on_screen.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::{ItemId, ItemState};
    use proptest::prelude::*;

    /// A stand-in frame type for tests.
    #[derive(Debug, PartialEq, Eq)]
    struct Frame(u32);

    fn item(id: u64, visible: bool, state: ItemState) -> Item {
        Item {
            id: ItemId(id),
            name: format!("item {id}"),
            visible,
            state,
        }
    }

    fn on_ready(id: u64) -> Item {
        item(id, true, ItemState::Ready)
    }

    fn off(id: u64) -> Item {
        item(id, false, ItemState::Ready)
    }

    fn stack(items: Vec<Item>) -> CueStack {
        items.into_iter().collect()
    }

    fn pick(stack: &CueStack, controls: Controls) -> Option<ItemId> {
        match resolve::<Frame>(stack, controls, None) {
            Program::Item(item) => Some(item.id),
            _ => None,
        }
    }

    const LIVE: Controls = Controls {
        blackout: false,
        freeze: false,
    };

    // --- One test per PRD section 4 row and section 9 edge-case row -------------

    #[test]
    fn sl_6_no_eye_on_shows_standby() {
        let s = stack(vec![off(1), off(2)]);
        assert_eq!(resolve::<Frame>(&s, LIVE, None), Program::Standby);
    }

    #[test]
    fn sl_6_empty_list_shows_standby() {
        let s = CueStack::new();
        assert_eq!(resolve::<Frame>(&s, LIVE, None), Program::Standby);
    }

    #[test]
    fn sl_6_highest_enabled_ready_item_wins() {
        let s = stack(vec![off(1), on_ready(2), on_ready(3)]);
        assert_eq!(pick(&s, LIVE), Some(ItemId(2)));
    }

    #[test]
    fn sl_6_top_item_missing_falls_to_next_eligible_and_row_is_flagged() {
        let top = item(1, true, ItemState::Missing);
        let s = stack(vec![top.clone(), on_ready(2)]);
        assert_eq!(pick(&s, LIVE), Some(ItemId(2)));
        assert!(top.is_flagged());
        assert!(!top.is_eligible());
    }

    #[test]
    fn sl_6_every_not_ready_state_is_skipped() {
        for state in [
            ItemState::Loading,
            ItemState::Missing,
            ItemState::Unavailable,
            ItemState::Reconnecting,
        ] {
            let s = stack(vec![item(1, true, state), on_ready(2)]);
            assert_eq!(pick(&s, LIVE), Some(ItemId(2)), "{state:?} must be skipped");
        }
    }

    #[test]
    fn sl_6_only_not_ready_items_show_standby_never_an_error() {
        let s = stack(vec![
            item(1, true, ItemState::Missing),
            item(2, true, ItemState::Loading),
        ]);
        assert_eq!(resolve::<Frame>(&s, LIVE, None), Program::Standby);
    }

    #[test]
    fn sl_4_reorder_changes_program() {
        let mut s = stack(vec![on_ready(1), on_ready(2), on_ready(3)]);
        assert_eq!(pick(&s, LIVE), Some(ItemId(1)));
        assert!(s.move_item(2, 0));
        assert_eq!(pick(&s, LIVE), Some(ItemId(3)));
    }

    #[test]
    fn a2_dragging_third_above_second_switches_to_third() {
        // A2: add 3 files, eyes on 2 and 3. Item 2 shows; dragging 3 above 2 shows 3.
        let mut s = stack(vec![off(1), on_ready(2), on_ready(3)]);
        assert_eq!(pick(&s, LIVE), Some(ItemId(2)));
        assert!(s.move_item(2, 1));
        assert_eq!(pick(&s, LIVE), Some(ItemId(3)));
    }

    #[test]
    fn sl_5_eye_toggle_changes_program() {
        let mut s = stack(vec![on_ready(1), on_ready(2)]);
        assert!(s.set_visible(ItemId(1), false));
        assert_eq!(pick(&s, LIVE), Some(ItemId(2)));
        assert!(s.set_visible(ItemId(2), false));
        assert_eq!(resolve::<Frame>(&s, LIVE, None), Program::Standby);
    }

    #[test]
    fn sl_6_item_removed_while_on_screen_falls_to_next_eligible() {
        let mut s = stack(vec![on_ready(1), on_ready(2)]);
        assert_eq!(pick(&s, LIVE), Some(ItemId(1)));
        assert!(s.remove(ItemId(1)).is_some());
        assert_eq!(pick(&s, LIVE), Some(ItemId(2)));
        assert!(s.remove(ItemId(2)).is_some());
        assert_eq!(resolve::<Frame>(&s, LIVE, None), Program::Standby);
    }

    #[test]
    fn a4_item_going_unavailable_falls_to_next_eligible() {
        // A4: capture a window, close it. The row is flagged and output falls through.
        let mut s = stack(vec![on_ready(1), on_ready(2)]);
        assert!(s.set_state(ItemId(1), ItemState::Unavailable));
        assert_eq!(pick(&s, LIVE), Some(ItemId(2)));
    }

    #[test]
    fn a5_blackout_shows_black_and_toggles_back() {
        let s = stack(vec![on_ready(1)]);
        let black = Controls {
            blackout: true,
            freeze: false,
        };
        assert_eq!(resolve::<Frame>(&s, black, None), Program::Black);
        assert_eq!(pick(&s, LIVE), Some(ItemId(1)));
    }

    #[test]
    fn sl_6_blackout_beats_empty_list_and_standby() {
        let s = CueStack::new();
        let black = Controls {
            blackout: true,
            freeze: false,
        };
        assert_eq!(resolve::<Frame>(&s, black, None), Program::Black);
    }

    #[test]
    fn sl_6_blackout_beats_freeze() {
        let s = stack(vec![on_ready(1)]);
        let frame = Frame(7);
        let both = Controls {
            blackout: true,
            freeze: true,
        };
        assert_eq!(resolve(&s, both, Some(&frame)), Program::Black);
    }

    #[test]
    fn sl_6_freeze_holds_last_frame_even_when_list_changes() {
        let mut s = stack(vec![on_ready(1)]);
        let frame = Frame(7);
        let frozen = Controls {
            blackout: false,
            freeze: true,
        };
        assert_eq!(resolve(&s, frozen, Some(&frame)), Program::Frozen(&frame));
        assert!(s.remove(ItemId(1)).is_some());
        assert_eq!(resolve(&s, frozen, Some(&frame)), Program::Frozen(&frame));
    }

    #[test]
    fn sl_6_freeze_without_a_frame_falls_back_to_normal_rule() {
        let s = stack(vec![on_ready(1)]);
        let frozen = Controls {
            blackout: false,
            freeze: true,
        };
        assert_eq!(pick(&s, frozen), Some(ItemId(1)));
        let empty = CueStack::new();
        assert_eq!(resolve::<Frame>(&empty, frozen, None), Program::Standby);
    }

    #[test]
    fn sl_5_next_in_line_is_first_eligible_below_the_one_on_screen() {
        let s = stack(vec![
            on_ready(1),
            off(2),
            item(3, true, ItemState::Missing),
            on_ready(4),
        ]);
        let on_screen = s.items().first();
        assert!(on_screen.is_some());
        if let Some(on_screen) = on_screen {
            let next = next_in_line(&s, on_screen).map(|i| i.id);
            assert_eq!(next, Some(ItemId(4)));
        }
    }

    // --- CueStack edge handling -------------------------------------------------

    #[test]
    fn sl_4_move_out_of_range_changes_nothing() {
        let mut s = stack(vec![on_ready(1), on_ready(2)]);
        let before = s.clone();
        assert!(!s.move_item(0, 2));
        assert!(!s.move_item(5, 0));
        assert_eq!(s, before);
    }

    #[test]
    fn sl_4_unknown_ids_are_reported_not_panicked() {
        let mut s = stack(vec![on_ready(1)]);
        assert!(!s.set_visible(ItemId(99), true));
        assert!(!s.set_state(ItemId(99), ItemState::Missing));
        assert!(s.remove(ItemId(99)).is_none());
        assert_eq!(s.len(), 1);
        assert!(!s.is_empty());
    }

    // --- Property tests: the invariants of AGENTS.md section 3 ------------------

    fn arb_state() -> impl Strategy<Value = ItemState> {
        prop_oneof![
            Just(ItemState::Ready),
            Just(ItemState::Loading),
            Just(ItemState::Missing),
            Just(ItemState::Unavailable),
            Just(ItemState::Reconnecting),
        ]
    }

    /// A stack with unique ids (the index), arbitrary eyes and states.
    fn arb_stack() -> impl Strategy<Value = CueStack> {
        prop::collection::vec((any::<bool>(), arb_state()), 0..24).prop_map(|rows| {
            rows.into_iter()
                .enumerate()
                .map(|(i, (visible, state))| item(i as u64, visible, state))
                .collect()
        })
    }

    fn arb_controls() -> impl Strategy<Value = Controls> {
        (any::<bool>(), any::<bool>()).prop_map(|(blackout, freeze)| Controls { blackout, freeze })
    }

    proptest! {
        /// Invariant 3: blackout beats everything.
        #[test]
        fn prop_blackout_beats_everything(
            s in arb_stack(),
            freeze in any::<bool>(),
            has_frame in any::<bool>(),
        ) {
            let frame = Frame(1);
            let controls = Controls { blackout: true, freeze };
            let last = has_frame.then_some(&frame);
            prop_assert_eq!(resolve(&s, controls, last), Program::Black);
        }

        /// Invariant 2: only a visible, ready item can ever be on screen.
        #[test]
        fn prop_never_shows_a_not_ready_or_hidden_item(s in arb_stack(), c in arb_controls()) {
            if let Program::Item(item) = resolve::<Frame>(&s, c, None) {
                prop_assert!(item.visible);
                prop_assert!(item.state.is_ready());
            }
        }

        /// Invariant 4: index order is the only priority. The pick is the first
        /// eligible item, and nothing above it is eligible.
        #[test]
        fn prop_pick_is_first_eligible(s in arb_stack()) {
            let expected = s.items().iter().find(|i| i.is_eligible()).map(|i| i.id);
            prop_assert_eq!(pick(&s, LIVE), expected);
        }

        /// Invariant 6: nothing eligible means standby, never anything else.
        #[test]
        fn prop_nothing_eligible_means_standby(s in arb_stack()) {
            if !s.items().iter().any(|i| i.is_eligible()) {
                prop_assert_eq!(resolve::<Frame>(&s, LIVE, None), Program::Standby);
            }
        }

        /// Invariant 5: same input, same output.
        #[test]
        fn prop_resolver_is_idempotent(
            s in arb_stack(),
            c in arb_controls(),
            has_frame in any::<bool>(),
        ) {
            let frame = Frame(3);
            let last = has_frame.then_some(&frame);
            prop_assert_eq!(resolve(&s, c, last), resolve(&s, c, last));
        }

        /// Freeze without blackout holds the frame whenever there is one.
        #[test]
        fn prop_freeze_holds_the_frame(s in arb_stack()) {
            let frame = Frame(9);
            let c = Controls { blackout: false, freeze: true };
            prop_assert_eq!(resolve(&s, c, Some(&frame)), Program::Frozen(&frame));
        }

        /// Reordering is a permutation: same items, nothing lost or duplicated.
        #[test]
        fn prop_move_item_is_a_permutation(s in arb_stack(), from in 0usize..30, to in 0usize..30) {
            let mut moved = s.clone();
            let ok = moved.move_item(from, to);
            let mut before: Vec<_> = s.items().iter().map(|i| i.id).collect();
            let mut after: Vec<_> = moved.items().iter().map(|i| i.id).collect();
            prop_assert_eq!(ok, from < s.len() && to < s.len());
            before.sort();
            after.sort();
            prop_assert_eq!(before, after);
        }

        /// After a move, the item sits at the requested index.
        #[test]
        fn prop_move_item_lands_at_target(s in arb_stack(), from in 0usize..30, to in 0usize..30) {
            let mut moved = s.clone();
            if moved.move_item(from, to) {
                let id = s.items().get(from).map(|i| i.id);
                prop_assert_eq!(moved.items().get(to).map(|i| i.id), id);
            }
        }
    }
}
