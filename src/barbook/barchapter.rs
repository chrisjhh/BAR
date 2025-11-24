use crate::BinaryStruct;
use std::cell::RefCell;
use std::error::Error;
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

    pub fn decompress(&self) -> Result<String, Box<dyn std::error::Error>> {
        let data = self.data()?;
        match self.compression_algorith() {
            CompressionAlgorithm::None => Ok(String::from_utf8(data)?),
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
    pub struct CompressionError(pub CompressionAlgorithm, pub String);
    impl fmt::Display for CompressionError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} compression error: {}", self.0.to_string(), self.1)
        }
    }
    impl std::error::Error for CompressionError {}

    pub mod lzo {
        use super::Result;
        use crate::barbook::barchapter::CompressionAlgorithm;
        use lzokay_native;
        const ALGORITHM: CompressionAlgorithm = CompressionAlgorithm::Lzo;
        const MAX_SIZE: u32 = 100 * 1024;
        const FIRST_BYTE: u8 = 241;
        use super::CompressionError;

        pub fn decompress(data: &[u8]) -> Result<String> {
            // First byte is 241 and then decompressed length in bigendian 4-byte format
            let first_byte = data[0];
            if first_byte != FIRST_BYTE {
                return Err(CompressionError(
                    ALGORITHM,
                    format!(
                        "Unexpected first byte [{:X}] expected {:X}",
                        first_byte, FIRST_BYTE
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
            match String::from_utf8(decompressed) {
                Ok(string) => Ok(string),
                Err(err) => {
                    return Err(CompressionError(
                        ALGORITHM,
                        format!("Error parsing bytes into string {}", err),
                    ));
                }
            }
        }

        pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
            let uncompressed_size: u32 = data.len() as u32;
            let result = lzokay_native::compress(data);
            if result.is_err() {
                return Err(CompressionError(
                    ALGORITHM,
                    format!("{}", result.unwrap_err()),
                ));
            }
            let mut compressed = result.unwrap();
            let mut lzo_data: Vec<u8> = Vec::new();
            lzo_data.push(FIRST_BYTE);
            lzo_data.append(&mut uncompressed_size.to_be_bytes().to_vec());
            lzo_data.append(&mut compressed);
            Ok(lzo_data)
        }
    }

    pub mod zlib {
        use super::Result;
        use crate::barbook::barchapter::CompressionAlgorithm;
        use flate2::read::ZlibDecoder;
        const ALGORITHM: CompressionAlgorithm = CompressionAlgorithm::ZLib;
        use super::CompressionError;
        use flate2::Compression;
        use flate2::write::ZlibEncoder;
        use std::io::{Read, Write};

        pub fn decompress(data: &[u8]) -> Result<String> {
            let mut decoder = ZlibDecoder::new(&data[..]);
            let mut decompressed = String::new();
            let result = decoder.read_to_string(&mut decompressed);
            if result.is_err() {
                return Err(CompressionError(
                    ALGORITHM,
                    format!("Decompression error: {}", result.unwrap_err()),
                ));
            }
            Ok(decompressed)
        }

        pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            let result = encoder.write_all(&data);
            if result.is_err() {
                return Err(CompressionError(
                    ALGORITHM,
                    format!("Compression error: {}", result.unwrap_err()),
                ));
            }
            match encoder.finish() {
                Ok(bytes) => Ok(bytes),
                Err(err) => {
                    return Err(CompressionError(
                        ALGORITHM,
                        format!("Compression error: {}", err),
                    ));
                }
            }
        }
    }

    pub mod gzip {
        use super::Result;
        use crate::barbook::barchapter::CompressionAlgorithm;
        use flate2::read::GzDecoder;
        use flate2::write::GzEncoder;
        const ALGORITHM: CompressionAlgorithm = CompressionAlgorithm::GZip;
        use super::CompressionError;
        use flate2::Compression;
        use std::io::{Read, Write};

        pub fn decompress(data: &[u8]) -> Result<String> {
            let mut decoder = GzDecoder::new(&data[..]);
            let mut decompressed = String::new();
            let result = decoder.read_to_string(&mut decompressed);
            if result.is_err() {
                return Err(CompressionError(
                    ALGORITHM,
                    format!("{}", result.unwrap_err()),
                ));
            }
            Ok(decompressed)
        }

        pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            let result = encoder.write_all(&data);
            if result.is_err() {
                return Err(CompressionError(
                    ALGORITHM,
                    format!("Compression error: {}", result.unwrap_err()),
                ));
            }
            match encoder.finish() {
                Ok(bytes) => Ok(bytes),
                Err(err) => {
                    return Err(CompressionError(
                        ALGORITHM,
                        format!("Compression error: {}", err),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA : &str = "In the beginning God created the heaven and the earth.
And the earth was without form, and void; and darkness was upon the face of the deep. And the Spirit of God moved upon the face of the waters.
And God said, Let there be light: and there was light.
And God saw the light, that it was good: and God divided the light from the darkness.
And God called the light Day, and the darkness he called Night. And the evening and the morning were the first day";

    const LZO_DATA: &str = "F1000001C52D496E2074686520626567696E6E696E6720476F6420637265617\
    465648503680A0276650209616EBB016561720B012E0A412A38000F2077617320776974686F757420666F726\
    D2C980502766F69643B8501640409016B6E65735004430575706FAF0E66616302016F668C0103646565702E2\
    0F40A035370697269746C0376116D6F01206434D900775A147273B8116A057361081C022C204C65747503727\
    81A04206C696768743AFC174C026C1090022AD90077980999022C49016100105C0F6605676F0C07B407630564\
    69760018C0239005022066726F6DA418CC1C2854010163616C6C2BAF00446179A4232ABC00402CE5054E94122\
    71504655C2E6432E5066D04569D017774188521690D3E74410979110000";

    const ZLIB_DATA: &str = "789C6D51414EC4300CBCF30A3FA0DA07C009090921212EBCC0346E6BD1C695E36\
    DB4BF2771DAB248DCEC783C9E99BC45B089E08B468E91E308AF12A05742A3E0938970A308185B4BA8365D1E9EE\
    F5BC89820B34D7235184497CEE19B7078F22AA07E474AC981D755DACD017B0219BC0E44EB050ED6CF9595ADCEA\
    A9A45B6A2E5DFB55C646A6A722A3421870EDEC9EA54AB2D98799CECF1D0AFE41AFCF17E2D3B9D3F77A5448372B\
    F224791D0B62B30F0C6610FC6C130A82CCDC1EEF197B5C779FE837DC15B7706796652EA1DF9E1AACE14A8E45E7\
    FE4D85844BDCFD58527C19AAC10DD7E0067919D53";

    const GZIP_DATA: &str = "1F8B08000000000000FF6D51414EC4300CBCF30A3FA0DA07C009090921212EBC\
    C0346E6BD1C695E36DB4BF2771DAB248DCEC783C9E99BC45B089E08B468E91E308AF12A05742A3E0938970A30\
    8185B4BA8365D1E9EEF5BC89820B34D7235184497CEE19B7078F22AA07E474AC981D755DACD017B0219BC0E44\
    EB050ED6CF9595ADCEAA9A45B6A2E5DFB55C646A6A722A3421870EDEC9EA54AB2D98799CECF1D0AFE41AFCF17\
    E2D3B9D3F77A5448372BF224791D0B62B30F0C6610FC6C130A82CCDC1EEF197B5C779FE837DC15B7706796652\
    EA1DF9E1AACE14A8E45E7FE4D85844BDCFD58527C19AAC10DD7E0057BE1A7DC5010000";

    #[test]
    fn test_lzo_compression() {
        let compressed = compress::lzo::compress(&DATA.to_string().into_bytes()).unwrap();
        let hex_string = hex::encode_upper(compressed);
        assert_eq!(hex_string.as_str(), LZO_DATA);
    }

    #[test]
    fn test_lzo_decompression() {
        let data = hex::decode(LZO_DATA).unwrap();
        let decompressed = compress::lzo::decompress(&data).unwrap();
        assert_eq!(decompressed, DATA);
    }

    #[test]
    fn test_zlib_compression() {
        let compressed = compress::zlib::compress(&DATA.to_string().into_bytes()).unwrap();
        let hex_string = hex::encode_upper(compressed);
        assert_eq!(hex_string.as_str(), ZLIB_DATA);
    }

    #[test]
    fn test_zlib_decompression() {
        let data = hex::decode(ZLIB_DATA).unwrap();
        let decompressed = compress::zlib::decompress(&data).unwrap();
        assert_eq!(decompressed, DATA);
    }

    #[test]
    fn test_gzip_compression() {
        let compressed = compress::gzip::compress(&DATA.to_string().into_bytes()).unwrap();
        let hex_string = hex::encode_upper(compressed);
        assert_eq!(hex_string.as_str(), GZIP_DATA);
    }

    #[test]
    fn test_zip_decompression() {
        let data = hex::decode(GZIP_DATA).unwrap();
        let decompressed = compress::gzip::decompress(&data).unwrap();
        assert_eq!(decompressed, DATA);
    }
}
