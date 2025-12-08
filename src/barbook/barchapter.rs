use crate::BinaryStruct;
use crate::error::{BARFileError, BARResult};
use compress::CompressionError;
use std::cell::RefCell;
use std::io;
use std::rc::Rc;

mod rcsubstring;
use rcsubstring::RcSubstring;

#[derive(Debug, Clone)]
pub enum CompressionAlgorithm {
    None,
    Lzo,
    ZLib,
    GZip,
    Unknown,
}
impl From<u8> for CompressionAlgorithm {
    fn from(value: u8) -> Self {
        match value {
            0 => CompressionAlgorithm::None,
            1 => CompressionAlgorithm::Lzo,
            2 => CompressionAlgorithm::ZLib,
            3 => CompressionAlgorithm::GZip,
            _ => CompressionAlgorithm::Unknown,
        }
    }
}
impl Into<u8> for &CompressionAlgorithm {
    fn into(self) -> u8 {
        match self {
            CompressionAlgorithm::None => 0,
            CompressionAlgorithm::Lzo => 1,
            CompressionAlgorithm::ZLib => 2,
            CompressionAlgorithm::GZip => 3,
            CompressionAlgorithm::Unknown => 255,
        }
    }
}
impl ToString for CompressionAlgorithm {
    fn to_string(&self) -> String {
        match self {
            CompressionAlgorithm::None => "None".to_string(),
            CompressionAlgorithm::Lzo => "LZO".to_string(),
            CompressionAlgorithm::ZLib => "ZLIB".to_string(),
            CompressionAlgorithm::GZip => "GZip".to_string(),
            CompressionAlgorithm::Unknown => "Unknown".to_string(),
        }
    }
}

#[allow(dead_code)]
struct BlockHeaderV2 {
    chapter_number: u8,
    start_verse: u8,
    end_verse: u8,
    compression_algorithm: CompressionAlgorithm,
    block_size: u32,
}

impl BinaryStruct for BlockHeaderV2 {
    fn byte_size() -> usize {
        8
    }

    fn from_bytes(buf: &[u8]) -> Self {
        crate::check_size!(buf);
        let chapter_number = buf[0];
        let start_verse = buf[1];
        let end_verse = buf[2];
        let compression_algorithm: CompressionAlgorithm = buf[3].into();
        let mut bytes: [u8; 4] = [0; 4];
        bytes.copy_from_slice(&buf[4..8]);
        let block_size = u32::from_le_bytes(bytes);
        BlockHeaderV2 {
            chapter_number,
            start_verse,
            end_verse,
            compression_algorithm,
            block_size,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        result.push(self.chapter_number);
        result.push(self.start_verse);
        result.push(self.end_verse);
        let compression_algorithm = &self.compression_algorithm;
        result.push(compression_algorithm.into());
        for byte in self.block_size.to_le_bytes() {
            result.push(byte);
        }
        result
    }
}

#[allow(dead_code)]
struct BlockHeaderV1 {
    chapter_number: u8,
    start_verse: u8,
    end_verse: u8,
    block_size: u32,
}

impl BinaryStruct for BlockHeaderV1 {
    fn byte_size() -> usize {
        7
    }

