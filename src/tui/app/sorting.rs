use crate::db::Note;

/// Note sorting mode (cycle with 's' key).
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SortMode {
	UpdatedDesc,
	UpdatedAsc,
	TitleAsc,
	TitleDesc,
	CreatedDesc,
	CreatedAsc,
}

impl SortMode {
	/// Returns the next sort mode in the cycle.
	pub const fn next(self) -> Self {
		match self {
			Self::UpdatedDesc => Self::UpdatedAsc,
			Self::UpdatedAsc => Self::TitleAsc,
			Self::TitleAsc => Self::TitleDesc,
			Self::TitleDesc => Self::CreatedDesc,
			Self::CreatedDesc => Self::CreatedAsc,
			Self::CreatedAsc => Self::UpdatedDesc,
		}
	}

	/// Returns the display name for the UI.
	pub const fn name(self) -> &'static str {
		match self {
			Self::UpdatedDesc => "Updated ↓",
			Self::UpdatedAsc => "Updated ↑",
			Self::TitleAsc => "Title A→Z",
			Self::TitleDesc => "Title Z→A",
			Self::CreatedDesc => "Created ↓",
			Self::CreatedAsc => "Created ↑",
		}
	}

	/// Sorts notes in-place according to this mode.
	pub fn sort_notes(self, notes: &mut [Note]) {
		notes.sort_unstable_by(|a, b| match self {
			Self::UpdatedDesc => b.updated_at.cmp(&a.updated_at),
			Self::UpdatedAsc => a.updated_at.cmp(&b.updated_at),
			Self::TitleAsc => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
			Self::TitleDesc => b.title.to_lowercase().cmp(&a.title.to_lowercase()),
			Self::CreatedDesc => b.created_at.cmp(&a.created_at),
			Self::CreatedAsc => a.created_at.cmp(&b.created_at),
		});
	}
}
