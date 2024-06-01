use crate::{error::RuntimeError, helpers::trap, runtime::Runtime};
use wasm_types::TypeIdx;

impl Runtime {
    /// Resolves indirect call via table
    #[no_mangle]
    pub extern "C" fn indirect_call(
        &self,
        table_idx: usize,
        type_idx: usize,
        entry_idx: usize,
    ) -> TypeIdx {
        if table_idx >= self.tables.len() {
            trap()
        }
        let table = &self.tables[table_idx];

        if type_idx >= self.config.module.function_types.len() {
            trap()
        }
        let func_type = &self.config.module.function_types[type_idx];

        if entry_idx >= table.values.len() {
            trap()
        }

        let func_idx = match table.values[entry_idx] {
            TableItem::FuncIdx(func_idx) => func_idx,
            TableItem::Null => trap(),
            TableItem::ExternIdx(_) => unimplemented!(),
        };
        let func_ref = &self.config.module.ir.functions[func_idx as usize];
        let func_ref_type = &self.config.module.function_types[func_ref.type_idx as usize];
        if func_ref_type != func_type {
            trap()
        }
        func_idx
    }
}

// this type information is also stored in the config. We could therefore omit this whole enum...
#[derive(Clone, Copy)]
pub(crate) enum TableItem {
    FuncIdx(u32),
    ExternIdx(u32),
    Null,
}

// All other information like the table type, etc. are stored in the config
#[derive(Default)]
pub(crate) struct TableInstance {
    // final, evaluated references
    pub(crate) values: Vec<TableItem>,
}

impl Runtime {
    fn init_tables(&mut self) -> Result<(), RuntimeError> {
        for table in self.config.tables.iter() {
            self.tables.push(TableInstance {
                values: vec![TableItem::Null; table.min_size as usize],
            });
        }

        // for elem in self.config.module.elements.iter_mut() {
        //     match &elem.mode {
        //         ElemMode::Active { table, offset } => {
        //             let numeric_offset: u32 = match offset.to_owned().try_into() {
        //                 Ok(val) => val,
        //                 _ => return Err(RuntimeError::NotImplemented("non numeric offset".into())),
        //             };
        //             let num_elem_items = elem.init.len();
        //             let table_instance = self.tables.get_mut(*table_idx as usize).unwrap();

        //             let min_required_table_size = evaluated_offset as usize + num_elem_items;
        //             if min_required_table_size > table_instance.values.len() {
        //                 if min_required_table_size
        //                     > self.config.tables[*table_idx as usize].max_size as usize
        //                 {
        //                     return Err(RuntimeError::TableAccessOutOfBounds);
        //                 }
        //                 table_instance
        //                     .values
        //                     .resize(min_required_table_size, TableItem::Null);
        //             }

        //             for (idx, initializer) in elem.initializers.drain(..).enumerate() {
        //                 if self.config.tables[*table_idx as usize].r#type == RTTableType::FuncRef {
        //                     table_instance.values[evaluated_offset as usize + idx] =
        //                         TableItem::ExternIdx(initializer());
        //                 } else {
        //                     table_instance.values[evaluated_offset as usize + idx] =
        //                         TableItem::ExternIdx(initializer());
        //                 }
        //             }
        //             // remove this element, it is not needed anymore
        //             elem.mode = RTElemMode::Placeholder
        //         }
        //         _ => (),
        //     }
        // }
        Ok(())
    }
}

// impl Tablelike for Table {
//     fn get(&self, offset: u32) -> Option<Reference> {
//         let i = offset as usize;
//         match &self.data {
//             TableData::FunctionReference(fn_refs) => {
//                 fn_refs.get(i).cloned().map(Reference::Function)
//             }
//             TableData::ExternReference(ext_refs) => ext_refs.get(i).cloned().map(Reference::Extern),
//         }
//     }

//     fn set(&mut self, offset: u32, data: Reference) -> Result<(), TableError> {
//         if offset as usize >= self.data.len() {
//             return Err(TableError::OutOfRangeError {
//                 index: offset as usize,
//                 limits: self.limits.clone(),
//             });
//         }

//         match (&mut self.data, data) {
//             (TableData::FunctionReference(ref mut v), Reference::Function(r)) => {
//                 v[offset as usize] = r;
//             }
//             (TableData::ExternReference(ref mut v), Reference::Extern(r)) => {
//                 v[offset as usize] = r;
//             }
//             _ => return Err(TableError::TypeMismatchError),
//         };

//         Ok(())
//     }

//     fn size(&self) -> TableSize {
//         self.data.len() as TableSize
//     }

//     fn grow(&mut self) -> Result<TableSize, TableError> {
//         todo!()
//     }

//     fn fill(&mut self, range: Range<usize>, data: Reference) -> Result<(), TableError> {
//         if range.end > self.data.len() {
//             return Err(TableError::OutOfRangeError {
//                 index: range.end,
//                 limits: self.limits.clone(),
//             });
//         }

//         match (&mut self.data, data) {
//             (TableData::FunctionReference(ref mut v), Reference::Function(r)) => {
//                 v[range].iter_mut().for_each(|x| *x = r.clone());
//             }
//             (TableData::ExternReference(ref mut v), Reference::Extern(r)) => {
//                 v[range].iter_mut().for_each(|x| *x = r.clone());
//             }
//             _ => return Err(TableError::TypeMismatchError),
//         };

//         Ok(())
//     }

//     fn copy_within(&mut self, src: Range<usize>, dst: u32) -> Result<(), TableError> {
//         if src.end > self.data.len() || dst as usize + src.len() > self.data.len() {
//             return Err(TableError::OutOfRangeError {
//                 index: src.end,
//                 limits: self.limits.clone(),
//             });
//         }

//         Ok(())
//     }

//     fn copy_inter(
//         _src_table: &mut Table,
//         _src_range: Range<u32>,
//         _dst_table: &mut Table,
//         _dst_range: Range<u32>,
//     ) -> Result<(), TableError> {
//         todo!()
//     }

//     fn init(&mut self, _offset: u32, _data: Vec<Element>) -> Result<(), TableError> {
//         todo!()
//     }

//     fn get_ref_type(&self) -> RefType {
//         match self.data {
//             TableData::FunctionReference(_) => RefType::FunctionReference,
//             TableData::ExternReference(_) => RefType::ExternReference,
//         }
//     }
// }

// #[derive(Debug, Clone, Error)]
// pub enum TableError {
//     #[error("Table index out of range: {index}, limits: {limits:?}")]
//     OutOfRangeError { index: usize, limits: Limits },
//     #[error("Table type mismatch in operation")]
//     TypeMismatchError,
// }

// pub type TableSize = u32;

// pub trait Tablelike {
//     fn get(&self, index: u32) -> Option<Reference>;
//     fn set(&mut self, index: u32, data: Reference) -> Result<(), TableError>;
//     fn size(&self) -> TableSize;
//     fn grow(&mut self) -> Result<TableSize, TableError>;
//     fn fill(&mut self, range: Range<usize>, data: Reference) -> Result<(), TableError>;
//     fn copy_within(&mut self, src: Range<usize>, dst: u32) -> Result<(), TableError>;
//     fn copy_inter(
//         src_table: &mut Table,
//         src_range: Range<u32>,
//         dst_table: &mut Table,
//         dst_range: Range<u32>,
//     ) -> Result<(), TableError>;
//     fn init(&mut self, offset: u32, data: Vec<Element>) -> Result<(), TableError>;
//     fn get_ref_type(&self) -> RefType;
// }
