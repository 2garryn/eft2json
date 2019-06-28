use super::parse_result::ParseError;

pub trait ReadStream {
    fn read_u8(&mut self) -> Result<u8, ParseError>;
    fn read_u16(&mut self) -> Result<u16, ParseError>;
    fn read_u32(&mut self) -> Result<u32, ParseError>;
    fn read_i32(&mut self) -> Result<i32, ParseError>;
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), ParseError>;
}
