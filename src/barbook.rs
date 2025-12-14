use crate::BinaryStruct;
use std::cell::RefCell;
use std::error::Error;
use std::io;
use std::rc::Rc;

use bible_data::{BOOK_ABBREVS, BOOK_NAMES};

pub mod barchapter;
use barchapter::BARChapter;

#[allow(dead_code)]
pub struct BARBook<T: io::Read + io::Seek> {
    reader: Rc<RefCell<T>>,
    file_version: u8,
    file_offset: u32,
    header: BARBookHeader,
    chapter_index: Vec<BARChapterIndexEntry>,
    iterator_index: Option<usize>,
}

#[allow(dead_code)]
struct BARBookHeader {
    book_number: u8,
    number_of_chapters: u8,
}

#[allow(dead_code)]
enum BARChapterIndexEntry {
    Live {
        additional_offset: u32, // file offset of chapter from start of book entry
    },
    Empty,
}

impl BinaryStruct for BARBookHeader {
    fn byte_size() -> usize {
        2
    }

    fn from_bytes(buf: &[u8]) -> Self {
        super::check_size!(buf);
        let book_number = buf[0];
        let number_of_chapters = buf[1];
        BARBookHeader {
            book_number,
            number_of_chapters,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let result: Vec<u8> = vec![self.book_number, self.number_of_chapters];
        result
    }
}

impl BinaryStruct for BARChapterIndexEntry {
    fn byte_size() -> usize {
        4
    }

    fn from_bytes(buf: &[u8]) -> Self {
        super::check_size!(buf);
        let mut bytes: [u8; 4] = [0; 4];
        bytes.copy_from_slice(&buf[0..4]);
        let additional_offset = u32::from_le_bytes(bytes);
        match additional_offset {
            0 => BARChapterIndexEntry::Empty,
            offset => BARChapterIndexEntry::Live {
                additional_offset: offset,
            },
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        match self {
            BARChapterIndexEntry::Live { additional_offset } => {
                for byte in additional_offset.to_le_bytes() {
                    result.push(byte);
                }
            }
            BARChapterIndexEntry::Empty => {
                result.resize(BARChapterIndexEntry::byte_size(), b'\0');
            }
        }
        result
    }
}

impl<T: io::Read + io::Seek> BARBook<T> {
    pub fn build(
        shared_reader: Rc<RefCell<T>>,
        book_number: u8,
        file_offset: u32,
        file_version: u8,
    ) -> Result<Self, Box<dyn Error>> {
        let reader = &mut *shared_reader.borrow_mut();
        reader.seek(io::SeekFrom::Start(u64::from(file_offset)))?;
        let header = BARBookHeader::read_from(reader)?;
        if header.book_number != book_number {
            return Err(format!(
                "Book index number mismatch. Expected: {}. Got: {}",
                book_number, header.book_number
            )
            .into());
        }
        let chapter_index =
            BARChapterIndexEntry::read_array(usize::from(header.number_of_chapters), reader)?;
        Ok(BARBook {
            reader: Rc::clone(&shared_reader),
            file_offset,
            header,
            chapter_index,
            file_version,
            iterator_index: None,
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

    pub fn book_name(&self) -> &str {
        let i = usize::from(self.book_number());
        if i > 0 && i <= 66 {
            return BOOK_NAMES[i - 1];
        }
        "Unknown"
    }

    pub fn book_abbrev(&self) -> &str {
        let i = usize::from(self.book_number());
        if i > 0 && i <= 66 {
            return BOOK_ABBREVS[i - 1];
        }
        "???"
    }

    pub fn chapter(&self, chapter_number: u8) -> Option<BARChapter<T>> {
        // First chapter is 1 but array starts at zero
        if chapter_number == 0 {
            return None;
        }
        let index = chapter_number - 1;
        let chapter_option = self.chapter_index.get(usize::from(index));
        match chapter_option? {
            BARChapterIndexEntry::Empty => None,
            BARChapterIndexEntry::Live { additional_offset } => {
                let file_offset = self.file_offset + additional_offset;
                BARChapter::build(
                    Rc::clone(&self.reader),
                    self.header.book_number,
                    chapter_number,
                    file_offset,
                    self.file_version,
                )
                .ok()
            }
        }
    }

    pub fn chapters<'a>(&'a self) -> BARBookIterator<'a, T> {
        BARBookIterator {
            barbook: self,
            index: 1,
        }
    }
}

pub struct BARBookIterator<'a, T: io::Seek + io::Read> {
    barbook: &'a BARBook<T>,
    index: u8,
}

impl<'a, T: io::Seek + io::Read> Iterator for BARBookIterator<'a, T> {
    type Item = Option<BARChapter<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let current_index = self.index;
        if current_index as usize > self.barbook.chapter_index.len() {
            return None;
        }
        self.index += 1;
        Some(self.barbook.chapter(current_index))
    }
}
