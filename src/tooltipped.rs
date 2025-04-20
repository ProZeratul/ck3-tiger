//! A helper type used for effects and triggers, which tracks what kind of tooltipping to expect.
//! This affects which errors are logged about them. Some things only matter if an item is being
//! tooltipped.

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Tooltipped {
    No,
    Yes,
    /// for triggers
    FailuresOnly,
    /// for effects
    Past,
    #[cfg(feature = "hoi4")]
    /// for triggers that pretend to be inside a `custom_override_tooltip`
    Inner,
}

impl Tooltipped {
    pub fn is_tooltipped(self) -> bool {
        !matches!(self, Tooltipped::No)
    }
}
