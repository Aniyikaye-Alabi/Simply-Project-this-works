//! The ordered show list (PRD section 4: `CueStack`).

use crate::item::{Item, ItemId};

/// The show list. Top (index 0) is the highest priority; index order is the only
/// priority there is (AGENTS.md section 3, invariant 4).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CueStack {
    items: Vec<Item>,
}

impl CueStack {
    /// An empty show list.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Items in priority order, highest first.
    #[must_use]
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// Number of items.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// True when there are no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Add an item at the bottom of the list (lowest priority).
    pub fn push(&mut self, item: Item) {
        self.items.push(item);
    }

    /// Remove an item by id. Returns it, or `None` if it was not in the list.
    pub fn remove(&mut self, id: ItemId) -> Option<Item> {
        let index = self.items.iter().position(|item| item.id == id)?;
        Some(self.items.remove(index))
    }

    /// Move the item at `from` so it ends up at index `to` (SL-4). The result is
    /// always a permutation of the same items. Returns `false` and changes nothing
    /// if either index is out of range.
    ///
    /// The UI calls this once, on drop. A drag in progress never reorders the list.
    pub fn move_item(&mut self, from: usize, to: usize) -> bool {
        if from >= self.items.len() || to >= self.items.len() {
            return false;
        }
        let item = self.items.remove(from);
        self.items.insert(to, item);
        true
    }

    /// Turn an item's eye on or off (SL-5). Returns `false` if the id is unknown.
    pub fn set_visible(&mut self, id: ItemId, visible: bool) -> bool {
        match self.items.iter_mut().find(|item| item.id == id) {
            Some(item) => {
                item.visible = visible;
                true
            }
            None => false,
        }
    }

    /// Update an item's state, for example when a file goes missing. Returns `false`
    /// if the id is unknown.
    pub fn set_state(&mut self, id: ItemId, state: crate::item::ItemState) -> bool {
        match self.items.iter_mut().find(|item| item.id == id) {
            Some(item) => {
                item.state = state;
                true
            }
            None => false,
        }
    }
}

impl FromIterator<Item> for CueStack {
    fn from_iter<I: IntoIterator<Item = Item>>(iter: I) -> Self {
        Self {
            items: iter.into_iter().collect(),
        }
    }
}
