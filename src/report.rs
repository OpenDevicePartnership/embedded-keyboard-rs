pub(crate) struct KeyReport {
    coordinate: KeyCoordinate,
    code: KeyCode,
}

pub(crate) struct KeyCoordinate {
    col: usize,
    row: usize,
}

#[repr(u16)]
pub(crate) enum KeyCode {
    No = 0,
}
