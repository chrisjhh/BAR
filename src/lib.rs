use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::rc::Rc;

mod error;

mod barbook;
use barbook::BARBook;

const CURRENT_VERSION: (u8, u8) = (2, 2);

pub mod binarystruct;
use binarystruct::BinaryStruct;

#[macro_export]
macro_rules! check_size {
    ($buf:ident) => {
        if $buf.len() != Self::byte_size() {
            panic!("Buffer should be {} bytes long.", Self::byte_size());
        }
    };
}

pub struct BARFileHeader {
    leader: [u8; 3],
    pub major_version: u8,
    pub minor_version: u8,
    pub number_of_books: u8,
    pub version_abbrev: String,
}

pub enum BARBookIndexEntry {
    Live {
        book_number: u8, // (1=Gen 66=Rev)
        file_offset: u32,
    },
    Empty,
}

#[allow(dead_code)]
pub struct BARFile<T> {
    file: Rc<RefCell<T>>,
    pub header: BARFileHeader,
    pub book_index: Vec<BARBookIndexEntry>,
    iterator_index: Option<usize>,
}

impl BinaryStruct for BARFileHeader {
    fn byte_size() -> usize {
        16
    }

    fn from_bytes(buf: &[u8]) -> Self {
        check_size!(buf);
        let mut leader: [u8; 3] = [0; 3];
        leader.copy_from_slice(&buf[0..3]);
        let major_version = buf[3];
        let minor_version = buf[4];
        let number_of_books = buf[5];
        let version_abbrev = str::from_utf8(&buf[6..16])
            .unwrap_or("")
            .trim_end_matches("\0")
            .to_string();
        BARFileHeader {
            leader,
            major_version,
            minor_version,
            number_of_books,
            version_abbrev,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        result.append(&mut "BAR".as_bytes().to_vec());
        result.push(self.major_version);
        result.push(self.minor_version);
        result.push(self.number_of_books);
        result.append(&mut self.version_abbrev.as_bytes().to_vec());
        while result.len() < 16 {
            result.push(b'\0');
        }
        result
    }
}

impl BARFileHeader {
    fn default() -> Self {
        let mut leader: [u8; 3] = [0; 3];
        leader.copy_from_slice("BAR".as_bytes());
        BARFileHeader {
            leader,
            major_version: CURRENT_VERSION.0,
            minor_version: CURRENT_VERSION.1,
            number_of_books: 66,
            version_abbrev: String::from("N/A"),
        }
    }
}

impl BinaryStruct for BARBookIndexEntry {
    fn byte_size() -> usize {
        5
    }

    fn from_bytes(buf: &[u8]) -> Self {
        check_size!(buf);
        let book_number = buf[0];
        let mut bytes: [u8; 4] = [0; 4];
        bytes.copy_from_slice(&buf[1..5]);
        let file_offset = u32::from_le_bytes(bytes);
        if file_offset == 0 || book_number == 0 {
            return Self::Empty;
        }
        Self::Live {
            book_number,
            file_offset,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        match &self {
            Self::Live {
                book_number,
                file_offset,
            } => {
                result.push(*book_number);
                for i in file_offset.to_le_bytes() {
                    result.push(i);
                }
            }
            Self::Empty => {
                result.resize(Self::byte_size(), b'\0');
            }
        };
        result
    }
}

impl BARBookIndexEntry {
    fn default() -> Self {
        Self::Empty
    }
}

pub struct BARVersion(pub u8, pub u8);
impl std::fmt::Display for BARVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

impl<T: io::Read + io::Seek> Iterator for BARFile<T> {
    type Item = BARBook<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let current_index: Option<usize> = match self.iterator_index {
            None if self.book_index.is_empty() => None,
            None => Some(0),
            Some(x) if x + 1 >= self.book_index.len() => None,
            Some(x) => Some(x + 1),
        };
        self.iterator_index = current_index;
        let i = match current_index {
            None => return None,
            Some(index) => index,
        };
        let entry = &self.book_index[i];
        match entry {
            BARBookIndexEntry::Empty => None,
            BARBookIndexEntry::Live {
                book_number,
                file_offset: _,
            } => self.book(*book_number),
        }
    }
}

#[allow(dead_code)]
impl BARFile<File> {
    pub fn open(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let header = BARFileHeader::read_from(&mut reader)?;
        if header.leader != [b'B', b'A', b'R'] {
            return Err(format!(
                "Invalid BAR file. Unexpected leader: {}",
                String::from_utf8(header.leader.to_vec()).unwrap_or("???".to_string())
            )
            .into());
        }
        if header.version_abbrev.is_empty() {
            return Err(
                format!("Invalid BAR file. Version Abbrev not specified or corrupt.").into(),
            );
        }
        if header.major_version > CURRENT_VERSION.0 {
            return Err(format!(
                "Unsupported future BARFile version: {}.{}",
                header.major_version, header.minor_version
            )
            .into());
        }
        let book_index: Vec<BARBookIndexEntry> =
            BARBookIndexEntry::read_array(usize::from(header.number_of_books), &mut reader)?;
        let file = reader.into_inner();
        Ok(Self {
            file: Rc::new(RefCell::new(file)),
            header,
            book_index,
            iterator_index: None,
        })
    }

