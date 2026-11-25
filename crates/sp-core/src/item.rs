//! One thing that can be shown (PRD section 4: `Item` / `Source`).

/// Stable identity of an item. Never reused within a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemId(pub u64);

/// Whether an item can be shown right now (PRD section 4, section 9).
///
/// Only [`ItemState::Ready`] items can ever reach the audience. Every other state is
/// skipped by the resolver and flagged on the row, so the audience never sees an
/// error screen (AGENTS.md section 3, invariant 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemState {
    /// Loaded and showable.
    Ready,
    /// Still preparing (thumbnail, PDF render, video preload).
    Loading,
    /// The file was moved, renamed or deleted.
    Missing,
    /// The window, screen or device is gone.
    Unavailable,
    /// A remote stream dropped and is reconnecting.
    Reconnecting,
}

impl ItemState {
    /// True only for [`ItemState::Ready`].
    #[must_use]
    pub fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }
}

/// An entry in the show list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// Stable identity.
    pub id: ItemId,
    /// Name shown on the row and under the preview (SL-7).
    pub name: String,
    /// The eye: on means eligible to be shown (SL-5, SL-6).
    pub visible: bool,
    /// Whether the item can be shown right now.
    pub state: ItemState,
}

impl Item {
    /// True when the eye is on and the item is ready: the only items the resolver
    /// will pick (SL-6).
    #[must_use]
    pub fn is_eligible(&self) -> bool {
        self.visible && self.state.is_ready()
    }

    /// True when the eye is on but the item cannot be shown. The row gets a warning
    /// badge and the resolver skips it (PRD section 4).
    #[must_use]
    pub fn is_flagged(&self) -> bool {
        self.visible && !self.state.is_ready()
    }
}
