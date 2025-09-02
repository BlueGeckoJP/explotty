// VT100/DEC Private Mode Parser
//
// This module handles VT100 and DEC Private Mode escape sequences, which have the format:
// CSI ? Pn h/l  (where CSI is typically "\x1b[")
//
// Supported DEC Private Mode Parameters:
// ┌────────┬─────────────────────────────────────┬─────────────────────────────────────┐
// │ Param  │ Name                                │ Description                         │ 
// ├────────┼─────────────────────────────────────┼─────────────────────────────────────┤
// │ ?1h/l  │ DECCKM (Cursor Key Application)     │ Application/Normal cursor key mode  │
// │ ?5h/l  │ DECSCNM (Screen Reverse Video)      │ Reverse/Normal video mode           │
// │ ?6h/l  │ DECOM (Origin Mode)                 │ Relative/Absolute cursor addressing │
// │ ?7h/l  │ DECAWM (Auto Wrap Mode)             │ Enable/Disable automatic line wrap │
// │ ?20h/l │ LNM (New Line Mode)                 │ New line/Line feed mode             │
// │ ?25h/l │ DECTCEM (Text Cursor Enable)        │ Show/Hide cursor                    │
// │ ?1049h/l│ Alternate Screen Buffer            │ Switch to/from alternate screen     │
// │ ?2004h/l│ Bracketed Paste Mode               │ Enable/Disable bracketed paste     │
// └────────┴─────────────────────────────────────┴─────────────────────────────────────┘
//
// The 'h' suffix sets (enables) the mode, 'l' suffix resets (disables) the mode.
// Multiple parameters can be specified with semicolon separation: ?1;25h
//
// References:
// - https://invisible-island.net/xterm/ctlseqs/ctlseqs.html
// - https://vt100.net/docs/vt100-ug/chapter3.html
// - https://espterm.github.io/docs/VT100%20escape%20codes.html

use crate::terminal_widget::TerminalWidget;
use crate::terminal_buffer::TerminalBuffer;

impl TerminalWidget {
    /// Parse DEC Private Mode sequences (CSI ? Pn h/l format)
    /// Returns (parameter_numbers, is_set_mode) if valid, None otherwise
    fn parse_dec_private_mode(&self, sequence: &str) -> Option<(Vec<u16>, bool)> {
        // DEC Private Mode sequences have format: ? Pn h/l or ? Pn ; Pm h/l
        if !sequence.starts_with('?') {
            return None;
        }
        
        let (params_str, is_set) = if sequence.ends_with('h') {
            (sequence.strip_prefix('?')?.strip_suffix('h')?, true)
        } else if sequence.ends_with('l') {
            (sequence.strip_prefix('?')?.strip_suffix('l')?, false)
        } else {
            return None;
        };
        
        // Parse parameter numbers (can be semicolon-separated)
        let mut params = Vec::new();
        for param_str in params_str.split(';') {
            if let Ok(param) = param_str.trim().parse::<u16>() {
                params.push(param);
            } else {
                // Invalid parameter format
                return None;
            }
        }
        
        if params.is_empty() {
            None
        } else {
            Some((params, is_set))
        }
    }

    /// Handle alternate screen buffer switching
    fn enter_alternate_screen(&mut self) {
        let new_buffer = TerminalBuffer::new(self.buffer.width, self.buffer.height);
        self.saved_screen_buffer = Some(std::mem::replace(&mut self.buffer, new_buffer));
        self.buffer.cursor_x = 0;
        self.buffer.cursor_y = 0;
    }

    /// Handle alternate screen buffer switching back
    fn leave_alternate_screen(&mut self) {
        if let Some(saved_buffer) = self.saved_screen_buffer.take() {
            self.buffer = saved_buffer;
        } else {
            warn!("No saved screen buffer to switch back to");
        }
        self.saved_screen_buffer = None;
    }

    /// Process VT100/DEC Private Mode sequences
    /// Extended implementation supporting all major DEC Private Mode sequences
    /// 
    /// Supported sequences:
    /// - ?1h/l   (DECCKM: Cursor Key Application Mode)
    /// - ?5h/l   (DECSCNM: Screen Reverse Video)
    /// - ?6h/l   (DECOM: Origin Mode)
    /// - ?7h/l   (DECAWM: Auto Wrap Mode)
    /// - ?20h/l  (New Line Mode)
    /// - ?25h/l  (DECTCEM: Cursor Show/Hide)
    /// - ?1049h/l (Alternate Screen Buffer)
    /// - ?2004h/l (Bracketed Paste Mode)
    pub fn process_vt100(&mut self, sequence: &str) -> bool {
        if let Some((params, is_set)) = self.parse_dec_private_mode(sequence) {
            for &param in &params {
                match param {
                    1 => {
                        // DECCKM - Cursor Key Application Mode
                        self.decckm_mode = is_set;
                        debug!("DECCKM mode set to: {}", is_set);
                    }
                    5 => {
                        // DECSCNM - Screen Reverse Video Mode
                        self.reverse_video_mode = is_set;
                        if is_set {
                            warn!("DECSCNM (Screen Reverse Video) enabled but rendering not implemented");
                        }
                        debug!("DECSCNM mode set to: {}", is_set);
                    }
                    6 => {
                        // DECOM - Origin Mode
                        self.decom_mode = is_set;
                        if is_set {
                            warn!("DECOM (Origin Mode) enabled but margin-relative positioning not fully implemented");
                        }
                        debug!("DECOM mode set to: {}", is_set);
                    }
                    7 => {
                        // DECAWM - Auto Wrap Mode
                        self.decawm_mode = is_set;
                        debug!("DECAWM mode set to: {}", is_set);
                    }
                    20 => {
                        // LNM - New Line Mode
                        self.new_line_mode = is_set;
                        debug!("New Line Mode set to: {}", is_set);
                    }
                    25 => {
                        // DECTCEM - Cursor Show/Hide
                        self.show_cursor = is_set;
                        debug!("Cursor visibility set to: {}", is_set);
                    }
                    1049 => {
                        // Alternate Screen Buffer
                        if is_set {
                            self.enter_alternate_screen();
                            debug!("Entered alternate screen buffer");
                        } else {
                            self.leave_alternate_screen();
                            debug!("Left alternate screen buffer");
                        }
                    }
                    2004 => {
                        // Bracketed Paste Mode
                        self.bracket_paste_mode = is_set;
                        debug!("Bracketed paste mode set to: {}", is_set);
                    }
                    _ => {
                        warn!("Unsupported DEC Private Mode parameter: ?{}{}", param, if is_set { 'h' } else { 'l' });
                        return false;
                    }
                }
            }
            true
        } else {
            // Not a valid DEC Private Mode sequence
            false
        }
    }
}
