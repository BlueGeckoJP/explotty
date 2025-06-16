use std::{sync::Arc, thread, time::Duration};

use eframe::egui::{self, mutex::Mutex};
use portable_pty::{Child, CommandBuilder, PtyPair, PtySize, native_pty_system};

use crate::terminal_widget::TerminalWidget;

pub struct App {
    pub terminal_widget: TerminalWidget,
    pub pty_pair: Option<PtyPair>,
    pub child: Option<Box<dyn Child + Send + Sync>>,
    output_buffer: Arc<Mutex<Vec<u8>>>,
    input_buffer: Arc<Mutex<Vec<u8>>>,
    is_running: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            terminal_widget: TerminalWidget::new(80, 24),
            pty_pair: None,
            child: None,
            is_running: false,
            output_buffer: Arc::new(Mutex::new(Vec::new())),
            input_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        app.start_pty();
        app
    }

    fn start_pty(&mut self) {
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("Failed to create PTY");

        // Spawn a shell in the PTY
        let cmd = CommandBuilder::new("bash");
        let child = pty_pair
            .slave
            .spawn_command(cmd)
            .expect("Failed to spawn shell");

        self.pty_pair = Some(pty_pair);
        self.child = Some(child);
        self.is_running = true;

        // Initialize output thread
        let output_buffer = self.output_buffer.clone();
        if let Some(ref pty_pair) = self.pty_pair {
            let mut reader = pty_pair
                .master
                .try_clone_reader()
                .expect("Failed to clone reader");

            thread::spawn(move || {
                let mut buffer = [0u8; 4096];
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            let mut output = output_buffer.lock();
                            output.extend_from_slice(&buffer[..n]);
                        }
                        Err(e) => {
                            eprintln!("Error reading from PTY: {e}");
                            break;
                        }
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            });
        }

        // Initialize input thread
        let input_buffer = self.input_buffer.clone();
        if let Some(ref pty_pair) = self.pty_pair {
            let mut writer = pty_pair
                .master
                .take_writer()
                .expect("Failed to take writer");

            thread::spawn(move || {
                loop {
                    let data_to_write = {
                        let mut input = input_buffer.lock();
                        if input.is_empty() {
                            None
                        } else {
                            let data = input.clone();
                            input.clear();
                            Some(data)
                        }
                    };

                    if let Some(data) = data_to_write
                        && let Err(e) = writer.write_all(&data)
                    {
                        eprintln!("Error writing to PTY: {e}");
                        break;
                    }

                    thread::sleep(Duration::from_millis(10));
                }
            });
        }
    }

    fn handle_pty_output(&mut self) {
        let data = {
            let mut output = self.output_buffer.lock();
            if output.is_empty() {
                return;
            }
            let data = output.clone();
            output.clear();
            data
        };

        self.terminal_widget.process_output(&data);
    }

    fn send_input_to_pty(&mut self, data: Vec<u8>) {
        if !data.is_empty() {
            let mut input = self.input_buffer.lock();
            input.extend_from_slice(&data);
        }
    }

    fn resize_pty(&mut self, cols: u16, rows: u16) {
        if let Some(ref pty_pair) = self.pty_pair {
            let new_size = PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            };
            if let Err(e) = pty_pair.master.resize(new_size) {
                eprintln!("Failed to resize PTY: {e}");
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Start the PTY processing
        self.handle_pty_output();

        // Repainting requests for continuous updating | ~60fps
        ctx.request_repaint_after(Duration::from_millis(16));

        egui::CentralPanel::default().show(ctx, |ui| {
            let response = self.terminal_widget.show(ui);

            let cols = self.terminal_widget.buffer.width as u16;
            let rows = self.terminal_widget.buffer.height as u16;

            static mut LAST_SIZE: (u16, u16) = (0, 0);
            unsafe {
                if LAST_SIZE != (cols, rows) {
                    self.resize_pty(cols, rows);
                    LAST_SIZE = (cols, rows);
                }
            }

            // If it has focus, handle input
            if response.has_focus() || ui.memory(|mem| mem.has_focus(response.id)) {
                let input_data = self.terminal_widget.handle_input(ctx);
                self.send_input_to_pty(input_data);
            }

            // Click to start the PTY
            if response.clicked() {
                ui.memory_mut(|mem| mem.request_focus(response.id));
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
