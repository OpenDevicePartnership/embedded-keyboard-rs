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

/// Matrix of [`InputPin`]s and [`OutputPin`]s describing a keyboard
pub struct KeyMatrix<const ROWS: usize, const COLS: usize, I: InputPin, O: OutputPin> {
    rows: KeyRows<ROWS, I>,
    cols: KeyColumns<COLS, O>,
    state: KeyState<ROWS, COLS>,
}

impl<const ROWS: usize, const COLS: usize, I: InputPin, O: OutputPin> KeyMatrix<ROWS, COLS, I, O> {
    /// Instantiate a new matrix with the given rows and columns
    pub fn new(cols: [O; COLS], rows: [I; ROWS]) -> Self {
        Self {
            cols: KeyColumns::new(cols),
            rows: KeyRows::new(rows),
            state: KeyState::new(),
        }
    }

    /// Destroys this instance and returns cols and rows arrays back to the caller.
    pub fn destroy(self) -> ([O; COLS], [I; ROWS]) {
        (self.cols.pins, self.rows.pins)
    }
}

impl<const ROWS: usize, const COLS: usize, I: InputPin, O: OutputPin> KeyMatrix<ROWS, COLS, I, O> {
    /// Scan the current state of the key matrix.
    pub fn scan_matrix(&mut self) -> Result<()> {
        // iterate over columns, enabling each along the way, then check the
        // state of each row by mapping each row to its current state.

        for col in 0..COLS {
            self.cols.enable_column(col)?;

            // check each row
            for row in 0..ROWS {
                let mut current = self.state.state[col][row];

                if self.rows.get_row(row)? {
                    current += 1;
                } else {
                    current -= 1;
                }

                self.state.state[col][row] = current.clamp(
                    KeyState::<ROWS, COLS>::MINIMUM,
                    KeyState::<ROWS, COLS>::MAXIMUM,
                );
            }

            self.cols.disable_column(col)?;
        }

        Ok(())
    }
}

/// The latest state of all the keys
struct KeyState<const ROWS: usize, const COLS: usize> {
    state: [[u8; ROWS]; COLS],
}

impl<const ROWS: usize, const COLS: usize> KeyState<ROWS, COLS> {
    const MINIMUM: u8 = 0;
    const MAXIMUM: u8 = 3;

    fn new() -> Self {
        Self {
            state: [[0; ROWS]; COLS],
        }
    }
}
