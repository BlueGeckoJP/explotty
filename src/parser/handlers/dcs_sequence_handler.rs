use crate::parser::sequence_handler::SequenceHandler;

pub struct DcsSequenceHandler;

impl SequenceHandler for DcsSequenceHandler {
    fn handle(&self, _ctx: &mut crate::parser::handler_context::HandlerContext, sequence: &str) {
        warn!("Unhandled DCS sequence: {}", sequence);
    }
}
