use crate::BinaryStruct;
use crate::error::BARResult;
use compress::CompressionError;
use std::cell::RefCell;
use std::io;
use std::rc::Rc;

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
}

#[allow(dead_code)]
struct BARBlock<T> {
    reader: Rc<RefCell<T>>,
    header: BlockHeader,
    file_offset: u32,
    text: RefCell<Option<String>>,
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

    pub fn text(&self) -> BARResult<String> {
        if self.text.borrow().is_none() {
            *self.text.borrow_mut() = Some(self.decompress()?)
        }
        Ok(self.text.borrow().as_ref().unwrap().clone())
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
        let file_offset = self.file_offset + self.header.block_size();
        let next = BARBlock::build(Rc::clone(&self.reader), file_offset, self.file_version())?;
        if next.header.chapter_number() != self.header.chapter_number() {
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

    pub fn chapter_text(&self) -> BARResult<String> {
        self.fetch_first_block()?;
        let mut result = self.current_block.borrow().as_ref().unwrap().text()?;
        while self.fetch_next_block()? {
            result.push_str(&self.current_block.borrow().as_ref().unwrap().text()?);
        }
        Ok(result)
    }

    fn fetch_first_block(&self) -> BARResult<()> {
        if self.current_block.borrow().is_none()
            || self.current_block.borrow().as_ref().unwrap().start_verse() != 0u8
        {
            *self.current_block.borrow_mut() = Some(BARBlock::build(
                Rc::clone(&self.reader),
                self.file_offset,
                self.file_version,
            )?);
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
}

#[allow(dead_code)]
mod compress;
