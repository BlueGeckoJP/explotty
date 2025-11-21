use crate::parser::{handler_context::HandlerContext, sequence_handler::SequenceHandler};

pub struct CsiSequenceHandler;

impl SequenceHandler for CsiSequenceHandler {
    fn handle(&self, ctx: &mut HandlerContext, sequence: &str) {}
}
