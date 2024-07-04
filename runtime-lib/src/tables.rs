use crate::{linker::RTImport, module_instance::InstantiationError, Cluster, InstanceHandle};
use ir::structs::table::Table;
use runtime_interface::RawFunctionPtr;

#[derive(Clone, Copy)]
pub(crate) enum TableItem {
    // store raw function addresses
    Func(RawFunctionPtr),
    Extern(RawFunctionPtr),
    Null,
}

// All other information like the table type, etc. are stored in the config
#[derive(Default, Clone)]
pub struct TableInstance {
    // final, evaluated references
    pub(crate) values: Vec<TableItem>,
}

impl InstanceHandle<'_> {
    pub(crate) fn init_tables_on_cluster<'a>(
        cluster: &'a Cluster,
        tables_meta: &[Table],
        imports: &[RTImport],
    ) -> Result<&'a mut [TableInstance], InstantiationError> {
        Ok(cluster.alloc_tables(Vec::new()))
    }
}
