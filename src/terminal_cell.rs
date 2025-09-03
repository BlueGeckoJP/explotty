use eframe::egui::Color32;

#[derive(Clone, Debug)]
pub struct TerminalCell {
    pub character: char,
    pub fg_color: Color32,
    pub bg_color: Color32,
    pub bold: bool,
    pub underline: bool,
    pub italic: bool,
    pub blink: bool,
    pub strikethrough: bool,
    pub hidden: bool,
    pub wide_tail: bool,
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            character: ' ',
            fg_color: Color32::WHITE,
            bg_color: Color32::TRANSPARENT,
            bold: false,
            underline: false,
            italic: false,
            blink: false,
            strikethrough: false,
            hidden: false,
            wide_tail: false,
        }
    }
}
