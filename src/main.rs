use biblearchive::BARFile;
use std::{env, io};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_path = &args[1];
        let bar = BARFile::open(file_path).expect("Failed to open");
        println!("Version {}", bar.archive_version());
        println!("{}", bar.bible_version());

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

    let bar =
        BARFile::open(r"C:\Users\hamer-c\OneDrive\Backup\Bible\ESV.ibar").unwrap_or_else(|_err| {
            // ALternative on linux
            BARFile::open(r"/home/chris/data/bible/niv_v1.bar").unwrap()
        });
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

    for book in bar.books_in_order() {
        //println!("{}", book.book_name());
        for chapter in book.chapters().flatten() {
            let mut count = 0;
            for verse in chapter.verses() {
                if verse.contains("seven") || verse.contains("Seven") {
                    count += 1;
                }
            }
            if count > 4 {
                println!(
                    "{} {} : {} times",
                    book.book_abbrev(),
                    chapter.chapter_number(),
                    count
                )
            }
        }
    }

    let ps23 = bar.book_from_abbrev("Ps").unwrap().chapter(23).unwrap();
    for (i, verse) in ps23.enumerated_verses() {
        println!("{} {}", i, verse);
    }

    Ok(())
}
