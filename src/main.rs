use bar::{BARFile, BinaryStruct};
use hex;
use std::{env, io};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_path = &args[1];
        //let mut file = File::open(file_path)?;
        //let mut header = [0; 16];
        //let size = file.read(&mut header[..])?;
        //assert!(size == 16);
        //let hex_output = hex::encode_upper(header);
        let bar = BARFile::open(file_path).expect("Failed to open");
        let hex_output = hex::encode_upper(bar.header.to_bytes());
        println!("{hex_output}");
        println!("Version {}", bar.archive_version());
        println!("{}", bar.bible_version());
        for entry in bar.book_index {
            println!("{} {}", entry.book_number, entry.file_offset)
        }
    }
    Ok(())
}
