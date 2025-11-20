#[derive(Debug, Clone)]
pub enum SequenceToken {
    Csi(String),     // ESC [
    Osc(String),     // ESC ]
    Dcs(String),     // ESC (
    VT100(String),   // Other VT100 sequences
    Sgr(String),     // SGR sequences
    Character(char), // Normal character
    ControlChar(u8), // CR, LF, TAB, BS, etc.
}
