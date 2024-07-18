//! This crate provides an abstraction over GPIO matrices for the purposes of
//! creating keyboards and keypads. It is based on [`embedded-hal`]
//! traits. Specifically `Input` and `Output` for managing Rows and Columns of
//! the keyboard matrix.

#![doc(html_root_url = "https://docs.rs/gpio-keyboard/latest")]
#![cfg_attr(not(test), no_std)]

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_keyboard::{Error, ErrorKind, ErrorType, Keyboard, Keycode};

/// Result type alias
pub type Result<T> = core::result::Result<T, KeyboardError>;

/// Errors produced by this crate
#[derive(Debug, PartialEq, Eq)]
pub enum KeyboardError {
    /// Unable to set pin high
    SetColumnHigh,

    /// Unable to set pin low
    SetColumnLow,

    /// Unable to read row state
    GetRow,

    /// Some other error occurred.
    Other,
}

impl Error for KeyboardError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

/// Matrix of [`InputPin`]s and [`OutputPin`]s describing a keyboard
pub struct KeyMatrix<
    const ROWS: usize,
    const COLS: usize,
    const NKRO: usize,
    I: InputPin,
    O: OutputPin,
> {
    rows: [I; ROWS],
    cols: [O; COLS],
    keys: [[Key; ROWS]; COLS],
    report: [Keycode; NKRO],
}

impl<const ROWS: usize, const COLS: usize, const NKRO: usize, I: InputPin, O: OutputPin>
    KeyMatrix<ROWS, COLS, NKRO, I, O>
{
    /// Instantiate a new matrix with the given rows and columns
    pub fn new(cols: [O; COLS], rows: [I; ROWS]) -> Self {
        Self {
            cols,
            rows,
            keys: [[Key::new(); ROWS]; COLS],
            report: [Keycode::NoEvent; NKRO],
        }
    }

    /// Destroys this instance and returns cols and rows arrays back to the caller.
    pub fn destroy(self) -> ([O; COLS], [I; ROWS]) {
        (self.cols, self.rows)
    }
}

impl<const ROWS: usize, const COLS: usize, const NKRO: usize, I: InputPin, O: OutputPin> ErrorType
    for KeyMatrix<ROWS, COLS, NKRO, I, O>
{
    type Error = KeyboardError;
}

impl<const ROWS: usize, const COLS: usize, const NKRO: usize, I: InputPin, O: OutputPin> Keyboard
    for KeyMatrix<ROWS, COLS, NKRO, I, O>
{
    /// Scan the current state of the key matrix.
    fn scan(&mut self) -> Result<&[Keycode]> {
        // iterate over columns, enabling each along the way, then check the
        // state of each row by mapping each row to its current state.

        for (x, col) in self.cols.iter_mut().enumerate() {
            col.set_high().map_err(|_| KeyboardError::SetColumnHigh)?;

            // check each row
            for (y, row) in self.rows.iter_mut().enumerate() {
                let key = self.keys.get_mut(x).unwrap().get_mut(y).unwrap();
                let state = row.is_high().map_err(|_| KeyboardError::GetRow)?;
                key.update(state);
            }

            col.set_low().map_err(|_| KeyboardError::SetColumnLow)?;
        }

        Ok(&self.report[..])
    }
}

/// The latest state of all the keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    state: i8,
    output: bool,
}

impl Default for Key {
    fn default() -> Self {
        Self {
            state: 0,
            output: false,
        }
    }
}

impl Key {
    const MINIMUM: i8 = 0;
    const MAXIMUM: i8 = 3;

    fn new() -> Self {
        Self::default()
    }

