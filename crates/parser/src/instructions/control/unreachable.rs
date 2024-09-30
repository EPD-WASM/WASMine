use super::*;

pub(crate) fn r#unreachable(_: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> ParseResult {
    o.write_unreachable();
    Ok(())
}
