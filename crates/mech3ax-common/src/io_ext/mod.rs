use crate::assert::{assert_utf8, AssertionError};
use crate::size::ReprSize;
use crate::string::str_from_c_sized;
use std::io::{Read, Result, Seek, SeekFrom, Write};
use std::mem::MaybeUninit;

pub struct CountingReader<R: Read> {
    inner: R,
    pub offset: u32,
    pub prev: u32,
}

impl<R: Read> CountingReader<R> {
    pub fn new(read: R) -> Self {
        Self {
            inner: read,
            offset: 0,
            prev: 0,
        }
    }

    pub fn into_inner(self) -> R {
        self.inner
    }

    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        if let Err(err) = self.inner.read_exact(buf) {
            return Err(err);
        }
        self.prev = self.offset;
        self.offset += buf.len() as u32;
        Ok(())
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    pub fn read_f32(&mut self) -> Result<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub fn read_i16(&mut self) -> Result<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    #[allow(clippy::uninit_assumed_init)]
    pub fn read_struct<S>(&mut self) -> Result<S> {
        let size = std::mem::size_of::<S>();
        unsafe {
            let mut mem = MaybeUninit::uninit().assume_init();
            let buf = std::slice::from_raw_parts_mut(&mut mem as *mut S as *mut u8, size);
            match self.read_exact(buf) {
                Ok(()) => Ok(mem),
                Err(e) => {
                    std::mem::forget(mem);
                    Err(e)
                }
            }
        }
    }

    pub fn assert_end(&mut self) -> crate::Result<()> {
        let mut buf = [0; 1];
        match self.inner.read(&mut buf)? {
            0 => Ok(()),
            _ => Err(
                AssertionError(format!("Expected all data to be read (at {})", self.offset)).into(),
            ),
        }
    }

    pub fn read_string(&mut self) -> crate::Result<String> {
        let count = self.read_u32()? as usize;
        let mut buf = vec![0u8; count];
        self.read_exact(&mut buf)?;
        let value = assert_utf8("value", self.prev, || str_from_c_sized(&buf))?;
        Ok(value)
    }
}

impl<R: Read + Seek> Seek for CountingReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let offset = self.inner.seek(pos)?;
        self.offset = offset as u32;
        Ok(offset)
    }
}

pub trait ReadHelper {
    fn read_u32(&mut self) -> Result<u32>;
    fn read_u16(&mut self) -> Result<u16>;
}

impl<R> ReadHelper for R
where
    R: Read,
{
    fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }
}

pub trait WriteHelper {
    fn write_u32(&mut self, value: u32) -> Result<()>;
    fn write_i32(&mut self, value: i32) -> Result<()>;
    fn write_f32(&mut self, value: f32) -> Result<()>;
    fn write_u16(&mut self, value: u16) -> Result<()>;
    fn write_i16(&mut self, value: i16) -> Result<()>;
    fn write_struct<S: ReprSize>(&mut self, value: &S) -> Result<()>;
    fn write_string(&mut self, value: &str) -> crate::Result<()>;
    fn write_zeros(&mut self, count: u32) -> Result<()>;
}

impl<W> WriteHelper for W
where
    W: Write,
{
    fn write_u32(&mut self, value: u32) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    fn write_i32(&mut self, value: i32) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    fn write_f32(&mut self, value: f32) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    fn write_u16(&mut self, value: u16) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    fn write_i16(&mut self, value: i16) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    fn write_struct<S: ReprSize>(&mut self, value: &S) -> Result<()> {
        let size = std::mem::size_of::<S>();
        let buf = unsafe { std::slice::from_raw_parts(value as *const S as *const u8, size) };
        self.write_all(buf)
    }

    fn write_string(&mut self, value: &str) -> crate::Result<()> {
        if !value.is_ascii() {
            return Err(crate::Error::Assert(AssertionError(
                "Expected ASCII string".to_owned(),
            )));
        }
        let buf = value.as_bytes();
        let count = buf.len() as u32;
        self.write_u32(count)?;
        self.write_all(buf)?;
        Ok(())
    }

    fn write_zeros(&mut self, count: u32) -> Result<()> {
        let buf = vec![0; count as usize];
        self.write_all(&buf)
    }
}

#[cfg(test)]
mod tests;
