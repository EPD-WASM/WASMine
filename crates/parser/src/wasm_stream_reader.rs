use super::error::ParserError;
use module::utils::integer_traits::IsSigned;

pub(crate) struct WasmBinaryReader<'a> {
    binary_source: &'a [u8],
    pub(crate) pos: usize,
}

impl<'a> WasmBinaryReader<'a> {
    pub(crate) fn new<S>(reader: &'a S) -> Self
    where
        S: AsRef<[u8]> + 'a,
    {
        Self {
            binary_source: reader.as_ref(),
            pos: 0,
        }
    }

    /// Test if the stream is at the end.
    ///
    /// @return true if the stream is at the end, false otherwise.
    pub(crate) fn eof(&mut self) -> Result<bool, ParserError> {
        Ok(self.pos >= self.binary_source.len())
    }

    /// Advance the stream by n bytes.
    ///
    /// @param n the number of bytes to advance the stream by.
    pub(crate) fn advance(&mut self, n: usize) {
        self.pos += n;
    }

    /// Set the limit of the stream.
    ///
    /// This will truncate the stream to the given limit.
    /// @param limit the new limit of the stream.
    pub(crate) fn set_limit(&mut self, limit: usize) {
        self.binary_source = &self.binary_source[..limit];
    }

    /// Read a single byte from the stream.
    ///
    /// @return the byte read from the stream.
    pub(crate) fn read_byte(&mut self) -> Result<u8, ParserError> {
        self.binary_source
            .get(self.pos)
            .map(|b| {
                self.pos += 1;
                *b
            })
            .ok_or(ParserError::UnexpectedEOF)
    }

    /// Peek a single byte from the stream.
    ///
    /// @return the byte peeked from the stream.
    pub(crate) fn peek_byte(&mut self) -> Result<u8, ParserError> {
        self.binary_source
            .get(self.pos)
            .ok_or(ParserError::UnexpectedEOF)
            .cloned()
    }

    /// Read a single 32-bit unsigned integer from the stream.
    ///
    /// @return the 32-bit unsigned integer read from the stream.
    // these could be made generic, but the const generic feature is still unstable
    pub(crate) fn read_u32(&mut self) -> Result<u32, ParserError> {
        if self.pos + 4 > self.binary_source.len() {
            return Err(ParserError::UnexpectedEOF);
        }
        let bytes: [u8; 4] = self.binary_source[self.pos..self.pos + 4]
            .try_into()
            .unwrap();
        self.pos += 4;
        Ok(u32::from_le_bytes(bytes))
    }

    /// Read a single 32-bit IEEE-754 floating point number from the stream.
    ///
    /// @return the 32-bit float read from the stream.
    // these could be made generic, but the const generic feature is still unstable
    pub(crate) fn read_f32(&mut self) -> Result<f32, ParserError> {
        if self.pos + 4 > self.binary_source.len() {
            return Err(ParserError::UnexpectedEOF);
        }
        let bytes: [u8; 4] = self.binary_source[self.pos..self.pos + 4]
            .try_into()
            .unwrap();
        self.pos += 4;
        Ok(f32::from_le_bytes(bytes))
    }

    /// Read a single 64-bit IEEE-754 floating point number from the stream.
    ///
    /// @return the 64-bit float read from the stream.
    // these could be made generic, but the const generic feature is still unstable
    pub(crate) fn read_f64(&mut self) -> Result<f64, ParserError> {
        if self.pos + 8 > self.binary_source.len() {
            return Err(ParserError::UnexpectedEOF);
        }
        let bytes: [u8; 8] = self.binary_source[self.pos..self.pos + 8]
            .try_into()
            .unwrap();
        self.pos += 8;
        Ok(f64::from_le_bytes(bytes))
    }

    /// Read a LEB128 encoded integer from the stream.
    ///
    /// @tparam T the type of the integer to read.
    /// @return the integer read from the stream.
    /// ALgorithm source: https://en.wikipedia.org/wiki/LEB128
    pub(crate) fn read_leb128<T>(&mut self) -> Result<T, ParserError>
    where
        T: IsSigned
            + Default
            + std::ops::Shl<usize, Output = T>
            + std::ops::Shr<usize, Output = T>
            + std::ops::BitOrAssign<<T as std::ops::Shl<usize>>::Output>
            + std::ops::Not<Output = T>
            + std::ops::BitAnd<Output = T>
            + std::cmp::Eq
            + From<u8>
            + Copy,
    {
        let out_size: usize = std::mem::size_of::<T>() * 8;

        let byte = self.read_byte()?;
        if (byte & 0x80) == 0 {
            if T::SIGNED {
                return Ok((T::from(byte) << (out_size - 7)) >> (out_size - 7));
            }
            return Ok(T::from(byte));
        }

        let mut result = T::from(byte & 0x7F);
        let mut i = 1;
        loop {
            let byte = self.read_byte()?;

            result |= T::from(byte & 0x7F) << (i * 7);
            if !T::SIGNED && (i * 7) >= out_size - 7 && (byte >> (out_size - i * 7)) != 0 {
                return Err(ParserError::InvalidLEB128Encoding);
            }

            if T::SIGNED && (i * 7) >= out_size - 7 {
                let high_bit_is_set = (byte & 0x80) != 0;
                let sign_and_unused_bit = (byte << 1) as i8 >> (out_size - 7 * i);
                if high_bit_is_set || (sign_and_unused_bit != 0 && sign_and_unused_bit != -1) {
                    return Err(ParserError::InvalidLEB128Encoding);
                }
                return Ok(result);
            }
            i += 1;
            if (byte & 0x80) == 0 {
                if !T::SIGNED {
                    return Ok(result);
                }
                let ashift = out_size - 7 * i;
                return Ok((result << ashift) >> ashift);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eof() {
        let stream_data = [];
        let mut reader = WasmBinaryReader::new(&stream_data);
        assert!(reader.eof().unwrap());

        let stream_data = [0x42];
        let mut reader = WasmBinaryReader::new(&stream_data);
        assert!(!reader.eof().unwrap());
    }

    #[test]
    fn test_read_byte() {
        let stream_data = [];
        let mut reader = WasmBinaryReader::new(&stream_data);
        assert!(reader.read_byte().is_err());

        let stream_data = [0x42, 0x43];
        let mut reader = WasmBinaryReader::new(&stream_data);

        assert_eq!(reader.read_byte().unwrap(), 0x42);
        assert_eq!(reader.read_byte().unwrap(), 0x43);
        assert!(reader.read_byte().is_err());
    }

    #[test]
    fn test_read_u32() {
        let stream_data = [];
        let mut reader = WasmBinaryReader::new(&stream_data);
        assert!(reader.read_u32().is_err());

        let stream_data = [0x0];
        let mut reader = WasmBinaryReader::new(&stream_data);
        assert!(reader.read_u32().is_err());

        let stream_data = [0x0, 0x0];
        let mut reader = WasmBinaryReader::new(&stream_data);
        assert!(reader.read_u32().is_err());

        let stream_data = [0x0, 0x0, 0x0];
        let mut reader = WasmBinaryReader::new(&stream_data);
        assert!(reader.read_u32().is_err());

        let stream_data = [0x1, 0x2, 0x3, 0x4, 0x0, 0x42, 0x0, 0x42];
        let mut reader = WasmBinaryReader::new(&stream_data);
        assert_eq!(
            reader.read_u32().unwrap(),
            u32::from_le_bytes([0x1, 0x2, 0x3, 0x4])
        );
        assert_eq!(
            reader.read_u32().unwrap(),
            u32::from_le_bytes([0x0, 0x42, 0x0, 0x42])
        );
        assert!(reader.read_byte().is_err());
    }

    #[test]
    fn test_read_leb128_single_signed() {
        let stream_data = [0x8e, 0x7f];
        let mut reader = WasmBinaryReader::new(&stream_data);
        assert_eq!(reader.read_leb128::<i32>().unwrap(), -114);
        assert!(reader.read_leb128::<u32>().is_err());
        assert!(reader.eof().unwrap());
    }

    #[test]
    fn test_read_leb128_multiple_signed() {
        #[rustfmt::skip]
        let stream_data = [
            /* 194751635 */ 0x93, 0xd9, 0xee, 0xdc, 0x00,
            /* -7280002181293982082 */ 0xfe, 0xfc, 0x9c, 0x9f, 0xe5, 0x92, 0x8f, 0xfc, 0x9a, 0x7f,
            /* 24909 */ 0xcd, 0xc2, 0x01,
            /* -37 */ 0x5b
        ];
        let mut reader = WasmBinaryReader::new(&stream_data);
        assert_eq!(reader.read_leb128::<i32>().unwrap(), 194751635);
        assert_eq!(reader.read_leb128::<i64>().unwrap(), -7280002181293982082);
        assert_eq!(reader.read_leb128::<i32>().unwrap(), 24909);
        assert_eq!(reader.read_leb128::<i64>().unwrap(), -37);
        assert!(reader.read_leb128::<i32>().is_err());
        assert!(reader.eof().unwrap());
    }

    #[test]
    fn test_read_leb128_single_unsigned() {
        let stream_data = [0xd9, 0x01];
        let mut reader = WasmBinaryReader::new(&stream_data);
        assert_eq!(reader.read_leb128::<u32>().unwrap(), 217);
        assert!(reader.read_leb128::<u32>().is_err());
        assert!(reader.eof().unwrap());
    }

    #[test]
    fn test_read_leb128_multiple_unsigned() {
        #[rustfmt::skip]
        let stream_data = [
            /* 64517 */ 0x85, 0xf8, 0x03,
            /* 2387606507 */ 0xeb, 0xf7, 0xbf, 0xf2, 0x08,
            /* 7074 */ 0xa2, 0x37,
            /* 10794028799708388741 */ 0x85, 0xbb, 0xd1, 0xef, 0x90, 0x80, 0x86, 0xe6, 0x95, 0x01
        ];
        let mut reader = WasmBinaryReader::new(&stream_data);
        assert_eq!(reader.read_leb128::<u32>().unwrap(), 64517);
        assert_eq!(reader.read_leb128::<u32>().unwrap(), 2387606507);
        assert_eq!(reader.read_leb128::<u32>().unwrap(), 7074);
        assert_eq!(reader.read_leb128::<u64>().unwrap(), 10794028799708388741);
        assert!(reader.read_leb128::<u32>().is_err());
        assert!(reader.eof().unwrap());
    }
}
