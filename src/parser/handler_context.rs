use eframe::egui;

use crate::{terminal_buffer::TerminalBuffer, terminal_cell::TerminalCell};

pub struct HandlerContext<'a> {
    pub buffer: &'a mut TerminalBuffer,
    pub scrollback_buffer: &'a mut Vec<Vec<TerminalCell>>,
    pub saved_screen_buffer: &'a mut Option<TerminalBuffer>,

    // DEC private mode flags
    pub decckm_mode: &'a mut bool,
    pub decom_mode: &'a mut bool,
    pub decawm_mode: &'a mut bool,
    pub reverse_video_mode: &'a mut bool,
    pub show_cursor: &'a mut bool,
    pub bracket_paste_mode: &'a mut bool,
    pub new_line_mode: &'a mut bool,

    // Other
    pub ctx: &'a egui::Context,
}
