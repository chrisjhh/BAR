struct BARFileHeader {
    major_version: u8,
    minor_version: u8,
    number_of_books: u8,
    version_abbrev: String,
}

impl From<&[u8]> for BARFileHeader {
    fn from(item: &[u8]) -> Self {
        assert!(item.len() == 16);
        let intro = str::from_utf8(&item[0..3]).expect("Not a bar file");
        assert!(intro == "BAR");
        let major_version = item[3];
        let minor_version = item[4];
        let number_of_books = item[5];
        let version_abbrev = String::from_utf8(item[6..16].to_vec())
            .expect("Could not read version abbrev")
            .trim_end_matches("\0")
            .to_string();
        BARFileHeader {
            major_version,
            minor_version,
            number_of_books,
            version_abbrev,
        }
    }
}

pub fn to_hex_string(buf: &[u8]) -> String {
    let result: String = buf.iter().map(|b| format!("{:02X}", b)).collect();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const NIV_HEADER: &str = "4241520200425A4C4942000000000000";

    #[test]
    fn test_to_hex_string() {
        let bytes: &[u8] = &[
            0x42, 0x41, 0x52, 0x02, 0x00, 0x42, 0x5A, 0x4C, 0x49, 0x42, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        let hex_string = to_hex_string(bytes);
        assert_eq!(hex_string.as_str(), NIV_HEADER);
        assert_eq!(hex_string, hex::encode_upper(bytes));
    }

    #[test]
    fn test_barfileheader() {
        let bytes = hex::decode(NIV_HEADER).expect("Covert to bytes failed.");
        let header: BARFileHeader = bytes[..].into();
        assert_eq!(header.major_version, 2);
        assert_eq!(header.minor_version, 0);
        assert_eq!(header.number_of_books, 66);
        assert_eq!(header.version_abbrev.as_str(), "ZLIB");
    }
}
