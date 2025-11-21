use crate::parser::sequence_token::SequenceToken;

pub struct SequenceTokenizer {
    buffer: Vec<u8>,
}

impl SequenceTokenizer {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Add incoming bytes to the tokenizer buffer, and extract complete sequences
    pub fn feed(&mut self, data: &[u8]) -> Vec<SequenceToken> {
        self.buffer.extend_from_slice(data);
        let mut tokens = Vec::new();
        let mut cursor = 0;

        while cursor < self.buffer.len() {
            match self.buffer[cursor] {
                b'\r' => {
                    tokens.push(SequenceToken::ControlChar(b'\r'));
                    cursor += 1;
                }
                b'\n' => {
                    tokens.push(SequenceToken::ControlChar(b'\n'));
                    cursor += 1;
                }
                b'\t' => {
                    tokens.push(SequenceToken::ControlChar(b'\t'));
                    cursor += 1;
                }
                b'\x08' => {
                    tokens.push(SequenceToken::ControlChar(b'\x08'));
                    cursor += 1;
                }
                b'\x1b' => {
                    // Detect escape sequences
                    if let Some((token, consumed)) =
                        self.parse_escape_sequence(&self.buffer[cursor..])
                    {
                        tokens.push(token);
                        cursor += consumed;
                    } else {
                        // Incomplete sequence -> leave in the buffer for next feed
                        break;
                    }
                }
                ch if ch < 32 || ch == 127 => {
                    // Skip other control characters
                    // ch < 32 are other control chars
                    // ch === 127 is DEL
                    cursor += 1;
                }
                _ => {
                    // Process normal character as UTF-8
                    match std::str::from_utf8(&self.buffer[cursor..]) {
                        Ok(s) => {
                            if let Some(ch) = s.chars().next() {
                                tokens.push(SequenceToken::Character(ch));
                                cursor += ch.len_utf8();
                            }
                        }
                        Err(e) => {
                            let valid_len = e.valid_up_to();
                            if valid_len > 0 {
                                /*let valid_str = unsafe {
                                    std::str::from_utf8_unchecked(
                                        &self.buffer[cursor..cursor + valid_len],
                                    )
                                };*/
                                let valid_str =
                                    std::str::from_utf8(&self.buffer[cursor..cursor + valid_len]);
                                if let Ok(valid_str) = valid_str {
                                    for ch in valid_str.chars() {
                                        tokens.push(SequenceToken::Character(ch))
                                    }
                                    cursor += valid_len;
                                } else {
                                    // Invalid UTF-8
                                    break;
                                }
                            } else {
                                // Invalid UTF-8
                                break;
                            }
                        }
                    }
                }
            }
        }

        self.buffer.drain(..cursor);
        tokens
    }

    /// Parse an escape sequence starting at the beginning of bytes
    fn parse_escape_sequence(&self, bytes: &[u8]) -> Option<(SequenceToken, usize)> {
        if bytes.len() < 2 || bytes[0] != b'\x1b' {
            return None;
        }

        match bytes[1] {
            b'[' => self
                .parse_csi(&bytes[2..])
                .map(|(s, len)| (SequenceToken::Csi(s), len + 2)),
            b']' => self
                .parse_osc(&bytes[2..])
                .map(|(s, len)| (SequenceToken::Osc(s), len + 2)),
            b'(' => self
                .parse_dcs(&bytes[2..])
                .map(|(s, len)| (SequenceToken::Dcs(s), len + 2)),
            _ => None,
        }
    }

    /// Find the end of the CSI sequence and return it
    fn parse_csi(&self, bytes: &[u8]) -> Option<(String, usize)> {
        for (i, &byte) in bytes.iter().enumerate() {
            if byte.is_ascii_lowercase() || byte.is_ascii_uppercase() {
                let sequence = String::from_utf8_lossy(&bytes[..=i]).to_string();
                return Some((sequence, i + 1));
            }
        }
        None // Incomplete sequence
    }

    /// Find the end of the OSC sequence and return it
    fn parse_osc(&self, bytes: &[u8]) -> Option<(String, usize)> {
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'\x07' {
                // BEL terminator
                let sequence = String::from_utf8_lossy(&bytes[..i]).to_string();
                return Some((sequence, i + 1));
            }
            if i + 1 < bytes.len() && bytes[i] == b'\x1b' && bytes[i + 1] == b'\\' {
                // ESC \ terminator
                let sequence = String::from_utf8_lossy(&bytes[..i]).to_string();
                return Some((sequence, i + 2));
            }
            i += 1;
        }
        None // Incomplete sequence
    }

    /// Find the end of the DCS sequence and return it
    fn parse_dcs(&self, bytes: &[u8]) -> Option<(String, usize)> {
        if bytes.len() >= 2 {
            let sequence = String::from_utf8_lossy(&bytes[..2]).to_string();
            Some((sequence, 2))
        } else {
            None // Incomplete sequence
        }
    }
}
