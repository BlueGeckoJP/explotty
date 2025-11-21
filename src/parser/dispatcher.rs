use crate::parser::{
    handler_context::HandlerContext,
    handlers::{
        csi_sequence_handler::CsiSequenceHandler, dcs_sequence_handler::DcsSequenceHandler,
        osc_sequence_handler::OscSequenceHandler, sgr_sequence_handler::SgrSequenceHandler,
        vt100_sequence_handler::VT100SequenceHandler,
    },
    sequence_handler::SequenceHandler,
    sequence_token::SequenceToken,
};

pub struct SequenceDispatcher {
    csi_handler: CsiSequenceHandler,
    osc_handler: OscSequenceHandler,
    dcs_handler: DcsSequenceHandler,
    vt100_handler: VT100SequenceHandler,
    sgr_handler: SgrSequenceHandler,
}

impl SequenceDispatcher {
    pub fn new() -> Self {
        Self {
            csi_handler: CsiSequenceHandler,
            osc_handler: OscSequenceHandler,
            dcs_handler: DcsSequenceHandler,
            vt100_handler: VT100SequenceHandler,
            sgr_handler: SgrSequenceHandler,
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
            SequenceToken::VT100(seq) => {
                self.vt100_handler.handle(ctx, &seq);
            }
            SequenceToken::Sgr(seq) => {
                self.sgr_handler.handle(ctx, &seq);
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

                        // Limit the size of scrollback buffer
                        if ctx.scrollback_buffer.len() > *ctx.max_scroll_lines {
                            ctx.scrollback_buffer.remove(0);
                        }
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
            #[allow(unused)]
            _ => {
                warn!("Unhandled sequence token: {:?}", token);
            }
        }
    }
}
