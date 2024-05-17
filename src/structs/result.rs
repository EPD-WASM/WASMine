use super::value::Value;
// https://webassembly.github.io/spec/core/exec/runtime.html#results

#[derive(Debug, Clone)]
pub(crate) enum WResult {
    Values(Vec<Value>),
    Trap,
}
