use crate::{Error, Result};
use embedded_hal::digital::OutputPin;

/// A representation of a column of keys
pub(crate) struct KeyColumns<const COLS: usize, O: OutputPin> {
    pub(crate) pins: [O; COLS],
}

impl<const COLS: usize, O: OutputPin> KeyColumns<COLS, O> {
    pub(crate) fn new(pins: [O; COLS]) -> Self {
        Self { pins }
    }

    pub(crate) fn enable_column(&mut self, column: usize) -> Result<()> {
        self.pins[column].set_high().map_err(|_| Error::Unknown)
    }

    pub(crate) fn disable_column(&mut self, column: usize) -> Result<()> {
        self.pins[column].set_low().map_err(|_| Error::Unknown)
    }
}
