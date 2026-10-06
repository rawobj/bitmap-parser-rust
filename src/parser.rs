use super::error::BitmapParseError;

pub struct BoxedBufferParser<'a> {
    data: &'a Box<[u8]>, // Since an image buffer is fixed in size after reading from disk, we can use a boxed slice.
    // This drops the unused capacity field, reducing Vec's stack foorprint from 24 bytes to 16 bytes.
    cursor: usize,
}

impl<'a> BoxedBufferParser<'a> {
    pub fn new(data: &'a Box<[u8]>) -> Self {
        Self { data, cursor: 0 }
    }

    pub fn position(&self) -> &usize {
        return &self.cursor;
    }

    pub fn read_bytes(&mut self, count: usize) -> Result<&[u8], BitmapParseError> {
        if self.cursor + count > self.data.len() {
            return Err(BitmapParseError::UnexpectedEOF);
        }
        let slice = &self.data[self.cursor..self.cursor + count];
        self.cursor += count;
        Ok(slice)
    }

    pub fn read_u16_le(&mut self) -> Result<u16, BitmapParseError> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn read_u32_le(&mut self) -> Result<u32, BitmapParseError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }
}