    fn from_bytes(buf: &[u8]) -> Self {
        crate::check_size!(buf);
        let chapter_number = buf[0];
        let start_verse = buf[1];
        let end_verse = buf[2];
        let mut bytes: [u8; 4] = [0; 4];
        bytes.copy_from_slice(&buf[3..7]);
        let block_size = u32::from_le_bytes(bytes);
        BlockHeaderV1 {
            chapter_number,
            start_verse,
            end_verse,
            block_size,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        result.push(self.chapter_number);
        result.push(self.start_verse);
        result.push(self.end_verse);
        for byte in self.block_size.to_le_bytes() {
            result.push(byte);
        }
        result
    }
}

macro_rules! header_value {
    ($self:ident, $value:ident) => {
        match $self {
            BlockHeader::Ver1(header) => header.$value,
            BlockHeader::Ver2(header) => header.$value,
        }
    };
}

#[allow(dead_code)]
enum BlockHeader {
    Ver1(BlockHeaderV1),
    Ver2(BlockHeaderV2),
}
impl BlockHeader {
    fn block_size(&self) -> u32 {
        header_value!(self, block_size)
    }
    fn start_verse(&self) -> u8 {
        header_value!(self, start_verse)
    }
    fn end_verse(&self) -> u8 {
        header_value!(self, end_verse)
    }
    fn chapter_number(&self) -> u8 {
        header_value!(self, chapter_number)
    }
    fn header_size(&self) -> usize {
        match self {
            BlockHeader::Ver1(_) => BlockHeaderV1::byte_size(),
            BlockHeader::Ver2(_) => BlockHeaderV2::byte_size(),
        }
    }
}

#[allow(dead_code)]
struct BARBlock<T> {
    reader: Rc<RefCell<T>>,
    header: BlockHeader,
    file_offset: u32,
    text: RefCell<Option<Rc<String>>>,
    is_known_last: RefCell<bool>,
}
#[allow(dead_code)]
impl<T: io::Read + io::Seek> BARBlock<T> {
    fn build(shared_reader: Rc<RefCell<T>>, file_offset: u32, file_version: u8) -> BARResult<Self> {
        let reader = &mut *shared_reader.borrow_mut();
        reader.seek(io::SeekFrom::Start(u64::from(file_offset)))?;
        let header: BlockHeader = match file_version {
            1 => BlockHeader::Ver1(BlockHeaderV1::read_from(reader)?),
            2 => BlockHeader::Ver2(BlockHeaderV2::read_from(reader)?),
            _ => {
                return Err(CompressionError(
                    CompressionAlgorithm::Unknown,
                    "Unsuported Compression Algorithm".to_string(),
                )
                .into());
            }
        };
        Ok(BARBlock {
            reader: Rc::clone(&shared_reader),
            header,
            file_offset,
            text: RefCell::new(None),
            is_known_last: RefCell::new(false),
        })
    }

    fn data(&self) -> BARResult<Vec<u8>> {
        let reader = &mut *self.reader.borrow_mut();
        let header_size = match &self.header {
            BlockHeader::Ver1(_) => BlockHeaderV1::byte_size(),
            BlockHeader::Ver2(_) => BlockHeaderV2::byte_size(),
        };
        let file_offset = self.file_offset as usize + header_size;
        reader.seek(io::SeekFrom::Start(file_offset as u64))?;
        let mut buf: Vec<u8> = Vec::new();
        let data_size = match &self.header {
            BlockHeader::Ver1(header) => header.block_size,
            BlockHeader::Ver2(header) => header.block_size,
        };
        buf.resize(data_size as usize, b'\0');
        reader.read_exact(&mut buf[..])?;
        Ok(buf)
    }

    fn compression_algorith(&self) -> &CompressionAlgorithm {
        match &self.header {
            BlockHeader::Ver1(..) => &CompressionAlgorithm::Lzo,
            BlockHeader::Ver2(header) => &header.compression_algorithm,
        }
    }

    fn decompress(&self) -> BARResult<String> {
        let data = self.data()?;
        match self.compression_algorith() {
            CompressionAlgorithm::None => Ok(compress::none::decompress(&data)?),
            CompressionAlgorithm::Lzo => Ok(compress::lzo::decompress(&data)?),
            CompressionAlgorithm::GZip => Ok(compress::gzip::decompress(&data)?),
            CompressionAlgorithm::ZLib => Ok(compress::zlib::decompress(&data)?),
            CompressionAlgorithm::Unknown => Err(compress::CompressionError(
                CompressionAlgorithm::Unknown,
                "Unsupported compression algorithm".to_string(),
            )
            .into()),
        }
    }

