use bar::{BARBookIndexEntry, BARFile, BinaryStruct};
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
        let mut book_numbers: Vec<u8> = Vec::new();
        for entry in &bar.book_index {
            match entry {
                BARBookIndexEntry::Live {
                    book_number,
                    file_offset,
                } => {
                    book_numbers.push(*book_number);
                    println!("{} {}", book_number, file_offset);
                }
                BARBookIndexEntry::Empty => break,
            }
        }
        for book in bar {
            println!(
                "Book {} {} ({}). Number of chapters {}.",
                book.book_number(),
                book.book_name(),
                book.book_abbrev(),
                book.number_of_chapters(),
            );
            for i in 0..book.number_of_chapters() {
                let chapter = book.chapter(i);
                if let Some(chapt) = chapter {
                    assert_eq!(i, chapt.chapter_number());
                    assert_eq!(chapt.book_number(), book.book_number());
                    println!("  chapter {i} present");
                    println!("{}", chapt.first_block().unwrap().decompress())
                }
            }
        }
    }
    Ok(())
}
