/// The latest state of all the keys
#[derive(Debug, PartialEq, Eq)]
pub struct KeyState<const ROWS: usize, const COLS: usize> {
    pub(crate) state: [[i8; ROWS]; COLS],
}

impl<const ROWS: usize, const COLS: usize> KeyState<ROWS, COLS> {
    pub(crate) const MINIMUM: i8 = 0;
    pub(crate) const MAXIMUM: i8 = 3;

    pub(crate) fn new() -> Self {
        Self {
            state: [[0; ROWS]; COLS],
        }
    }
}