    fn text(&self) -> BARResult<Rc<String>> {
        if self.text.borrow().is_none() {
            *self.text.borrow_mut() = Some(Rc::new(self.decompress()?));
        }
        Ok(Rc::clone(&self.text.borrow_mut().as_ref().unwrap()))
    }

    fn start_verse(&self) -> u8 {
        self.header.start_verse()
    }

    fn end_verse(&self) -> u8 {
        self.header.end_verse()
    }

    fn file_version(&self) -> u8 {
        match self.header {
            BlockHeader::Ver1(_) => 1,
            BlockHeader::Ver2(_) => 2,
        }
    }

    fn next_block(&self) -> BARResult<Option<Self>> {
        // Check if we already know we are the last block
        if *self.is_known_last.borrow() {
            // We are already at the end
            return Ok(None);
        }
        let file_offset =
            self.file_offset + self.header.header_size() as u32 + self.header.block_size();
        let next = BARBlock::build(Rc::clone(&self.reader), file_offset, self.file_version());
        if let Err(BARFileError::IOError(_)) = next {
            // Check if we are at the end of the file
            let reader = &mut *self.reader.borrow_mut();
            let eof_pos = reader.seek(io::SeekFrom::End(0))?;
            if file_offset as u64 + self.header.header_size() as u64 > eof_pos {
                // We reached the end of the file. That is why there was an io::Error
                // Do not treat this as an error. Just return None
                *self.is_known_last.borrow_mut() = true;
                return Ok(None);
            }
        }
        let next = next?;
        if next.header.chapter_number() != self.header.chapter_number() {
            // This is the last block
            // Record that we know as mutch so we don't have to check again
            *self.is_known_last.borrow_mut() = true;
            return Ok(None);
        }
        Ok(Some(next))
    }
}

#[allow(dead_code)]
pub struct BARChapter<T> {
    reader: Rc<RefCell<T>>,
    book_number: u8,
    chapter_number: u8,
    file_version: u8,
    file_offset: u32,
    current_block: RefCell<Option<BARBlock<T>>>,
}

#[allow(dead_code)]
impl<T: io::Read + io::Seek> BARChapter<T> {
    pub fn build(
        shared_reader: Rc<RefCell<T>>,
        book_number: u8,
        chapter_number: u8,
        file_offset: u32,
        file_version: u8,
    ) -> BARResult<Self> {
        Ok(BARChapter {
            reader: shared_reader,
            book_number,
            chapter_number,
            file_version,
            file_offset,
            current_block: RefCell::new(None),
        })
    }

    pub fn chapter_number(&self) -> u8 {
        self.chapter_number
    }

    pub fn book_number(&self) -> u8 {
        self.book_number
    }

    pub fn chapter_text(&self) -> BARResult<Rc<String>> {
        self.fetch_first_block()?;
        let mut result = self.current_block.borrow().as_ref().unwrap().text()?;
        while self.fetch_next_block()? {
            Rc::make_mut(&mut result)
                .push_str(&self.current_block.borrow().as_ref().unwrap().text()?);
        }
        Ok(result)
    }

    pub fn verse_text(&self, num: u32) -> BARResult<String> {
        if self.current_block.borrow().is_none()
            || self.current_block.borrow().as_ref().unwrap().start_verse() as u32 > num
        {
            self.fetch_first_block()?;
        }
        while u32::from(self.current_block.borrow().as_ref().unwrap().end_verse()) < num {
            let ok = self.fetch_next_block()?;
            if !ok {
                let book = super::BOOK_NAMES[self.book_number as usize - 1];
                return Err(BARFileError::ReferenceError(format!(
                    "Could not retrieve verse {} for chapter {} in {}",
                    num, self.chapter_number, book
                )));
            }
        }
        let index = num - self.current_block.borrow().as_ref().unwrap().start_verse() as u32;
        let verse = match self
            .current_block
            .borrow()
            .as_ref()
            .unwrap()
            .text()?
            .lines()
            .nth(index as usize)
        {
            None => None,
            Some(str) => Some(str.to_owned()),
        };

        match verse {
            None => Err(BARFileError::InvalidFileFormat(
                "Unable to get verse from block that should have contained it".to_string(),
            )),
            Some(text) => Ok(text),
        }
    }

