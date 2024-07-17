use crate::{Error, Result};
use embedded_hal::digital::InputPin;

/// A representation of a row of keys
pub(crate) struct KeyRows<const ROWS: usize, I: InputPin> {
    pub(crate) pins: [I; ROWS],
}

impl<const ROWS: usize, I: InputPin> KeyRows<ROWS, I> {
    pub(crate) fn new(pins: [I; ROWS]) -> Self {
        Self { pins }
    }

    pub(crate) fn get_row(&mut self, row: usize) -> Result<bool> {
        self.pins[row].is_high().map_err(|_| Error::Unknown)
    }
}
