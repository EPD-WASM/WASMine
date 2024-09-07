use super::*;

pub(crate) fn r#unreachable(_: &mut C, _: &mut I, o: &mut O) -> ParseResult {
    o.write(Unreachable {});
    Ok(())
}
