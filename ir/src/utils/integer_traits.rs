pub trait IsSigned {
    const SIGNED: bool;
}

impl IsSigned for i8 {
    const SIGNED: bool = true;
}

impl IsSigned for i16 {
    const SIGNED: bool = true;
}

impl IsSigned for i32 {
    const SIGNED: bool = true;
}

impl IsSigned for i64 {
    const SIGNED: bool = true;
}

impl IsSigned for isize {
    const SIGNED: bool = true;
}

impl IsSigned for u8 {
    const SIGNED: bool = false;
}

impl IsSigned for u16 {
    const SIGNED: bool = false;
}

impl IsSigned for u32 {
    const SIGNED: bool = false;
}

impl IsSigned for u64 {
    const SIGNED: bool = false;
}

impl IsSigned for usize {
    const SIGNED: bool = false;
}

pub trait Integer {
    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(self) -> Vec<u8>;
}

impl Integer for i8 {
    fn from_bytes(bytes: &[u8]) -> Self {
        i8::from_ne_bytes(<[u8; 1]>::try_from(bytes).unwrap())
    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_ne_bytes().to_vec()
    }
}

impl Integer for i16 {
    fn from_bytes(bytes: &[u8]) -> Self {
        i16::from_ne_bytes(<[u8; 2]>::try_from(bytes).unwrap())
    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_ne_bytes().to_vec()
    }
}

impl Integer for i32 {
    fn from_bytes(bytes: &[u8]) -> Self {
        i32::from_ne_bytes(<[u8; 4]>::try_from(bytes).unwrap())
    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_ne_bytes().to_vec()
    }
}

impl Integer for i64 {
    fn from_bytes(bytes: &[u8]) -> Self {
        i64::from_ne_bytes(<[u8; 8]>::try_from(bytes).unwrap())
    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_ne_bytes().to_vec()
    }
}

impl Integer for isize {
    fn from_bytes(bytes: &[u8]) -> Self {
        isize::from_ne_bytes(<[u8; std::mem::size_of::<isize>()]>::try_from(bytes).unwrap())
    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_ne_bytes().to_vec()
    }
}

impl Integer for u8 {
    fn from_bytes(bytes: &[u8]) -> Self {
        u8::from_ne_bytes(<[u8; 1]>::try_from(bytes).unwrap())
    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_ne_bytes().to_vec()
    }
}

impl Integer for u16 {
    fn from_bytes(bytes: &[u8]) -> Self {
        u16::from_ne_bytes(<[u8; 2]>::try_from(bytes).unwrap())
    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_ne_bytes().to_vec()
    }
}

impl Integer for u32 {
    fn from_bytes(bytes: &[u8]) -> Self {
        u32::from_ne_bytes(<[u8; 4]>::try_from(bytes).unwrap())
    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_ne_bytes().to_vec()
    }
}

impl Integer for u64 {
    fn from_bytes(bytes: &[u8]) -> Self {
        u64::from_ne_bytes(<[u8; 8]>::try_from(bytes).unwrap())
    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_ne_bytes().to_vec()
    }
}

impl Integer for usize {
    fn from_bytes(bytes: &[u8]) -> Self {
        usize::from_ne_bytes(<[u8; std::mem::size_of::<usize>()]>::try_from(bytes).unwrap())
    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_ne_bytes().to_vec()
    }
}
