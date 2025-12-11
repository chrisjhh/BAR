use super::CompressionAlgorithm;
use std::{fmt, string::FromUtf8Error};
type Result<T> = std::result::Result<T, CompressionError>;
use crate::error::BARFileError;

#[derive(Debug, Clone)]
pub struct CompressionError(pub CompressionAlgorithm, pub String);
impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} compression error: {}", self.0, self.1)
    }
}
impl std::error::Error for CompressionError {}

impl From<FromUtf8Error> for CompressionError {
    fn from(value: FromUtf8Error) -> Self {
        CompressionError(
            CompressionAlgorithm::Unknown,
            format!(
                "Non UTF8 character sequence when decoding String: {}",
                value
            ),
        )
    }
}

impl From<CompressionError> for BARFileError {
    fn from(value: CompressionError) -> Self {
        BARFileError::CompressionError(value.to_string())
    }
}

pub mod none {
    use super::Result;
    pub fn decompress(data: &[u8]) -> Result<String> {
        Ok(String::from_utf8(data.to_vec())?)
    }
}

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

        let result = lzokay_native::decompress_all(&data[5..], Some(decompressed_size as usize));
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
        Ok(String::from_utf8(decompressed)?)
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
        let mut decoder = ZlibDecoder::new(data);
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
        let result = encoder.write_all(data);
        if result.is_err() {
            return Err(CompressionError(
                ALGORITHM,
                format!("Compression error: {}", result.unwrap_err()),
            ));
        }
        match encoder.finish() {
            Ok(bytes) => Ok(bytes),
            Err(err) => Err(CompressionError(
                ALGORITHM,
                format!("Compression error: {}", err),
            )),
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
        let mut decoder = GzDecoder::new(data);
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
        let result = encoder.write_all(data);
        if result.is_err() {
            return Err(CompressionError(
                ALGORITHM,
                format!("Compression error: {}", result.unwrap_err()),
            ));
        }
        match encoder.finish() {
            Ok(bytes) => Ok(bytes),
            Err(err) => Err(CompressionError(
                ALGORITHM,
                format!("Compression error: {}", err),
            )),
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
        let compressed = lzo::compress(&DATA.to_string().into_bytes()).unwrap();
        let hex_string = hex::encode_upper(compressed);
        assert_eq!(hex_string.as_str(), LZO_DATA);
    }

    #[test]
    fn test_lzo_decompression() {
        let data = hex::decode(LZO_DATA).unwrap();
        let decompressed = lzo::decompress(&data).unwrap();
        assert_eq!(decompressed, DATA);
    }

    #[test]
    fn test_zlib_compression() {
        let compressed = zlib::compress(&DATA.to_string().into_bytes()).unwrap();
        let hex_string = hex::encode_upper(compressed);
        assert_eq!(hex_string.as_str(), ZLIB_DATA);
    }

    #[test]
    fn test_zlib_decompression() {
        let data = hex::decode(ZLIB_DATA).unwrap();
        let decompressed = zlib::decompress(&data).unwrap();
        assert_eq!(decompressed, DATA);
    }

    #[test]
    fn test_gzip_compression() {
        let compressed = gzip::compress(&DATA.to_string().into_bytes()).unwrap();
        let hex_string = hex::encode_upper(compressed);
        assert_eq!(hex_string.as_str(), GZIP_DATA);
    }

    #[test]
    fn test_zip_decompression() {
        let data = hex::decode(GZIP_DATA).unwrap();
        let decompressed = gzip::decompress(&data).unwrap();
        assert_eq!(decompressed, DATA);
    }
}
