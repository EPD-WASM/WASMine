pub mod block;
pub mod br;
pub mod br_if;
pub mod br_table;
pub mod call;
pub mod call_indirect;
pub mod if_else;
pub mod r#loop;
pub mod pseudo;
pub mod r#return;
pub mod unreachable;

pub use block::*;
pub use br::*;
pub use br_if::*;
pub use br_table::*;
pub use call::*;
pub use call_indirect::*;
pub use if_else::*;
pub use pseudo::*;
pub use r#loop::*;
pub use r#return::*;
pub use unreachable::*;

use super::*;
use wasm_types::BlockType;
