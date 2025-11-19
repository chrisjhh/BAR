use crate::BinaryStruct;
use std::io;

#[allow(dead_code)]
pub struct BARBook<'a, T: io::Read> {
    reader: &'a mut T,
    file_offset: u32,
    header: BARBookHeader,
    chapter_index: Vec<BARChapterIndexEntry>,
}

#[allow(dead_code)]
struct BARBookHeader {
    book_number: u8,
    number_of_chapters: u8,
}

#[allow(dead_code)]
struct BARChapterIndexEntry {
    additional_offset: u32, // file offset of chapter from start of book entry
}

impl BinaryStruct for BARBookHeader {
    fn byte_size() -> usize {
        2
    }

    fn from_bytes(buf: &[u8]) -> Result<Box<Self>, Box<dyn std::error::Error>> {
        if buf.len() != Self::byte_size() {
            return Err(format!("Buffer should be {} bytes long.", Self::byte_size()).into());
        }
        let book_number = buf[0];
        let number_of_chapters = buf[1];
        Ok(Box::new(BARBookHeader {
            book_number,
            number_of_chapters,
        }))
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        result.push(self.book_number);
        result.push(self.number_of_chapters);
        result
    }
}

impl BinaryStruct for BARChapterIndexEntry {
    fn byte_size() -> usize {
        4
    }

    fn from_bytes(buf: &[u8]) -> Result<Box<Self>, Box<dyn std::error::Error>> {
        super::check_size!(buf);
        let mut bytes: [u8; 4] = [0; 4];
        bytes.copy_from_slice(&buf[0..4]);
        let additional_offset = u32::from_le_bytes(bytes);
        Ok(Box::new(BARChapterIndexEntry { additional_offset }))
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        for byte in self.additional_offset.to_le_bytes() {
            result.push(byte);
        }
        result
    }
}
