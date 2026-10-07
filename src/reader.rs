use std::fmt::Display;

pub struct BufferReader<'a> {
    data: &'a [u8],
    cursor: usize,
}

#[derive(Debug)]
pub enum BufferError {
    UnexpectedEOF,
}

impl Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEOF => {
                write!(f, "Unexpected end of file.",)
            }
        }
    }
}

impl<'a> BufferReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    pub fn seek(&mut self, target_position: usize) -> Result<(), BufferError> {
        if target_position > self.data.len() {
            return Err(BufferError::UnexpectedEOF);
        }
        self.cursor = target_position;
        Ok(())
    }

    pub fn read_bytes(&mut self, count: usize) -> Result<&[u8], BufferError> {
        let end = self.cursor + count;
        if end > self.data.len() {
            return Err(BufferError::UnexpectedEOF);
        }
        let bytes = &self.data[self.cursor..end];
        self.cursor = end;
        Ok(bytes)
    }

    pub fn read_u16_le(&mut self) -> Result<u16, BufferError> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn read_u32_le(&mut self) -> Result<u32, BufferError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn read_i32_le(&mut self) -> Result<i32, BufferError> {
        let bytes = self.read_bytes(4)?;
        Ok(i32::from_le_bytes(bytes.try_into().unwrap()))
    }
}
