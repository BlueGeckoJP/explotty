use crate::terminal_widget::TerminalWidget;

impl TerminalWidget {
    pub fn process_vt100(&mut self, sequence: &str) -> bool {
        // https://espterm.github.io/docs/VT100%20escape%20codes.html
        match sequence {
            // setnl LMN / Set new line mode
            ch if ch.ends_with("20h") => {
                self.new_line_mode = true;
                true
            }
            // setlf LMN / Set line feed mode
            ch if ch.ends_with("20l") => {
                self.new_line_mode = false;
                true
            }
            // setappl DECCKM / Set cursor key to application
            ch if ch.ends_with('h') && sequence.contains("1") => {
                self.decckm_mode = true;
                true
            }
            // setcursor DECCKM / Set cursor key to cursor
            ch if ch.ends_with('l') && sequence.contains("1") => {
                self.decckm_mode = false;
                true
            }
            _ => false,
        }
    }
}
