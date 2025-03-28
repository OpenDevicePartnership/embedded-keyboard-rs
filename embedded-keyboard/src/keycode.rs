/// Key Events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum KeyEvent {
    /// No event to report
    NoEvent,
    /// Key change from released to pressed
    KeyDown(Coordinate),
    /// Key change from pressed to released
    KeyUp(Coordinate),
}

/// Key coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Coordinate {
    row: usize,
    col: usize,
}

impl Coordinate {
    /// Create a new `Coordinate' instance
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

/// Representation for all Keycodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u16)]
#[non_exhaustive]
pub enum KeyCode {
    NoEvent = 0x0000,
    ErrorRollOver = 0x0001,
    PostFail = 0x0002,
    ErrorUndefined = 0x0003,
    KA = 0x0004,
    KB = 0x0005,
    KC = 0x0006,
    KD = 0x0007,
    KE = 0x0008,
    KF = 0x0009,
    KG = 0x000a,
    KH = 0x000b,
    KI = 0x000c,
    KJ = 0x000d,
    KK = 0x000e,
    KL = 0x000f,
    KM = 0x0010,
    KN = 0x0011,
    KO = 0x0012,
    KP = 0x0013,
    KQ = 0x0014,
    KR = 0x0015,
    KS = 0x0016,
    KT = 0x0017,
    KU = 0x0018,
    KV = 0x0019,
    KW = 0x001a,
    KX = 0x001b,
    KY = 0x001c,
    KZ = 0x001d,
    K1 = 0x001e,
    K2 = 0x001f,
    K3 = 0x0020,
    K4 = 0x0021,
    K5 = 0x0022,
    K6 = 0x0023,
    K7 = 0x0024,
    K8 = 0x0025,
    K9 = 0x0026,
    K0 = 0x0027,
    /// Return
    KEnter = 0x0028,
    KEscape = 0x0029,
    /// Delete backward
    KBackspace = 0x002a,
    KTab = 0x002b,
    KSpaceBar = 0x002c,
    /// - and _
    KDash = 0x002d,
    /// = and +
    KEqual = 0x002e,
    /// [ and {
    KLeftBracket = 0x002f,
    /// ] and }
    KRightBracket = 0x0030,
    /// \ and |
    KBackslash = 0x0031,
    /// Non-US # and ~
    KNonUSPound = 0x0032,
    /// ; and :
    KSemiColon = 0x0033,
    /// ' and "
    KQuote = 0x0034,
    /// ` and ~,
    KGrave = 0x0035,
    /// , and <
    KComma = 0x0036,
    /// . and >
    KDot = 0x0037,
    /// / and ?
    KSlash = 0x0038,
    KCapsLock = 0x0039,
    KF1 = 0x003a,
    KF2 = 0x003b,
    KF3 = 0x003c,
    KF4 = 0x003d,
    KF5 = 0x003e,
    KF6 = 0x003f,
    KF7 = 0x0040,
    KF8 = 0x0041,
    KF9 = 0x0042,
    KF10 = 0x0043,
    KF11 = 0x0044,
    KF12 = 0x0045,
    KPrintScreen = 0x0046,
    KScrollLock = 0x0047,
    KPause = 0x0048,
    KInsert = 0x0049,
    KHome = 0x004a,
    KPageUp = 0x004b,
    /// Delete forward
    KDelete = 0x004c,
    KEnd = 0x004d,
    KPageDown = 0x004e,
    KRightArrow = 0x004f,
    KLeftArrow = 0x0050,
    KDownArrow = 0x0051,
    KUpArrow = 0x0052,
    /// Keypad Num Lock
    KpNumLock = 0x0053,
    /// Keypad /
    KpSlash = 0x0054,
    /// Keypad *
    KpAsterisk = 0x0055,
    /// Keypad -
    KpMinus = 0x0056,
    /// Keypad +
    KpPlus = 0x0057,
    /// Keypad Enter
    KpEnter = 0x0058,
    Kp1 = 0x0059,
    Kp2 = 0x005a,
    Kp3 = 0x005b,
    Kp4 = 0x005c,
    Kp5 = 0x005d,
    Kp6 = 0x005e,
    Kp7 = 0x005f,
    Kp8 = 0x0060,
    Kp9 = 0x0061,
    Kp0 = 0x0062,
    KpDot = 0x0063,
    /// Non-US \ and |
    KNonUSBackslash = 0x0064,
    KApplication = 0x0065,
    KpEqual = 0x0067,
    KF13 = 0x0068,
    KF14 = 0x0069,
    KF15 = 0x006a,
    KF16 = 0x006b,
    KF17 = 0x006c,
    KF18 = 0x006d,
    KF19 = 0x006e,
    KF20 = 0x006f,
    KF21 = 0x0070,
    KF22 = 0x0071,
    KF23 = 0x0072,
    KF24 = 0x0073,
    KExecute = 0x0074,
    KHelp = 0x0075,
    KMenu = 0x0076,
    KSelect = 0x0077,
    KStop = 0x0078,
    KAgain = 0x0079,
    KUndo = 0x007a,
    KCut = 0x007b,
    KCopy = 0x007c,
    KPaste = 0x007d,
    KFind = 0x007e,
    KMute = 0x007f,
    KVolumeUp = 0x0080,
    KVolumeDown = 0x0081,
    KLockingCapsLock = 0x0082,
    KLockingNumLock = 0x0083,
    KLockingScrollLock = 0x0084,
    KpComma = 0x0085,
    /// Keypad Equal Sign on AS/400 Keyboards
    KpEqualAS400 = 0x0086,
    KIntl1 = 0x0087,
    KIntl2 = 0x0088,
    KIntl3 = 0x0089,
    KIntl4 = 0x008a,
    KIntl5 = 0x008b,
    KIntl6 = 0x008c,
    KIntl7 = 0x008d,
    KIntl8 = 0x008e,
    KIntl9 = 0x008f,
    KLang1 = 0x0090,
    KLang2 = 0x0091,
    KLang3 = 0x0092,
    KLang4 = 0x0093,
    KLang5 = 0x0094,
    KLang6 = 0x0095,
    KLang7 = 0x0096,
    KLang8 = 0x0097,
    KLang9 = 0x0098,
    KAltErase = 0x0099,
    KSysReq = 0x009a,
    KCancel = 0x009b,
    KClear = 0x009c,
    KPrior = 0x009d,
    KReturn = 0x009e,
    KSeparator = 0x009f,
    KOut = 0x00a0,
    KOper = 0x00a1,
    KClearAgain = 0x00a2,
    KCrSel = 0x00a3,
    KExSel = 0x00a4,
    // a5 - a4: Reserved
    Kp00 = 0x00b0,
    Kp000 = 0x00b1,
    KpThousandsSeparator = 0x00b2,
    KpDecimalSeparator = 0x00b3,
    KpCurrencyUnit = 0x00b4,
    KpSubunit = 0x00b5,
    KpLeftParenthesis = 0x00b6,
    KpRightParenthesis = 0x00b7,
    KpLeftBrace = 0x00b8,
    KpRightBrace = 0x00b9,
    KpTab = 0x00ba,
    KpBackspace = 0x00bb,
    KpA = 0x00bc,
    KpB = 0x00bd,
    KpC = 0x00be,
    KpD = 0x00bf,
    KpE = 0x00c0,
    KpF = 0x00c1,
    KpXor = 0x00c2,
    KpCaret = 0x00c3,
    KpPercent = 0x00c4,
    KpLessThan = 0x00c5,
    KpGreaterThan = 0x00c6,
    KpAmpersand = 0x00c7,
    KpDoubleAmpersand = 0x00c8,
    KpVerticalPipe = 0x00c9,
    KpDoubleVerticalPipe = 0x00ca,
    KpColon = 0x00cb,
    KpPound = 0x00cc,
    KpSpace = 0x00cd,
    KpAt = 0x00ce,
    KpExclamationMark = 0x00cf,
    KpMemoryStore = 0x00d0,
    KpMemoryRecall = 0x00d1,
    KpMemoryClear = 0x00d2,
    KpMemoryAdd = 0x00d3,
    KpMemorySubtract = 0x00d4,
    KpMemoryMultiply = 0x00d5,
    KpMemoryDivide = 0x00d6,
    KpPlusMinus = 0x00d7,
    KpClear = 0x00d8,
    KpClearEntry = 0x00d9,
    KpBinary = 0x00da,
    KpOctal = 0x00db,
    KpDecimal = 0x00dc,
    KpHexadecimal = 0x00dd,
    // de - df: Reserved
    KpLeftControl = 0x00e0,
    KpLeftShift = 0x00e1,
    KpLeftAlt = 0x00e2,
    KpLeftGUI = 0x00e3,
    KpRightControl = 0x00e4,
    KpRightShift = 0x00e5,
    KpRightAlt = 0x00e6,
    KpRightGUI = 0x00e7,
    // e8 - ffff: Reserved
}
