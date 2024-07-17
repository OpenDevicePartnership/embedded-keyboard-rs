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

    /// Get key states
    pub fn get_state(&self) -> &KeyState<ROWS, COLS> {
        &self.state
    }
}

/// The latest state of all the keys
#[derive(Debug, PartialEq, Eq)]
pub struct KeyState<const ROWS: usize, const COLS: usize> {
    state: [[i8; ROWS]; COLS],
}

impl<const ROWS: usize, const COLS: usize> KeyState<ROWS, COLS> {
    const MINIMUM: i8 = 0;
    const MAXIMUM: i8 = 3;

    fn new() -> Self {
        Self {
            state: [[0; ROWS]; COLS],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::digital::{Mock, State, Transaction};

    #[test]
    fn create_keymatrix() {
        let expectations = vec![];

        let cols = [Mock::new(&expectations), Mock::new(&expectations)];
        let rows = [Mock::new(&expectations), Mock::new(&expectations)];

        let matrix = KeyMatrix::new(cols, rows);
        let (cols, rows) = matrix.destroy();

        for mut c in cols {
            c.done();
        }

        for mut r in rows {
            r.done();
        }
    }

    #[test]
    fn scan_keymatrix_no_pressed_keys() {
        let output_expectations = vec![
            // First column
            Transaction::set(State::High),
            // Second column
            Transaction::set(State::Low),
        ];

        let input_expectations = vec![
            // First column
            Transaction::get(State::High),
            // Second column
            Transaction::get(State::High),
        ];

        let cols = [
            Mock::new(&output_expectations),
            Mock::new(&output_expectations),
        ];
        let rows = [
            Mock::new(&input_expectations),
            Mock::new(&input_expectations),
        ];

        let mut matrix = KeyMatrix::new(cols, rows);

        let result = matrix.scan_matrix();
        assert!(result.is_ok());
        let state = matrix.get_state();
        dbg!(state);

        let (cols, rows) = matrix.destroy();

        for mut c in cols {
            c.done();
        }

        for mut r in rows {
            r.done();
        }
    }

    #[test]
    fn scan_keymatrix_pressed_keys() {
        let output_expectations = vec![
            // First column
            Transaction::set(State::High),
            // Second column
            Transaction::set(State::Low),
        ];

        let input_expectations = vec![
            // First column
            Transaction::get(State::Low),
            // Second column
            Transaction::get(State::High),
        ];

        let cols = [
            Mock::new(&output_expectations),
            Mock::new(&output_expectations),
        ];
        let rows = [
            Mock::new(&input_expectations),
            Mock::new(&input_expectations),
        ];

        let mut matrix = KeyMatrix::new(cols, rows);

        let result = matrix.scan_matrix();
        assert!(result.is_ok());

        let state = matrix.get_state();
        assert_eq!(
            state,
            &KeyState {
                state: [[0, 0], [1, 1]]
            }
        );

        let (cols, rows) = matrix.destroy();

        for mut c in cols {
            c.done();
        }

        for mut r in rows {
            r.done();
        }
    }

    #[test]
    fn scan_keymatrix_pressed_keys_not_more_than_maximum() {
        let output_expectations = vec![
            // First column
            Transaction::set(State::High),
            // Second column
            Transaction::set(State::Low),
            // First column
            Transaction::set(State::High),
            // Second column
            Transaction::set(State::Low),
            // First column
            Transaction::set(State::High),
            // Second column
            Transaction::set(State::Low),
            // First column
            Transaction::set(State::High),
            // Second column
            Transaction::set(State::Low),
            // First column
            Transaction::set(State::High),
            // Second column
            Transaction::set(State::Low),
            // First column
            Transaction::set(State::High),
            // Second column
            Transaction::set(State::Low),
            // First column
            Transaction::set(State::High),
            // Second column
            Transaction::set(State::Low),
            // First column
            Transaction::set(State::High),
            // Second column
            Transaction::set(State::Low),
            // First column
            Transaction::set(State::High),
            // Second column
            Transaction::set(State::Low),
            // First column
            Transaction::set(State::High),
            // Second column
            Transaction::set(State::Low),
        ];

        let input_expectations = vec![
            // First column
            Transaction::get(State::Low),
            // Second column
            Transaction::get(State::High),
            // First column
            Transaction::get(State::Low),
            // Second column
            Transaction::get(State::High),
            // First column
            Transaction::get(State::Low),
            // Second column
            Transaction::get(State::High),
            // First column
            Transaction::get(State::Low),
            // Second column
            Transaction::get(State::High),
            // First column
            Transaction::get(State::Low),
            // Second column
            Transaction::get(State::High),
            // First column
            Transaction::get(State::Low),
            // Second column
            Transaction::get(State::High),
            // First column
            Transaction::get(State::Low),
            // Second column
            Transaction::get(State::High),
            // First column
            Transaction::get(State::Low),
            // Second column
            Transaction::get(State::High),
            // First column
            Transaction::get(State::Low),
            // Second column
            Transaction::get(State::High),
            // First column
            Transaction::get(State::Low),
            // Second column
            Transaction::get(State::High),
        ];

        let cols = [
            Mock::new(&output_expectations),
            Mock::new(&output_expectations),
        ];
        let rows = [
            Mock::new(&input_expectations),
            Mock::new(&input_expectations),
        ];

        let mut matrix = KeyMatrix::new(cols, rows);

        for _ in 0..10 {
            let result = matrix.scan_matrix();
            assert!(result.is_ok());
        }

        let state = matrix.get_state();
        assert_eq!(
            state,
            &KeyState {
                state: [[0, 0], [3, 3]]
            }
        );

        let (cols, rows) = matrix.destroy();

        for mut c in cols {
            c.done();
        }

        for mut r in rows {
            r.done();
        }
    }
}
