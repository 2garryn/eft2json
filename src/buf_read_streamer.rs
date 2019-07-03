
use crate::parser::{ReadStream};
use std::io::{BufRead, Error};

pub struct BufReadStreamer<'a> {
    buf_reader: &'a mut BufRead
}

impl <'a>BufReadStreamer<'a> {
    pub fn new<R: BufRead>(buff: &'a mut R) -> BufReadStreamer {
        BufReadStreamer{
            buf_reader: buff
        }
    }
}


impl<'a> ReadStream for BufReadStreamer<'a> {
    fn read_u8(&mut self) -> Result<u8, Error> {
        let mut b: [u8; 1] = [0];
        self.buf_reader.read_exact(&mut b)?;
        Ok(u8::from_be_bytes(b))
    }
    fn read_u16(&mut self) -> Result<u16, Error> {
        let mut b: [u8; 2] = [0; 2];
        self.buf_reader.read_exact(&mut b)?;
        Ok(u16::from_be_bytes(b))
    }
    fn read_u32(&mut self) -> Result<u32, Error> {
        let mut b: [u8; 4] = [0; 4];
        self.buf_reader.read_exact(&mut b)?;
        Ok(u32::from_be_bytes(b))
    }
    fn read_i32(&mut self) -> Result<i32, Error> {
        let mut b: [u8; 4] = [0; 4];
        self.buf_reader.read_exact(&mut b)?;
        Ok(i32::from_be_bytes(b))
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        self.buf_reader.read_exact(buf)?;
        Ok(())
    }
}
