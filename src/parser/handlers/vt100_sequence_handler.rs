use crate::{
    parser::{handler_context::HandlerContext, sequence_handler::SequenceHandler},
    terminal_buffer::TerminalBuffer,
};

pub struct VT100SequenceHandler;

impl VT100SequenceHandler {
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
    fn enter_alternate_screen(ctx: &mut HandlerContext) {
        let new_buffer = TerminalBuffer::new(ctx.buffer.width, ctx.buffer.height);
        *ctx.saved_screen_buffer = Some(std::mem::replace(ctx.buffer, new_buffer));
        ctx.buffer.cursor_x = 0;
        ctx.buffer.cursor_y = 0;
    }

    /// Handle alternate screen buffer switching back
    fn leave_alternate_screen(ctx: &mut HandlerContext) {
        if let Some(saved_buffer) = ctx.saved_screen_buffer.take() {
            *ctx.buffer = saved_buffer;
        } else {
            warn!("No saved screen buffer to switch back to");
        }
        *ctx.saved_screen_buffer = None;
    }
}

impl SequenceHandler for VT100SequenceHandler {
    fn handle(&self, ctx: &mut HandlerContext, sequence: &str) {
        if let Some((params, is_set)) = self.parse_dec_private_mode(sequence) {
            for &param in &params {
                match param {
                    1 => {
                        // DECCKM - Cursor Key Application Mode
                        *ctx.decckm_mode = is_set;
                        debug!("DECCKM mode set to: {is_set}");
                    }
                    5 => {
                        // DECSCNM - Screen Reverse Video Mode
                        *ctx.reverse_video_mode = is_set;
                        if is_set {
                            warn!(
                                "DECSCNM (Screen Reverse Video) enabled but rendering not implemented"
                            );
                        }
                        debug!("DECSCNM mode set to: {is_set}");
                    }
                    6 => {
                        // DECOM - Origin Mode
                        *ctx.decom_mode = is_set;
                        if is_set {
                            warn!(
                                "DECOM (Origin Mode) enabled but margin-relative positioning not fully implemented"
                            );
                        }
                        debug!("DECOM mode set to: {is_set}");
                    }
                    7 => {
                        // DECAWM - Auto Wrap Mode
                        *ctx.decawm_mode = is_set;
                        debug!("DECAWM mode set to: {is_set}");
                    }
                    20 => {
                        // LNM - New Line Mode
                        *ctx.new_line_mode = is_set;
                        debug!("New Line Mode set to: {is_set}");
                    }
                    25 => {
                        // DECTCEM - Cursor Show/Hide
                        *ctx.show_cursor = is_set;
                        debug!("Cursor visibility set to: {is_set}");
                    }
                    1049 => {
                        // Alternate Screen Buffer
                        if is_set {
                            Self::enter_alternate_screen(ctx);
                            debug!("Entered alternate screen buffer");
                        } else {
                            Self::leave_alternate_screen(ctx);
                            debug!("Left alternate screen buffer");
                        }
                    }
                    2004 => {
                        // Bracketed Paste Mode
                        *ctx.bracket_paste_mode = is_set;
                        debug!("Bracketed paste mode set to: {is_set}");
                    }
                    _ => {
                        warn!(
                            "Unsupported DEC Private Mode parameter: ?{}{}",
                            param,
                            if is_set { 'h' } else { 'l' }
                        );
                    }
                }
            }
        }
    }
}
