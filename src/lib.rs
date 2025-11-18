use std::error::Error;
use std::fs::File;
use std::io;

#[allow(dead_code)]
trait BinaryStruct {
    fn byte_size() -> usize;
    fn from_bytes(buf: &[u8]) -> Result<Box<Self>, Box<dyn Error>>;
    fn to_bytes(&self) -> Vec<u8>;

    fn read_from(reader: &mut impl io::Read) -> Result<Box<Self>, Box<dyn Error>> {
        let mut buf: Vec<u8> = Vec::new();
        buf.resize(Self::byte_size(), b'\0');
        match reader.read_exact(&mut buf[..]) {
            Err(e) => return Err(format!("{e}").into()),
            _ => {}
        }
        Ok(Self::from_bytes(&buf)?)
    }

    fn write_to(&self, writer: &mut impl io::Write) -> Result<(), Box<dyn Error>> {
        let bytes = self.to_bytes();
        Ok(writer.write_all(&bytes)?)
    }
}

#[allow(dead_code)]
struct BARFileHeader {
    major_version: u8,
    minor_version: u8,
    number_of_books: u8,
    version_abbrev: String,
}

#[allow(dead_code)]
pub struct BarFile {
    file: File,
    header: Box<BARFileHeader>,
}

#[allow(dead_code)]
impl BinaryStruct for BARFileHeader {
    fn byte_size() -> usize {
        16
    }

    fn from_bytes(buf: &[u8]) -> Result<Box<Self>, Box<dyn Error>> {
        if buf.len() != Self::byte_size() {
            return Err(format!("Buffer should be {} bytes long.", Self::byte_size()).into());
        }
        let intro = str::from_utf8(&buf[0..3])?;
        if intro != "BAR" {
            return Err("Not a BAR file header.".into());
        }
        let major_version = buf[3];
        let minor_version = buf[4];
        let number_of_books = buf[5];
        let version_abbrev = str::from_utf8(&buf[6..16])?
            .trim_end_matches("\0")
            .to_string();
        Ok(Box::new(BARFileHeader {
            major_version,
            minor_version,
            number_of_books,
            version_abbrev,
        }))
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

#[allow(dead_code)]
impl BarFile {
    pub fn open(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(file_path)?;
        let header = BARFileHeader::read_from(&mut file)?;
        Ok(Self { file, header })
    }

    pub fn create(file_path: &str, version_abbrev: String) -> Result<Self, Box<dyn Error>> {
        let mut file = File::create_new(file_path)?;
        let header = BARFileHeader {
            major_version: 2,
            minor_version: 0,
            number_of_books: 66,
            version_abbrev,
        };
        header.write_to(&mut file)?;
        Ok(Self {
            file,
            header: Box::new(header),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Seek};

    use super::*;

    const NIV_HEADER: &str = "4241520200425A4C4942000000000000";
    const ESV_HEADER: &str = "42415202014245535600000000000000";
    const NIV_V1_HEADER: &str = "4241520100424E495600000000000000";
    const GREEK_HEADER: &str = "424152020142677265656B0000000000";

    fn test_header(hex_header: &str, expected: (u8, u8, u8, &str)) {
        let bytes = hex::decode(hex_header).expect("Covert to bytes failed.");
        let header = BARFileHeader::from_bytes(&bytes).expect("Construction from bytes failed");
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
        let header = BARFileHeader {
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
}
