use std::thread;

use portable_pty::{CommandBuilder, PtySize, native_pty_system};

fn main() {
    let pty_system = native_pty_system();

    let mut pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();

    let cmd = CommandBuilder::new("bash");
    let child = pair.slave.spawn_command(cmd).unwrap();

    let mut reader = pair.master.try_clone_reader().unwrap();

    thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let output = String::from_utf8_lossy(&buffer[..n]);
                    print!("{output}");
                }
                Err(e) => {
                    eprintln!("Error reading from PTY: {e}");
                    break;
                }
            }
        }
    });

    writeln!(
        pair.master.take_writer().unwrap(),
        "echo 'Hello from the PTY!'"
    )
    .unwrap();

    loop {}
}
