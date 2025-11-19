use crate::BinaryStruct;
use std::error::Error;
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

impl<'a, T: io::Read + io::Seek> BARBook<'a, T> {
    pub fn build(
        reader: &'a mut T,
        book_number: u8,
        file_offset: u32,
    ) -> Result<Self, Box<dyn Error>> {
        reader.seek(io::SeekFrom::Start(u64::from(file_offset)))?;
        let header = *BARBookHeader::read_from(reader)?;
        if header.book_number != book_number {
            return Err(format!(
                "Book index number mismatch. Expected: {}. Got: {}",
                book_number, header.book_number
            )
            .into());
        }
        let mut buf: Vec<u8> = Vec::new();
        let buf_size: usize =
            usize::from(header.number_of_chapters) * BARChapterIndexEntry::byte_size();
        buf.resize(buf_size, 0);
        reader.read_exact(&mut buf[..])?;
        let mut chapter_index: Vec<BARChapterIndexEntry> = Vec::new();
        for i in 0..header.number_of_chapters {
            let start: usize = usize::from(i) * BARChapterIndexEntry::byte_size();
            let end: usize = start + BARChapterIndexEntry::byte_size();
            let entry = BARChapterIndexEntry::from_bytes(&buf[start..end])?;
            chapter_index.push(*entry);
        }
        Ok(BARBook {
            reader,
            file_offset,
            header,
            chapter_index,
        })
    }

    /// Return the book number 1=Genesis 66=Revelation
    pub fn book_number(&self) -> u8 {
        self.header.book_number
    }

    /// Return the number of chapters
    pub fn number_of_chapters(&self) -> u8 {
        self.header.number_of_chapters
    }
}