    pub fn create(file_path: &str, version_abbrev: String) -> Result<Self, Box<dyn Error>> {
        let default = BARFileHeader::default();
        let header = BARFileHeader {
            version_abbrev,
            ..default
        };
        Self::create_with_options(file_path, header)
    }

    pub fn create_with_options(
        file_path: &str,
        header: BARFileHeader,
    ) -> Result<Self, Box<dyn Error>> {
        let file = File::create_new(file_path)?;
        let mut writer = BufWriter::new(file);
        header.write_to(&mut writer)?;
        let book_index = Self::new_book_index(header.number_of_books);
        BARBookIndexEntry::write_array(&book_index, &mut writer)?;
        let file = writer.into_inner().unwrap();
        Ok(Self {
            file: Rc::new(RefCell::new(file)),
            header,
            book_index,
            iterator_index: None,
        })
    }
}

impl<T> BARFile<T> {
    pub fn archive_version(&self) -> BARVersion {
        BARVersion(self.header.major_version, self.header.minor_version)
    }

    pub fn bible_version(&self) -> &String {
        &self.header.version_abbrev
    }

    pub fn number_of_books(&self) -> i32 {
        let mut count = 0;
        for entry in &self.book_index {
            match entry {
                BARBookIndexEntry::Live { .. } => count += 1,
                BARBookIndexEntry::Empty => break,
            }
        }
        count
    }

    pub fn book_capacity(&self) -> i32 {
        assert_eq!(
            usize::from(self.header.number_of_books),
            self.book_index.len()
        );
        self.header.number_of_books as i32
    }

