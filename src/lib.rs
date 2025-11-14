use std::error::Error;

#[allow(dead_code)]
struct BARFileHeader {
    major_version: u8,
    minor_version: u8,
    number_of_books: u8,
    version_abbrev: String,
}

#[allow(dead_code)]
impl BARFileHeader {
    fn from_bytes(buf: &[u8]) -> Result<BARFileHeader, Box<dyn Error>> {
        if buf.len() != 16 {
            return Err("Buffer should be 16 bytes long.".into());
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
        Ok(BARFileHeader {
            major_version,
            minor_version,
            number_of_books,
            version_abbrev,
        })
    }

    fn to_bytes(self: Self) -> Vec<u8> {
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

#[cfg(test)]
mod tests {
    use super::*;

    const NIV_HEADER: &str = "4241520200425A4C4942000000000000";

    #[test]
    fn test_barfileheader() {
        let bytes = hex::decode(NIV_HEADER).expect("Covert to bytes failed.");
        let header = BARFileHeader::from_bytes(&bytes).expect("Construction from bytes failed");
        assert_eq!(header.major_version, 2);
        assert_eq!(header.minor_version, 0);
        assert_eq!(header.number_of_books, 66);
        assert_eq!(header.version_abbrev.as_str(), "ZLIB");
        let bytes_out = header.to_bytes();
        assert_eq!(NIV_HEADER, hex::encode_upper(&bytes_out));
    }
}
