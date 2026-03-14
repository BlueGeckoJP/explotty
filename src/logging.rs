#![allow(dead_code)]
#![allow(unused_variables)]

pub fn log_input_data(data: &[u8]) {
    #[cfg(feature = "debug-logging")]
    {
        debug!("Input Data length: {} bytes", data.len());
    }
}

pub fn log_output_data(data: &[u8]) {
    #[cfg(feature = "debug-logging")]
    {
        debug!("Output Data length: {} bytes", data.len());
    }
}
