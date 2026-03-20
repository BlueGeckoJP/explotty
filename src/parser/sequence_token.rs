#[derive(Debug, Clone)]
pub enum SequenceToken {
    Csi(String),     // ESC [
    Osc(String),     // ESC ]
    Dcs(String),     // ESC (
    VT100(String),   // Other VT100 sequences (CSI starting with ?)
    Sgr(String),     // SGR sequences
    Esc(String),     // ESC followed by char (e.g. ESC 7, ESC c)
    Character(char), // Normal character
    ControlChar(u8), // CR, LF, TAB, BS, etc.
}