    fn update(&mut self, sample: bool) -> bool {
        let mut current = self.state;
        current += if sample { 1 } else { -1 };
        self.state = current.clamp(Key::MINIMUM, Key::MAXIMUM);

        self.output = if self.state == Key::MINIMUM {
            false
        } else if self.state == Key::MAXIMUM {
            true
        } else {
            self.output
        };

        self.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::{
        digital::{Mock, State, Transaction},
        MockError,
    };
    use itertools::izip;
    use std::io::ErrorKind;

    #[test]
    fn key_creation() {
        let key = Key::default();
        assert_eq!(
            key,
            Key {
                state: 0,
                output: false
            }
        );
    }

    #[test]
    fn update_state_once() {
        let mut key = Key::default();
        key.update(false);
        assert_eq!(
            key,
            Key {
                state: 0,
                output: false
            }
        );

        let mut key = Key::default();
        key.update(true);
        assert_eq!(
            key,
            Key {
                state: 1,
                output: false
            }
        );
    }

    #[test]
    fn state_never_goes_over_maximum() {
        let mut key = Key::default();

        for _ in 0..10 {
            key.update(true);
        }

        assert_eq!(
            key,
            Key {
                state: Key::MAXIMUM,
                output: true
            }
        );
    }

    #[test]
    fn state_filters_through_integrator() {
        let mut key = Key::default();
        let input = [
            false, false, false, true, true, false, true, false, false, true, true, false, true,
            true, true, false, true, true, false, false, true, true, true, false, true, true, true,
            true, true, false, false, false, true, false, true, false, false, true, true, false,
            false, false, true, false, true, true, false, true, true, false, false, false, false,
            true, true, false, false, false,
        ];
        let state = [
            0, 0, 0, 1, 2, 1, 2, 1, 0, 1, 2, 1, 2, 3, 3, 2, 3, 3, 2, 1, 2, 3, 3, 2, 3, 3, 3, 3, 3,
            2, 1, 0, 1, 0, 1, 0, 0, 1, 2, 1, 0, 0, 1, 0, 1, 2, 1, 2, 3, 2, 1, 0, 0, 1, 2, 1, 0, 0,
        ];
        let output = [
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, true, true, true, true, true, true, true, true, true, true, true, true, true,
            true, true, true, true, true, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false, false, false, true, true, true, false,
            false, false, false, false, false, false,
        ];

        for (i, s, o) in izip!(input.iter(), state.iter(), output.iter()) {
            key.update(*i);
            assert_eq!(
                key,
                Key {
                    state: *s,
                    output: *o,
                }
            )
        }
    }

    #[test]
    fn create_keymatrix() {
        let expectations = vec![];

        let cols = [Mock::new(&expectations), Mock::new(&expectations)];
        let rows = [Mock::new(&expectations), Mock::new(&expectations)];

        let matrix: KeyMatrix<2, 2, 6, _, _> = KeyMatrix::new(cols, rows);
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

        let mut matrix: KeyMatrix<2, 2, 6, _, _> = KeyMatrix::new(cols, rows);

        let result = matrix.scan();
        assert!(result.is_ok());

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

        let mut matrix: KeyMatrix<2, 2, 6, _, _> = KeyMatrix::new(cols, rows);

        let result = matrix.scan();
        assert!(result.is_ok());

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

        let mut matrix: KeyMatrix<2, 2, 6, _, _> = KeyMatrix::new(cols, rows);

        for _ in 0..10 {
            let result = matrix.scan();
            assert!(result.is_ok());
        }

        let (cols, rows) = matrix.destroy();

        for mut c in cols {
            c.done();
        }

        for mut r in rows {
            r.done();
        }
    }

    #[test]
    fn error_enabling_column() {
        let err = MockError::Io(ErrorKind::NotConnected);
        let expectations = vec![Transaction::set(State::High).with_error(err)];

        let cols = [Mock::new(&expectations), Mock::new(&vec![])];
        let rows = [Mock::new(&vec![]), Mock::new(&vec![])];

        let mut matrix: KeyMatrix<2, 2, 6, _, _> = KeyMatrix::new(cols, rows);
        let result = matrix.scan();
        assert!(result.is_err());
        assert_eq!(result, Err(KeyboardError::SetColumnHigh));

        let (cols, rows) = matrix.destroy();

        for mut c in cols {
            c.done();
        }

        for mut r in rows {
            r.done();
        }
    }

    #[test]
    fn error_reading_row() {
        let err = MockError::Io(ErrorKind::NotConnected);

        let output_expectations = vec![
            // First column
            Transaction::set(State::High),
        ];

        let input_expectations = vec![
            // First column
            Transaction::get(State::Low).with_error(err),
        ];

        let cols = [Mock::new(&output_expectations), Mock::new(&vec![])];
        let rows = [Mock::new(&input_expectations), Mock::new(&vec![])];

        let mut matrix: KeyMatrix<2, 2, 6, _, _> = KeyMatrix::new(cols, rows);
        let result = matrix.scan();
        assert!(result.is_err());
        assert_eq!(result, Err(KeyboardError::GetRow));

        let (cols, rows) = matrix.destroy();

        for mut c in cols {
            c.done();
        }

        for mut r in rows {
            r.done();
        }
    }

    #[test]
    fn error_disabling_column() {
        let err = MockError::Io(ErrorKind::NotConnected);

        let output_expectations = vec![
            // First column
            Transaction::set(State::High),
            // Second column
            Transaction::set(State::Low).with_error(err),
        ];

        let input_expectations = vec![
            // First column
            Transaction::get(State::High),
        ];

        let cols = [Mock::new(&output_expectations), Mock::new(&vec![])];
        let rows = [
            Mock::new(&input_expectations),
            Mock::new(&input_expectations),
        ];

        let mut matrix: KeyMatrix<2, 2, 6, _, _> = KeyMatrix::new(cols, rows);
        let result = matrix.scan();
        assert!(result.is_err());
        assert_eq!(result, Err(KeyboardError::SetColumnLow));

        let (cols, rows) = matrix.destroy();

        for mut c in cols {
            c.done();
        }

        for mut r in rows {
            r.done();
        }
    }
}
