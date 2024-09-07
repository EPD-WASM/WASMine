use super::value::Value;
// https://webassembly.github.io/spec/core/exec/runtime.html#results

#[derive(Debug, Clone)]
pub enum WResult {
    Values(Vec<Value>),
    Trap,
}
