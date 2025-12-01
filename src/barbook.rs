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
    "Ge", "Ex", "Lev", "Nu", "Dt", "Jos", "Jdg", "Ru", "1Sa", "2Sa", "1Ki", "2Ki", "1Ch",
    "2Ch", // 0-13
    "Ezr", "Ne", "Est", "Job", "Ps", "Pr", "Ecc", "SS", "Isa", "Jer", "La", "Eze", "Da",
    "Hos", // 14-27
    "Joel", "Am", "Ob", "Jnh", "Mic", "Na", "Hab", "Zep", "Hag", "Zec", "Mal", "Mt", "Mk",
    "Lk", // 28-41
    "Jn", "Ac", "Ro", "1Co", "2Co", "Gal", "Eph", "Php", "Col", "1Th", "2Th", "1Ti", "2Ti",
    "Tit", // 42-55
    "Phm", "Heb", "Jas", "1Pe", "2Pe", "1Jn", "2Jn", "3Jn", "Jude", "Rev", // 56-65
];

macro_rules! some_at_end {
    ($chars:ident, $val:literal) => {
        match $chars.next() {
            None => Some($val),
            Some(' ') => Some($val),
            Some(_) => None,
        }
    };
    ($chars:ident, $val:literal, $opt:literal) => {
        match $chars.next() {
            None => Some($val),
            Some(' ') => Some($val),
            Some($opt) => some_at_end!($chars, 0),
            _ => None,
        }
    };
}

