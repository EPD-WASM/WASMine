pub(crate) type Address = u64;

#[derive(Debug, Clone)]
pub(crate) struct FuncAddress(Address);

#[derive(Debug, Clone)]
pub(crate) struct TableAddress(Address);

#[derive(Debug, Clone)]
pub(crate) struct MemAddress(Address);

#[derive(Debug, Clone)]
pub(crate) struct GlobalAddress(Address);

#[derive(Debug, Clone)]
pub(crate) struct ElemAddress(Address);

#[derive(Debug, Clone)]
pub(crate) struct DataAddress(Address);

#[derive(Debug, Clone)]
pub(crate) struct ExternAddress(Address);
