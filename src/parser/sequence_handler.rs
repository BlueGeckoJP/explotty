use crate::parser::handler_context::HandlerContext;

pub trait SequenceHandler {
    fn handle(&self, ctx: &mut HandlerContext, sequence: &str);
}
