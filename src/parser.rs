use super::error::BitmapParseError;

pub struct BoxedBufferParser<'a> {
    data: &'a [u8],
    cursor: usize,
}

impl<'a> BoxedBufferParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    pub fn seek(&mut self, target_position: usize) -> Result<(), BitmapParseError> {
        if target_position > self.data.len() {
            return Err(BitmapParseError::UnexpectedEOF);
        }
        self.cursor = target_position;
        Ok(())
    }

    pub fn read_bytes(&mut self, count: usize) -> Result<&[u8], BitmapParseError> {
        let end = self.cursor + count;
        if end > self.data.len() {
            return Err(BitmapParseError::UnexpectedEOF);
        }
        let bytes = &self.data[self.cursor..end];
        self.cursor = end;
        Ok(bytes)
    }

    pub fn read_u16_le(&mut self) -> Result<u16, BitmapParseError> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn read_u32_le(&mut self) -> Result<u32, BitmapParseError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn read_i32_le(&mut self) -> Result<i32, BitmapParseError> {
        let bytes = self.read_bytes(4)?;
        Ok(i32::from_le_bytes(bytes.try_into().unwrap()))
    }
}