    pub fn verses<'a>(&'a self) -> BARChapterIterator<'a, T> {
        BARChapterIterator {
            chapter: &self,
            block: None,
            text: None,
            newline_pos: 0,
        }
    }

    fn first_block(&self) -> BARResult<BARBlock<T>> {
        BARBlock::build(Rc::clone(&self.reader), self.file_offset, self.file_version)
    }

    fn fetch_first_block(&self) -> BARResult<()> {
        if self.current_block.borrow().is_none()
            || self.current_block.borrow().as_ref().unwrap().start_verse() != 0u8
        {
            *self.current_block.borrow_mut() = Some(self.first_block()?);
        }
        Ok(())
    }

    fn fetch_next_block(&self) -> BARResult<bool> {
        let next = self.current_block.borrow().as_ref().unwrap().next_block()?;
        if next.is_none() {
            return Ok(false);
        }
        *self.current_block.borrow_mut() = next;
        Ok(true)
    }

    pub fn number_of_verses(&self) -> BARResult<u8> {
        if self.current_block.borrow().is_none() {
            self.fetch_first_block()?;
        }
        while self.fetch_next_block()? {}
        Ok(self.current_block.borrow().as_ref().unwrap().end_verse())
    }
}

#[allow(dead_code)]
pub struct BARChapterIterator<'a, T> {
    chapter: &'a BARChapter<T>,
    block: Option<BARBlock<T>>,
    text: Option<Rc<String>>,
    newline_pos: usize,
}
impl<'a, T: io::Seek + io::Read> Iterator for BARChapterIterator<'a, T> {
    type Item = RcSubstring;

    fn next(&mut self) -> Option<Self::Item> {
        if self.block.is_none() {
            // We don't have a block
            // See if we can take the first block from the chapter
            if self.chapter.current_block.borrow().is_some()
                && self
                    .chapter
                    .current_block
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .start_verse()
                    == 0
            {
                self.block = self.chapter.current_block.borrow_mut().take();
            } else {
                // Chapter doesn't have the first block
                // Fetch it
                let block = self.chapter.first_block();
                if block.is_err() {
                    return None;
                }
                self.block = Some(block.unwrap());
            }
        }
        if self.text.is_none() {
            let text = self.block.as_ref().unwrap().text();
            if text.is_err() {
                return None;
            }
            self.text = Some(text.unwrap());
        }
        let start = self.newline_pos;
        let next_newline = &self.text.as_ref().unwrap()[start..].find("\n");
        if next_newline.is_some() {
            let next_newline = start + next_newline.unwrap();
            self.newline_pos = next_newline + 1;
            return Some(RcSubstring::new(
                Rc::clone(&self.text.as_ref().unwrap()),
                start..next_newline,
            ));
        }
        // Reached the end of this block
        // See if there is another one
        let next = self.block.as_ref().unwrap().next_block();
        match next {
            Err(_) => return None,
            Ok(None) => return None,
            Ok(block @ Some(_)) => self.block = block,
        }
        // Unpack the new text
        let text = self.block.as_ref().unwrap().text();
        if text.is_err() {
            return None;
        }
        self.text = Some(text.unwrap());
        let next_newline = &self.text.as_ref().unwrap().find("\n");
        if next_newline.is_some() {
            let next_newline = next_newline.unwrap();
            let res = RcSubstring::new(Rc::clone(&self.text.as_ref().unwrap()), 0..next_newline);
            self.newline_pos = next_newline + 1;
            return Some(res);
        }
        None
    }
}

#[allow(dead_code)]
mod compress;
