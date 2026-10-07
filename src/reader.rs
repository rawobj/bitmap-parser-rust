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

    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N], BufferError> {
        let bytes = self.read_bytes(N)?;
        Ok(bytes.try_into().unwrap())
    }

    pub fn read_u16_le(&mut self) -> Result<u16, BufferError> {
        Ok(u16::from_le_bytes(self.read_array()?))
    }

    pub fn read_u32_le(&mut self) -> Result<u32, BufferError> {
        Ok(u32::from_le_bytes(self.read_array()?))
    }

    pub fn read_i32_le(&mut self) -> Result<i32, BufferError> {
        Ok(i32::from_le_bytes(self.read_array()?))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_bytes() {
        let data = [1, 2, 3, 4, 5];
        let mut reader = BufferReader::new(&data);

        assert_eq!(reader.read_bytes(3).unwrap(), &[1, 2, 3]);
        assert_eq!(reader.read_bytes(2).unwrap(), &[4, 5]);
        assert!(reader.read_bytes(1).is_err());
    }

    #[test]
    fn test_read_array() {
        let data = [1, 2, 3, 4, 5];
        let mut reader = BufferReader::new(&data);

        assert_eq!(reader.read_array::<3>().unwrap(), [1, 2, 3]);
        assert_eq!(reader.read_array::<2>().unwrap(), [4, 5]);
        assert!(reader.read_array::<1>().is_err());
    }

    #[test]
    fn test_seek() {
        let data = [1, 2, 3, 4, 5];
        let mut reader = BufferReader::new(&data);

        reader.seek(2).unwrap();
        assert_eq!(reader.read_bytes(2).unwrap(), &[3, 4]);

        assert!(reader.seek(6).is_err());
    }

    #[test]
    fn test_read_u16_le() {
        let data = [0x34, 0x12];
        let mut reader = BufferReader::new(&data);

        assert_eq!(reader.read_u16_le().unwrap(), 0x1234);
    }

    #[test]
    fn test_read_u32_le() {
        let data = [0x78, 0x56, 0x34, 0x12];
        let mut reader = BufferReader::new(&data);

        assert_eq!(reader.read_u32_le().unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_i32_le() {
        let data = [0xFF, 0xFF, 0xFF, 0xFF];
        let mut reader = BufferReader::new(&data);

        assert_eq!(reader.read_i32_le().unwrap(), -1);
    }
}