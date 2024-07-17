//! This crate provides an abstraction over GPIO matrices for the purposes of
//! creating keyboards and keypads. It is based on [`embedded-hal`]
//! traits. Specifically `Input` and `Output`, with an optional need for
//! `DelayNs` if the `debounce` feature is enabled.

#![doc(html_root_url = "https://docs.rs/embedded-keymatrix/latest")]
#![cfg_attr(not(test), no_std)]

use embedded_hal::digital::{InputPin, OutputPin};

/// Result type alias
pub type Result<T> = core::result::Result<T, Error>;

/// Errors produced by this crate
pub enum Error {
    /// Unknown errors
    Unknown,
}

/// A representation of a column of keys
pub struct KeyColumns<const COLS: usize, O: OutputPin> {
    pins: [O; COLS],
}

impl<const COLS: usize, O: OutputPin> KeyColumns<COLS, O> {
    fn new(pins: [O; COLS]) -> Self {
        Self { pins }
    }

    fn enable_column(&mut self, column: usize) -> Result<()> {
        self.pins[column].set_high().map_err(|_| Error::Unknown)
    }

    fn disable_column(&mut self, column: usize) -> Result<()> {
        self.pins[column].set_low().map_err(|_| Error::Unknown)
    }
}

/// A representation of a row of keys
pub struct KeyRows<const ROWS: usize, I: InputPin> {
    pins: [I; ROWS],
}

impl<const ROWS: usize, I: InputPin> KeyRows<ROWS, I> {
    fn new(pins: [I; ROWS]) -> Self {
        Self { pins }
    }

    fn get_row(&mut self, row: usize) -> Result<bool> {
        self.pins[row].is_high().map_err(|_| Error::Unknown)
    }
}

/// A mapper from the keymatrix to actual keycodes
pub struct KeyMap<const ROWS: usize, const COLS: usize, I: InputPin, O: OutputPin> {
    rows: [I; ROWS],
    cols: [O; COLS],
}

/// The latest state of all the keys
pub struct KeyState<const ROWS: usize, const COLS: usize, I: InputPin, O: OutputPin> {
    rows: [I; ROWS],
    cols: [O; COLS],
}

/// Matrix of [`InputPin`]s and [`OutputPin`]s describing a keyboard
pub struct KeyMatrix<const ROWS: usize, const COLS: usize, I: InputPin, O: OutputPin> {
    rows: KeyRows<ROWS, I>,
    cols: KeyColumns<COLS, O>,
}

impl<const ROWS: usize, const COLS: usize, I: InputPin, O: OutputPin> KeyMatrix<ROWS, COLS, I, O> {
    /// Instantiate a new matrix with the given rows and columns
    pub fn new(cols: [O; COLS], rows: [I; ROWS]) -> Self {
        Self {
            cols: KeyColumns::new(cols),
            rows: KeyRows::new(rows),
        }
    }
}

impl<const ROWS: usize, const COLS: usize, I: InputPin, O: OutputPin> KeyMatrix<ROWS, COLS, I, O> {
    /// Scan the current state of the key matrix.
    pub fn scan_matrix(&mut self) -> Result<KeyState<ROWS, COLS, I, O>> {
        // iterate over columns, enabling each along the way, then check the
        // state of each row by mapping each row to its current state.

        for col in 0..COLS {
            self.cols.enable_column(col)?;

            // check each row
            for row in 0..ROWS {
                if self.rows.get_row(row)? {
                    todo!()
                }
            }

            self.cols.disable_column(col)?;
        }

        todo!()
    }
}
