pub type Address = u64;

#[derive(Debug, Clone)]
pub struct FuncAddress(Address);

#[derive(Debug, Clone)]
pub struct TableAddress(Address);

#[derive(Debug, Clone)]
pub struct MemAddress(Address);

#[derive(Debug, Clone)]
pub struct GlobalAddress(Address);

#[derive(Debug, Clone)]
pub struct ElemAddress(Address);

#[derive(Debug, Clone)]
pub struct DataAddress(Address);

#[derive(Debug, Clone)]
pub struct ExternAddress(Address);
