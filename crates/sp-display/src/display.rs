//! What a connected display looks like (PRD OUT-2).

/// Stable identity of a physical display: the Windows device path.
///
/// It survives unplug and replug, so the same projector is recognised when it comes
/// back (OUT-5). Two identical monitors have different device paths.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DisplayId(pub String);

/// A connected display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Display {
    /// Stable identity (device path).
    pub id: DisplayId,
    /// Friendly name shown in the "Show on" list.
    pub name: String,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Windows scaling, for example 100 or 150.
    pub scale_percent: u32,
    /// Left edge in the virtual desktop.
    pub x: i32,
    /// Top edge in the virtual desktop.
    pub y: i32,
    /// True for the main display.
    pub is_primary: bool,
}

/// Displays in a stable reading order: left to right, then top to bottom, then by id.
fn in_reading_order(displays: &[Display]) -> Vec<&Display> {
    let mut ordered: Vec<&Display> = displays.iter().collect();
    ordered.sort_by(|a, b| (a.x, a.y, &a.id).cmp(&(b.x, b.y, &b.id)));
    ordered
}

/// The big number each physical screen flashes when the user presses "Identify"
/// (OUT-2). Numbers start at 1 and follow the screens' positions, so identical
/// monitors still get different, predictable numbers.
#[must_use]
pub fn identify_numbers(displays: &[Display]) -> Vec<(DisplayId, u32)> {
    in_reading_order(displays)
        .into_iter()
        .zip(1u32..)
        .map(|(display, number)| (display.id.clone(), number))
        .collect()
}

/// The first non-primary display in reading order, if any (OUT-1).
pub(crate) fn first_non_primary(displays: &[Display]) -> Option<&Display> {
    in_reading_order(displays)
        .into_iter()
        .find(|display| !display.is_primary)
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::{Display, DisplayId};

    pub(crate) fn display(path: &str, x: i32, primary: bool) -> Display {
        Display {
            id: DisplayId(path.to_owned()),
            name: format!("Monitor {path}"),
            width: 1920,
            height: 1080,
            scale_percent: 100,
            x,
            y: 0,
            is_primary: primary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::display;
    use super::*;

    #[test]
    fn out_2_identify_numbers_follow_position_not_list_order() {
        // Same resolution and model; only the device path and position differ.
        let right = display("DISPLAY2", 1920, false);
        let left = display("DISPLAY1", 0, true);
        let numbers = identify_numbers(&[right, left]);
        assert_eq!(
            numbers,
            vec![
                (DisplayId("DISPLAY1".into()), 1),
                (DisplayId("DISPLAY2".into()), 2)
            ]
        );
    }

    #[test]
    fn out_2_identify_numbers_are_unique_for_identical_monitors_at_same_position() {
        // Mirrored identical monitors share a position; the device path breaks the tie.
        let a = display("B", 0, true);
        let b = display("A", 0, false);
        let numbers = identify_numbers(&[a, b]);
        assert_eq!(
            numbers,
            vec![(DisplayId("A".into()), 1), (DisplayId("B".into()), 2)]
        );
    }

    #[test]
    fn out_2_identify_with_no_displays_is_empty() {
        assert!(identify_numbers(&[]).is_empty());
    }
}