    fn new_book_index(number_of_books: u8) -> Vec<BARBookIndexEntry> {
        let mut book_index: Vec<BARBookIndexEntry> = Vec::new();
        book_index.resize_with(usize::from(number_of_books), || {
            BARBookIndexEntry::default()
        });
        book_index
    }
}

impl<T: io::Read + io::Seek> BARFile<T> {
    pub fn book(&mut self, book_number: u8) -> Option<BARBook<T>> {
        let mut file_offset: u32 = 0;
        for entry in &self.book_index {
            match entry {
                BARBookIndexEntry::Live {
                    book_number: entry_book_number,
                    file_offset: entry_file_offset,
                } => {
                    if *entry_book_number == book_number {
                        file_offset = *entry_file_offset;
                        break;
                    }
                }
                BARBookIndexEntry::Empty => break,
            }
        }
        if file_offset == 0 {
            return None;
        }
        match BARBook::build(
            self.file.clone(),
            book_number,
            file_offset,
            self.header.major_version,
        ) {
            Ok(bar) => Some(bar),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read, Seek};

    const NIV_HEADER: &str = "4241520200425A4C4942000000000000";
    const ESV_HEADER: &str = "42415202014245535600000000000000";
    const NIV_V1_HEADER: &str = "4241520100424E495600000000000000";
    const GREEK_HEADER: &str = "424152020142677265656B0000000000";

    impl<'a> BARFile<Cursor<&'a mut Vec<u8>>> {
        fn open_from_memory(buf: &'a mut Vec<u8>) -> Self {
            let mut file = Cursor::new(buf);
            let header = BARFileHeader::read_from(&mut file).unwrap();
            if header.major_version > CURRENT_VERSION.0 {
                panic!(
                    "Unsupported future BARFile version: {}.{}",
                    header.major_version, header.minor_version
                )
            }
            let book_index: Vec<BARBookIndexEntry> =
                BARBookIndexEntry::read_array(usize::from(header.number_of_books), &mut file)
                    .unwrap();
            Self {
                file: Rc::new(RefCell::new(file)),
                header,
                book_index,
                iterator_index: None,
            }
        }

        fn create_in_memory(buf: &'a mut Vec<u8>, version_abbrev: String) -> Self {
            let default = BARFileHeader::default();
            let header = BARFileHeader {
                version_abbrev,
                ..default
            };
            Self::create_in_memory_with_options(buf, header)
        }

        fn create_in_memory_with_options(buf: &'a mut Vec<u8>, header: BARFileHeader) -> Self {
            let mut file = Cursor::new(buf);
            header.write_to(&mut file).unwrap();
            let book_index = Self::new_book_index(header.number_of_books);
            for entry in &book_index {
                entry.write_to(&mut file).unwrap();
            }
            Self {
                file: Rc::new(RefCell::new(file)),
                header,
                book_index,
                iterator_index: None,
            }
        }
    }

    fn test_header(hex_header: &str, expected: (u8, u8, u8, &str)) {
        let bytes = hex::decode(hex_header).expect("Covert to bytes failed.");
        let header = BARFileHeader::from_bytes(&bytes);
        assert_eq!(header.major_version, expected.0);
        assert_eq!(header.minor_version, expected.1);
        assert_eq!(header.number_of_books, expected.2);
        assert_eq!(header.version_abbrev.as_str(), expected.3);
        let bytes_out = header.to_bytes();
        assert_eq!(hex_header, hex::encode_upper(&bytes_out));
    }

    #[test]
    fn test_barfileheader() {
        test_header(NIV_HEADER, (2, 0, 66, "ZLIB"));
    }

    #[test]
    fn test_esvheader() {
        test_header(ESV_HEADER, (2, 1, 66, "ESV"));
    }

    #[test]
    fn test_niv_v1_header() {
        test_header(NIV_V1_HEADER, (1, 0, 66, "NIV"));
    }

    #[test]
    fn test_greek_header() {
        test_header(GREEK_HEADER, (2, 1, 66, "greek"));
    }

    #[test]
    fn test_read_from() {
        let bytes = hex::decode(NIV_HEADER).expect("Covert to bytes failed.");
        let mut buf = io::Cursor::new(bytes);
        let header = BARFileHeader::read_from(&mut buf).expect("Failed to read from Cursor");
        assert_eq!(header.major_version, 2);
        assert_eq!(header.minor_version, 0);
        assert_eq!(header.number_of_books, 66);
        assert_eq!(header.version_abbrev.as_str(), "ZLIB");
    }

    #[test]
    fn test_write_to() {
        let mut writer = io::Cursor::new(Vec::<u8>::new());
        let version_abbrev = String::from("ZLIB");
        let mut leader: [u8; 3] = [0; 3];
        leader.copy_from_slice("BAR".as_bytes());
        let header = BARFileHeader {
            leader,
            major_version: 2,
            minor_version: 0,
            number_of_books: 66,
            version_abbrev,
        };
        header
            .write_to(&mut writer)
            .expect("Could not write to Cursor");
        writer.rewind().expect("Could not rewind Cursor");
        let mut buf = [0; 16];
        let size = writer
            .read(&mut buf[..])
            .expect("Could not read from cursor");
        assert!(size == 16);
        let hex_output = hex::encode_upper(buf);
        assert_eq!(NIV_HEADER, hex_output.as_str());
    }

    #[test]
    fn test_create_in_memory() {
        let mut buf: Vec<u8> = Vec::new();
        let version_abbrev = String::from("NIV");
        let bar = BARFile::create_in_memory(&mut buf, version_abbrev);
        assert_eq!(bar.header.major_version, 2);
        assert_eq!(bar.header.minor_version, 2);
        assert_eq!(bar.header.number_of_books, 66);
        assert_eq!(bar.header.version_abbrev.as_str(), "NIV");
        assert_eq!(bar.archive_version().to_string().as_str(), "2.2");
        assert_eq!(bar.bible_version().as_str(), "NIV");
        assert_eq!(bar.book_index.len(), 66);
        assert_eq!(bar.number_of_books(), 0);
        assert_eq!(bar.book_capacity(), 66);
        for index in bar.book_index {
            assert!(matches!(index, BARBookIndexEntry::Empty));
        }
    }

    #[test]
    fn test_read_from_memory() {
        let mut buf: Vec<u8> = Vec::new();
        let version_abbrev = String::from("NIV");
        {
            let _bar = BARFile::create_in_memory(&mut buf, version_abbrev);
        }
        let bar = BARFile::open_from_memory(&mut buf);
        assert_eq!(bar.header.major_version, 2);
        assert_eq!(bar.header.minor_version, 2);
        assert_eq!(bar.header.number_of_books, 66);
        assert_eq!(bar.header.version_abbrev.as_str(), "NIV");
        assert_eq!(bar.archive_version().to_string().as_str(), "2.2");
        assert_eq!(bar.bible_version().as_str(), "NIV");
        assert_eq!(bar.book_index.len(), 66);
        assert_eq!(bar.number_of_books(), 0);
        assert_eq!(bar.book_capacity(), 66);
        for index in bar.book_index {
            assert!(matches!(index, BARBookIndexEntry::Empty));
        }
    }
}
