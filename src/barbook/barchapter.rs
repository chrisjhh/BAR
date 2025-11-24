use crate::BinaryStruct;
use std::cell::RefCell;
use std::error::Error;
use std::io::{self, Read};
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

    fn from_bytes(buf: &[u8]) -> Result<Box<Self>, Box<dyn std::error::Error>> {
        crate::check_size!(buf);
        let chapter_number = buf[0];
        let start_verse = buf[1];
        let end_verse = buf[2];
        let compression_algorithm: CompressionAlgorithm = buf[3].into();
        let mut bytes: [u8; 4] = [0; 4];
        bytes.copy_from_slice(&buf[4..8]);
        let block_size = u32::from_le_bytes(bytes);
        Ok(Box::new(BlockHeaderV2 {
            chapter_number,
            start_verse,
            end_verse,
            compression_algorithm,
            block_size,
        }))
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

    fn from_bytes(buf: &[u8]) -> Result<Box<Self>, Box<dyn std::error::Error>> {
        crate::check_size!(buf);
        let chapter_number = buf[0];
        let start_verse = buf[1];
        let end_verse = buf[2];
        let mut bytes: [u8; 4] = [0; 4];
        bytes.copy_from_slice(&buf[3..7]);
        let block_size = u32::from_le_bytes(bytes);
        Ok(Box::new(BlockHeaderV1 {
            chapter_number,
            start_verse,
            end_verse,
            block_size,
        }))
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

#[allow(dead_code)]
enum BlockHeader {
    Ver1(BlockHeaderV1),
    Ver2(BlockHeaderV2),
}

//TODO: Remove pub
#[allow(dead_code)]
pub struct BARBlock<T> {
    reader: Rc<RefCell<T>>,
    header: BlockHeader,
    file_offset: u32,
    data: Option<Vec<u8>>,
}
#[allow(dead_code)]
impl<T: io::Read + io::Seek> BARBlock<T> {
    pub fn build(
        shared_reader: Rc<RefCell<T>>,
        file_offset: u32,
        file_version: u8,
    ) -> Result<Self, Box<dyn Error>> {
        let reader = &mut *shared_reader.borrow_mut();
        reader.seek(io::SeekFrom::Start(u64::from(file_offset)))?;
        let header: BlockHeader = match file_version {
            1 => BlockHeader::Ver1(*BlockHeaderV1::read_from(reader)?),
            2 => BlockHeader::Ver2(*BlockHeaderV2::read_from(reader)?),
            _ => return Err("Unsuporte file version".into()),
        };
        Ok(BARBlock {
            reader: shared_reader.clone(),
            header,
            file_offset,
            data: None,
        })
    }

    pub fn data(&self) -> Result<Vec<u8>, Box<dyn Error>> {
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

    pub fn decompress(&self) -> String {
        use flate2::read::{GzDecoder, ZlibDecoder};
        let data = self.data().unwrap();
        match self.compression_algorith() {
            CompressionAlgorithm::None => String::from_utf8(data).unwrap(),
            CompressionAlgorithm::Lzo => {
                let decompressed = compress::lzo::decompress(&data).unwrap();
                String::from_utf8(decompressed).unwrap()
            }
            CompressionAlgorithm::GZip => {
                let mut decoder = GzDecoder::new(&data[..]);
                let mut result = String::new();
                decoder.read_to_string(&mut result).unwrap();
                result
            }
            CompressionAlgorithm::ZLib => {
                let mut decoder = ZlibDecoder::new(&data[..]);
                let mut result = String::new();
                decoder.read_to_string(&mut result).unwrap();
                result
            }
            CompressionAlgorithm::Unknown => panic!("Unkown compression algorithm"),
        }
    }
}

#[allow(dead_code)]
pub struct BARChapter<T> {
    reader: Rc<RefCell<T>>,
    book_number: u8,
    chapter_number: u8,
    file_version: u8,
    file_offset: u32,
    current_block: Option<BARBlock<T>>,
}

#[allow(dead_code)]
impl<T: io::Read + io::Seek> BARChapter<T> {
    pub fn build(
        shared_reader: Rc<RefCell<T>>,
        book_number: u8,
        chapter_number: u8,
        file_offset: u32,
        file_version: u8,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(BARChapter {
            reader: shared_reader,
            book_number,
            chapter_number,
            file_version,
            file_offset,
            current_block: None,
        })
    }

    pub fn chapter_number(&self) -> u8 {
        self.chapter_number
    }

    pub fn book_number(&self) -> u8 {
        self.book_number
    }

    //TODO: Remove pub
    pub fn first_block(&self) -> Result<BARBlock<T>, Box<dyn Error>> {
        //TODO: Use current_block
        BARBlock::build(self.reader.clone(), self.file_offset, self.file_version)
    }
}

#[allow(dead_code)]
mod compress {
    use super::CompressionAlgorithm;
    use std::fmt;
    type Result<T> = std::result::Result<T, CompressionError>;

    #[derive(Debug, Clone)]
    pub struct CompressionError(CompressionAlgorithm, String);
    impl fmt::Display for CompressionError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} compression error: {}", self.0.to_string(), self.1)
        }
    }

    pub mod lzo {
        use crate::barbook::barchapter::CompressionAlgorithm;

        use super::Result;
        use lzokay_native;
        const ALGORITHM: CompressionAlgorithm = CompressionAlgorithm::Lzo;
        const MAX_SIZE: u32 = 100 * 1024;
        use super::CompressionError;

        pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
            // First byte is 241 and then decompressed length in bigendian 4-byte format
            let first_byte = data[0];
            if first_byte != 241 {
                return Err(CompressionError(
                    ALGORITHM,
                    format!(
                        "Unexpected first byte [{:X}] expected {:X}",
                        first_byte, 241
                    ),
                ));
            }
            let mut bytes: [u8; 4] = [0; 4];
            bytes.copy_from_slice(&data[1..5]);
            let decompressed_size = u32::from_be_bytes(bytes);
            if decompressed_size == 0 || decompressed_size > MAX_SIZE {
                return Err(CompressionError(
                    ALGORITHM,
                    format!("Unexpected decompression size {}", decompressed_size),
                ));
            }

            let result =
                lzokay_native::decompress_all(&data[5..], Some(decompressed_size as usize));
            if result.is_err() {
                return Err(CompressionError(
                    ALGORITHM,
                    format!("{}", result.unwrap_err()),
                ));
            }
            let decompressed = result.unwrap();
            if decompressed_size as usize != decompressed.len() {
                return Err(CompressionError(
                    ALGORITHM,
                    format!(
                        "Decompressed data was not of expected size: {} expected: {}",
                        decompressed.len(),
                        decompressed_size
                    ),
                ));
            }
            Ok(decompressed)
        }
    }
}
