use std::io::{self, Result};

pub trait BinaryStruct
where
    Self: Sized,
{
    fn byte_size() -> usize;
    fn from_bytes(buf: &[u8]) -> Self
    where
        Self: Sized;
    fn to_bytes(&self) -> Vec<u8>;

    fn read_from(reader: &mut impl io::Read) -> Result<Self>
    where
        Self: Sized,
    {
        let mut buf: Vec<u8> = Vec::new();
        buf.resize(Self::byte_size(), b'\0');
        reader.read_exact(&mut buf[..])?;
        Ok(Self::from_bytes(&buf))
    }

    fn write_to(&self, writer: &mut impl io::Write) -> Result<()> {
        let bytes = self.to_bytes();
        writer.write_all(&bytes)
    }

    fn read_array(size: usize, reader: &mut impl io::Read) -> Result<Vec<Self>>
    where
        Self: Sized,
    {
        let mut buf: Vec<u8> = Vec::new();
        let mut results: Vec<Self> = Vec::new();
        buf.resize(size * Self::byte_size(), b'\0');
        reader.read_exact(&mut buf[..])?;
        for i in 0..size {
            let start: usize = i * Self::byte_size();
            let end: usize = start + Self::byte_size();
            let entry = Self::from_bytes(&buf[start..end]);
            results.push(entry);
        }
        Ok(results)
    }

    fn write_array(entries: &Vec<Self>, writer: &mut impl io::Write) -> Result<()>
    where
        Self: Sized,
    {
        let mut buf: Vec<u8> = Vec::new();
        for entry in entries {
            buf.append(&mut entry.to_bytes());
        }
        writer.write_all(&buf)?;
        Ok(())
    }
}