#[allow(dead_code)]
pub fn parse_book_abbrev(text: &str) -> Option<usize> {
    let mut chars = text.chars();
    match chars.next()? {
        'G' => match chars.next()? {
            'e' => some_at_end!(chars, 0, 'n'),
            'a' => match chars.next()? {
                'l' => some_at_end!(chars, 47),
                _ => None,
            },
            _ => None,
        },
        'E' => match chars.next()? {
            'x' => some_at_end!(chars, 1),
            'z' => match chars.next()? {
                'r' => some_at_end!(chars, 14),
                'e' => some_at_end!(chars, 25),
                _ => None,
            },
            's' => match chars.next()? {
                't' => some_at_end!(chars, 16),
                _ => None,
            },
            'c' => match chars.next()? {
                'c' => some_at_end!(chars, 20),
                _ => None,
            },
            'p' => match chars.next()? {
                'h' => some_at_end!(chars, 48),
                _ => None,
            },
            _ => None,
        },
        'L' => match chars.next()? {
            'e' => match chars.next()? {
                'v' => some_at_end!(chars, 2),
                _ => None,
            },
            'a' => some_at_end!(chars, 24, 'm'),
            'k' => some_at_end!(chars, 41),
            _ => None,
        },
        'N' => match chars.next()? {
            'u' => some_at_end!(chars, 3, 'm'),
            'e' => some_at_end!(chars, 15),
            'a' => some_at_end!(chars, 33),
            _ => None,
        },
        'D' => match chars.next()? {
            't' => some_at_end!(chars, 4),
            'a' => some_at_end!(chars, 26, 'n'),
            _ => None,
        },
        'J' => match chars.next()? {
            'o' => match chars.next()? {
                's' => some_at_end!(chars, 5),
                'b' => some_at_end!(chars, 17),
                'e' => match chars.next()? {
                    'l' => some_at_end!(chars, 28),
                    _ => None,
                },
                _ => None,
            },
            'd' => match chars.next()? {
                'g' => some_at_end!(chars, 6),
                _ => None,
            },
            'e' => match chars.next()? {
                'r' => some_at_end!(chars, 23),
                _ => None,
            },
            'n' => match chars.next() {
                None => Some(42),
                Some(' ') => Some(42),
                Some('h') => some_at_end!(chars, 31),
                _ => None,
            },
            'a' => match chars.next()? {
                's' => some_at_end!(chars, 58),
                _ => None,
            },
            'u' => match chars.next()? {
                'd' => match chars.next()? {
                    'e' => some_at_end!(chars, 64),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        },
        'R' => match chars.next()? {
            'u' => some_at_end!(chars, 7),
            'o' => some_at_end!(chars, 44),
            'e' => match chars.next()? {
                'v' => some_at_end!(chars, 65),
                _ => None,
            },
            _ => None,
        },
        '1' => match chars.next()? {
            'S' => match chars.next()? {
                'a' => some_at_end!(chars, 8),
                _ => None,
            },
            'K' => match chars.next()? {
                'i' => some_at_end!(chars, 10),
                _ => None,
            },
            'C' => match chars.next()? {
                'h' => some_at_end!(chars, 12),
                'o' => some_at_end!(chars, 45),
                _ => None,
            },
            'T' => match chars.next()? {
                'h' => some_at_end!(chars, 51),
                'i' => some_at_end!(chars, 53),
                _ => None,
            },
            'P' => match chars.next()? {
                'e' => some_at_end!(chars, 59),
                _ => None,
            },
            'J' => match chars.next()? {
                'n' => some_at_end!(chars, 61),
                _ => None,
            },
            _ => None,
        },
        '2' => match chars.next()? {
            'S' => match chars.next()? {
                'a' => some_at_end!(chars, 9),
                _ => None,
            },
            'K' => match chars.next()? {
                'i' => some_at_end!(chars, 11),
                _ => None,
            },
            'C' => match chars.next()? {
                'h' => some_at_end!(chars, 13),
                'o' => some_at_end!(chars, 46),
                _ => None,
            },
            'T' => match chars.next()? {
                'h' => some_at_end!(chars, 52),
                'i' => some_at_end!(chars, 54),
                _ => None,
            },
            'P' => match chars.next()? {
                'e' => some_at_end!(chars, 60),
                _ => None,
            },
            'J' => match chars.next()? {
                'n' => some_at_end!(chars, 62),
                _ => None,
            },
            _ => None,
        },
        'P' => match chars.next()? {
            's' => some_at_end!(chars, 18),
            'r' => some_at_end!(chars, 19),
            'h' => match chars.next()? {
                'p' => some_at_end!(chars, 49),
                'm' => some_at_end!(chars, 56),
                _ => None,
            },
            _ => None,
        },
        'S' => match chars.next()? {
            'S' => some_at_end!(chars, 21),
            'o' => some_at_end!(chars, 21, 'S'),
            _ => None,
        },
        'I' => match chars.next()? {
            's' => match chars.next()? {
                'a' => some_at_end!(chars, 22),
                _ => None,
            },
            _ => None,
        },
        'H' => match chars.next()? {
            'o' => match chars.next()? {
                's' => some_at_end!(chars, 27),
                _ => None,
            },
            'a' => match chars.next()? {
                'b' => some_at_end!(chars, 34),
                'g' => some_at_end!(chars, 36),
                _ => None,
            },
            'e' => match chars.next()? {
                'b' => some_at_end!(chars, 57),
                _ => None,
            },
            _ => None,
        },
        'A' => match chars.next()? {
            'm' => some_at_end!(chars, 29),
            'c' => some_at_end!(chars, 43),
            _ => None,
        },
        'O' => match chars.next()? {
            'b' => some_at_end!(chars, 30),
            _ => None,
        },
        'M' => match chars.next()? {
            'i' => match chars.next()? {
                'c' => some_at_end!(chars, 32),
                _ => None,
            },
            'a' => match chars.next()? {
                'l' => some_at_end!(chars, 38),
                _ => None,
            },
            't' => some_at_end!(chars, 39),
            'k' => some_at_end!(chars, 40),
            _ => None,
        },
        'Z' => match chars.next()? {
            'e' => match chars.next()? {
                'p' => some_at_end!(chars, 35),
                'c' => some_at_end!(chars, 37),
                _ => None,
            },
            _ => None,
        },
        'C' => match chars.next()? {
            'o' => match chars.next()? {
                'l' => some_at_end!(chars, 50),
                _ => None,
            },
            _ => None,
        },
        'T' => match chars.next()? {
            'i' => match chars.next()? {
                't' => some_at_end!(chars, 55),
                _ => None,
            },
            _ => None,
        },
        '3' => match chars.next()? {
            'J' => match chars.next()? {
                'n' => some_at_end!(chars, 63),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

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
        if chapter_option.is_none() {
            return None;
        }
        match chapter_option.unwrap() {
            BARChapterIndexEntry::Empty => None,
            BARChapterIndexEntry::Live { additional_offset } => {
                let file_offset = self.file_offset + additional_offset;
                match BARChapter::build(
                    Rc::clone(&self.reader),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_book_abbrev() {
        for i in 0..66 {
            let abbrev = BOOK_ABBREVS[i];
            let book_index = parse_book_abbrev(abbrev).unwrap();
            assert_eq!(book_index, i, "Incorrect index for {}", abbrev);
            let abbrev_with_space = abbrev.to_string() + " ";
            let book_index = parse_book_abbrev(&abbrev_with_space).unwrap();
            assert_eq!(book_index, i, "Incorrect index for [{}]", abbrev);
            let abbrev_with_q = abbrev.to_string() + "q";
            let book_index = parse_book_abbrev(&abbrev_with_q);
            assert!(book_index.is_none());
        }
        let random_text = "Hello World!";
        let book_index = parse_book_abbrev(&random_text);
        assert!(book_index.is_none());
    }
}
