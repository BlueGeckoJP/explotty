#![allow(dead_code)]
#![allow(unused_variables)]

const OUTPUT_LOG_FILE: &str = "output_log.txt";
const INPUT_LOG_FILE: &str = "input_log.txt";

pub fn log_input_data(data: &[u8]) {
    #[cfg(feature = "debug-logging")]
    {
        let sanitized: String = data
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b.is_ascii_whitespace() {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(INPUT_LOG_FILE);
        if let Ok(ref mut f) = file {
            use std::io::Write;
            let _ = writeln!(f, "{}", sanitized);
        }
    }
}

pub fn log_output_data(data: &[u8]) {
    #[cfg(feature = "debug-logging")]
    {
        let sanitized: String = data
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b.is_ascii_whitespace() {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();
        debug!("Output Data: {}", sanitized);

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(OUTPUT_LOG_FILE);
        if let Ok(ref mut f) = file {
            use std::io::Write;
            let _ = writeln!(f, "{}", sanitized);
        }
    }
}
