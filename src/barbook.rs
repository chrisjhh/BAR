use crate::BinaryStruct;
use std::cell::RefCell;
use std::error::Error;
use std::io;
use std::rc::Rc;

const BOOK_NAMES: [&str; 66] = [
    "Genesis",
    "Exodus",
    "Leviticus",
    "Numbers",
    "Duteronomy",
    "Joshua",
    "Judges",
    "Ruth",
    "1 Samuel",
    "2 Samuel",
    "1 Kings",
    "2 Kings",
    "1 Chronicles",
    "2 Chronicles",
    "Ezra",
    "Nehemiah",
    "Esther",
    "Job",
    "Psalms",
    "Proverbs",
    "Eccesiastes",
    "Song of Songs",
    "Isaiah",
    "Jeremiah",
    "Lamentations",
    "Ezekiel",
    "Daniel",
    "Hosea",
    "Joel",
    "Amos",
    "Obadiah",
    "Jonah",
    "Micah",
    "Nahum",
    "Habakkuk",
    "Zephaniah",
    "Haggai",
    "Zechariah",
    "Malachi",
    "Matthew",
    "Mark",
    "Luke",
    "John",
    "Acts",
    "Romans",
    "1 Corinthians",
    "2 Corinthians",
    "Galatians",
    "Ephesians",
    "Philippians",
    "Colossians",
    "1 Thessalonians",
    "2 Thessalonians",
    "1 Timothy",
    "2 Timothy",
    "Titus",
    "Philemon",
    "Hebrews",
    "James",
    "1 Peter",
    "2 Peter",
    "1 John",
    "2 John",
    "3 John",
    "Jude",
    "Revelation",
];
const BOOK_ABBREVS: [&str; 66] = [
    "Ge", "Ex", "Lev", "Nu", "Dt", "Jos", "Jdg", "Ru", "1Sa", "2Sa", "1Ki", "2Ki", "1Ch", "2Ch",
    "Ezr", "Ne", "Est", "Job", "Ps", "Pr", "Ecc", "SS", "Isa", "Jer", "La", "Eze", "Da", "Hos",
    "Joel", "Am", "Ob", "Jnh", "Mic", "Na", "Hab", "Zep", "Hag", "Zec", "Mal", "Mt", "Mk", "Lk",
    "Jn", "Ac", "Ro", "1Co", "2Co", "Gal", "Eph", "Php", "Col", "1Th", "2Th", "1Ti", "2Ti", "Tit",
    "Phm", "Heb", "Jas", "1Pe", "2Pe", "1Jn", "2Jn", "3Jn", "Jude", "Rev",
];

mod barchapter;
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

    fn from_bytes(buf: &[u8]) -> Result<Box<Self>, Box<dyn std::error::Error>> {
        super::check_size!(buf);
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
        match additional_offset {
            0 => Ok(Box::new(BARChapterIndexEntry::Empty)),
            offset => Ok(Box::new(BARChapterIndexEntry::Live {
                additional_offset: offset,
            })),
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
        let header = *BARBookHeader::read_from(reader)?;
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
            reader: shared_reader.clone(),
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
        if chapter_option.is_none() {
            return None;
        }
        match chapter_option.unwrap() {
            BARChapterIndexEntry::Empty => None,
            BARChapterIndexEntry::Live { additional_offset } => {
                let file_offset = self.file_offset + additional_offset;
                match BARChapter::build(
                    self.reader.clone(),
                    self.header.book_number,
                    chapter_number,
                    file_offset,
                    self.file_version,
                ) {
                    Ok(chapter) => Some(chapter),
                    _ => None,
                }
            }
        }
    }
}

impl<T: io::Read + io::Seek> Iterator for BARBook<T> {
    type Item = Option<BARChapter<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let current_index: Option<usize> = match self.iterator_index {
            None if self.chapter_index.is_empty() => None,
            None => Some(1),
            Some(x) if x + 1 > self.chapter_index.len() => None,
            Some(x) => Some(x + 1),
        };
        self.iterator_index = current_index;
        let i = match current_index {
            None => return None,
            Some(index) => index,
        };
        Some(self.chapter(i as u8))
    }
}
