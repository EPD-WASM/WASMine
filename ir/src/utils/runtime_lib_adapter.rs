use crate::structs::element::ElemMode;
use crate::structs::module::Module;
use runtime_lib::{entrypoint::config::*, WASM_PAGE_LIMIT};
use wasm_types::RefType;

impl From<&Module> for RTConfig {
    fn from(module: &Module) -> Self {
        let mut config = RTConfig::default();
        for table in module.tables.iter() {
            config.tables.push(RTTable {
                min_size: table.r#type.lim.min,
                max_size: table.r#type.lim.max.unwrap_or(WASM_PAGE_LIMIT),
                r#type: match table.r#type.ref_type {
                    RefType::FunctionReference => RTTableType::FuncRef,
                    RefType::ExternReference => RTTableType::ExternRef,
                },
            })
        }
        for element in module.elements.iter() {
            config.elements.push(RTElement {
                initializers: todo!(),
                mode: match element.mode {
                    ElemMode::Active { table, offset } => RTElemMode::PreLoad {
                        table_idx: table,
                        offset: todo!(),
                    },
                    ElemMode::Passive => RTElemMode::RuntimeLoad,
                    ElemMode::Declarative => RTElemMode::Placeholder,
                },
            })
        }
        config
    }
}
