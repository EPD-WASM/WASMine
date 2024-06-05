use ir::basic_block::BasicBlockID;

use super::*;

// we trust the caller to verify that all input variables are of the same type
pub(crate) fn new_phinode(
    inputs: Vec<(BasicBlockID, VariableID)>,
    var_type: ValType,
    ctxt: &mut Context,
) -> PhiNode {
    let out = ctxt.create_var(var_type);
    let res = PhiNode {
        inputs,
        out: out.id,
        r#type: var_type,
    };
    ctxt.push_var(out);
    res
}
