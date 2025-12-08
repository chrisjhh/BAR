use bar::binarystruct::BinaryStruct;
use bar::{BARBookIndexEntry, BARFile};
use std::{env, io};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_path = &args[1];
        let bar = BARFile::open(file_path).expect("Failed to open");
        let hex_output = format!("{:?}", bar.header.to_bytes());
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
        for book in bar.books() {
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
                    let _data = chapt.chapter_text().unwrap();
                    //println!("{}", chapt.first_block().unwrap().decompress())
                }
            }
        }
    }

    let bar =
        BARFile::open(r"C:\Users\hamer-c\OneDrive\Backup\Bible\niv.bar").unwrap_or_else(|_err| {
            // ALternative on linux
            BARFile::open(r"/home/chris/data/bible/NIV.bar").unwrap()
        });
    let ge = bar.book(1).unwrap();
    let chapt1 = ge.chapter(1).unwrap();
    let text = chapt1.chapter_text().unwrap();
    println!("{text}");

    let bar = BARFile::open(r"C:\Users\hamer-c\OneDrive\Backup\Bible\niv_v1.bar").unwrap_or_else(
        |_err| {
            // ALternative on linux
            BARFile::open(r"/home/chris/data/bible/niv_v1.bar").unwrap()
        },
    );
    let ge = bar.book(1).unwrap();
    let chapt1 = ge.chapter(1).unwrap();
    let text = chapt1.chapter_text().unwrap();
    println!("{text}");

    let ps119 = bar.book_from_abbrev("Ps").unwrap().chapter(119).unwrap();

    let verse = ps119.verse_text(105).unwrap();
    println!("{verse}");

    let num_verses = ps119.number_of_verses().unwrap();
    println!("Psalm 119 has {num_verses} verses.");

    for (i, verse) in ps119.verses().enumerate() {
        if verse.contains("word") {
            println!("Ps 119:{} {}", i + 1, verse);
        }
    }

    for book in bar.books() {
        for chapter in book.chapters() {
            if let Some(chapter) = chapter {
                let mut count = 0;
                for (_i, verse) in chapter.verses().enumerate() {
                    if verse.contains("seven") {
                        count += 1;
                        /*println!(
                            "{} {}:{} {}",
                            book.book_abbrev(),
                            chapter.chapter_number(),
                            i + 1,
                            verse
                        )*/
                    }
                }
                if count > 5 {
                    println!(
                        "{} {} : {} times",
                        book.book_abbrev(),
                        chapter.chapter_number(),
                        count
                    )
                }
            }
        }
    }

    Ok(())
}
