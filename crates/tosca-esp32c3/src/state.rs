/// The device state.
///
/// The state can **only** be accessed and modified within a route handler.
pub struct State<S>(pub S);

/// A trait for retrieving the substates of the device [`State`].
pub trait ValueFromRef {
    /// Retrieves the internal value from its reference.
    #[must_use]
    fn value_from_ref(&self) -> Self;
}

impl ValueFromRef for () {
    fn value_from_ref(&self) -> Self {}
}
