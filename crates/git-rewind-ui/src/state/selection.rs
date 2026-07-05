/// A reusable, presentation-independent selection utility for lists/panels.
/// Keeps track of a selected index and enforces bounds boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Selection {
    index: usize,
}

impl Selection {
    /// Creates a new Selection with a specific starting index.
    pub fn new(index: usize) -> Self {
        Self { index }
    }

    /// Returns the currently selected index.
    pub fn selected(&self) -> usize {
        self.index
    }

    /// Moves selection to the next index, bounded by the given length.
    pub fn next(&mut self, len: usize) {
        if len > 0 && self.index + 1 < len {
            self.index += 1;
        }
    }

    /// Moves selection to the previous index, bounded by 0.
    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    /// Moves selection to the first index (0), if the list is not empty.
    pub fn first(&mut self, len: usize) {
        if len > 0 {
            self.index = 0;
        }
    }

    /// Moves selection to the last index (len - 1), if the list is not empty.
    pub fn last(&mut self, len: usize) {
        if len > 0 {
            self.index = len - 1;
        }
    }

    /// Clamps the selected index within the valid bounds `[0, len - 1]`
    /// or sets it to 0 if the list is empty.
    pub fn clamp(&mut self, len: usize) {
        if len == 0 {
            self.index = 0;
        } else if self.index >= len {
            self.index = len - 1;
        }
    }
}
