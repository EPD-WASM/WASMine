pub(crate) mod load;
pub(crate) mod manage;
pub(crate) mod store;

pub(crate) use load::*;
pub(crate) use manage::*;
pub(crate) use store::*;

use super::*;
use crate::parsable::Parse;
use ir::structs::memory::*;
use wasm_types::*;
