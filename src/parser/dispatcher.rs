use crate::parser::{
    csi_sequence_handler::CsiSequenceHandler, dcs_sequence_handler::DcsSequenceHandler,
    handler_context::HandlerContext, osc_sequence_handler::OscSequenceHandler,
    sequence_handler::SequenceHandler, sequence_token::SequenceToken,
};

pub struct SequenceDispatcher {
    csi_handler: CsiSequenceHandler,
    osc_handler: OscSequenceHandler,
    dcs_handler: DcsSequenceHandler,
}

impl SequenceDispatcher {
    pub fn new() -> Self {
        Self {
            csi_handler: CsiSequenceHandler,
            osc_handler: OscSequenceHandler,
            dcs_handler: DcsSequenceHandler,
        }
    }

    pub fn dispatch(&self, ctx: &mut HandlerContext, token: SequenceToken) {
        match token {
            SequenceToken::Csi(seq) => {
                self.csi_handler.handle(ctx, &seq);
            }
            SequenceToken::Osc(seq) => {
                self.osc_handler.handle(ctx, &seq);
            }
            SequenceToken::Dcs(seq) => {
                self.dcs_handler.handle(ctx, &seq);
            }
            SequenceToken::Character(ch) => {
                ctx.buffer.put_char(ch);
            }
            SequenceToken::ControlChar(code) => match code {
                b'\r' => ctx.buffer.carriage_return(),
                b'\n' => {
                    if ctx.buffer.cursor_y >= ctx.buffer.height - 1 {
                        let top_line = ctx.buffer.cells[0].clone();
                        ctx.scrollback_buffer.push(top_line);
                    }
                    ctx.buffer.new_line(*ctx.new_line_mode);
                }
                b'\t' => {
                    for _ in 0..4 {
                        ctx.buffer.put_char(' ');
                    }
                }
                b'\x08' => ctx.buffer.backspace(),
                _ => {}
            },
            _ => {
                warn!("Unhandled sequence token: {:?}", token);
            }
        }
    }
}
