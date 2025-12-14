use biblearchive::{self, BARFile};
use crc32fast;

#[test]
fn test_barfile() {
    // Load the testdata
    let bar =
        BARFile::open("tests/data/KJV.ibar").expect("Failed to load KJV.ibar from tests/data");
    assert_eq!(bar.archive_version().0, 2);
    assert_eq!(bar.archive_version().1, 1);
    assert_eq!(bar.archive_version().to_string(), "2.1".to_string());
    assert_eq!(bar.bible_version(), "KJV");
    assert_eq!(bar.number_of_books(), 3);
    assert_eq!(bar.book_capacity(), 66);
    let mut books = bar.books();
    let book = books.next();
    assert!(book.is_some());
    assert_eq!(book.as_ref().unwrap().book_number(), 27u8);
    assert_eq!(book.as_ref().unwrap().book_abbrev(), "Da");
    assert_eq!(book.as_ref().unwrap().book_name(), "Daniel");
    assert_eq!(book.as_ref().unwrap().number_of_chapters(), 12);
    let chapt = book.as_ref().unwrap().chapter(1);
    assert!(chapt.is_some());
    assert_eq!(chapt.as_ref().unwrap().chapter_number(), 1);
    assert_eq!(
        chapt.as_ref().unwrap().book_number(),
        book.as_ref().unwrap().book_number()
    );
    let chapt = book.as_ref().unwrap().chapter(2);
    assert!(chapt.is_none());
    let book = books.next();
    assert!(book.is_some());
    assert_eq!(book.as_ref().unwrap().book_number(), 1u8);
    assert_eq!(book.as_ref().unwrap().book_abbrev(), "Ge");
    assert_eq!(book.as_ref().unwrap().book_name(), "Genesis");
    assert_eq!(book.as_ref().unwrap().number_of_chapters(), 50);
    let chapt = book.as_ref().unwrap().chapter(1);
    assert!(chapt.is_some());
    assert_eq!(chapt.as_ref().unwrap().chapter_number(), 1);
    assert_eq!(
        chapt.as_ref().unwrap().book_number(),
        book.as_ref().unwrap().book_number()
    );
    let chapt = book.as_ref().unwrap().chapter(2);
    assert!(chapt.is_none());
    let book = books.next();
    assert!(book.is_some());
    assert_eq!(book.as_ref().unwrap().book_number(), 49u8);
    assert_eq!(book.as_ref().unwrap().book_abbrev(), "Eph");
    assert_eq!(book.as_ref().unwrap().book_name(), "Ephesians");
    assert_eq!(book.as_ref().unwrap().number_of_chapters(), 6);
    let chapt = book.as_ref().unwrap().chapter(4);
    assert!(chapt.is_some());
    assert_eq!(chapt.as_ref().unwrap().chapter_number(), 4);
    assert_eq!(
        chapt.as_ref().unwrap().book_number(),
        book.as_ref().unwrap().book_number()
    );
    let chapt = book.as_ref().unwrap().chapter(5);
    assert!(chapt.is_none());
    let book = books.next();
    assert!(book.is_none());
    assert_eq!(
        bar.book(1).expect("Failed to get book 1").book_abbrev(),
        "Ge"
    );
    assert_eq!(
        bar.book(27).expect("Failed to get book 27").book_abbrev(),
        "Da"
    );
    assert_eq!(
        bar.book(49).expect("Failed to get book 49").book_abbrev(),
        "Eph"
    );
    assert!(bar.book(2).is_none());
}

#[test]
fn test_iterators() {
    let mut output: Vec<String> = Vec::new();
    let bar =
        BARFile::open("tests/data/KJV.ibar").expect("Failed to load KJV.ibar from tests/data");
    for book in bar.books() {
        output.push(book.book_name().to_string());
        output.push(format!("Chapters: {}", book.number_of_chapters()));
        for chapter in book.chapters() {
            if chapter.is_some() {
                output.push(format!("- Chapter {}", chapter.unwrap().chapter_number()));
            }
            assert_eq!(book.chapters().count(), book.number_of_chapters() as usize);
        }
    }
    assert_eq!(
        output,
        vec!(
            "Daniel",
            "Chapters: 12",
            "- Chapter 1",
            "Genesis",
            "Chapters: 50",
            "- Chapter 1",
            "Ephesians",
            "Chapters: 6",
            "- Chapter 4"
        )
    );
}

#[test]
fn test_into_iterator() {
    let bar =
        BARFile::open("tests/data/KJV.ibar").expect("Failed to load KJV.ibar from tests/data");
    let mut count = 0;
    // Implicitly convert BARFile into iterator by using in for loop
    // This consumes the BARFile and it cannot be used afterwards.
    // Unlike with the iterator returned by books()
    for _book in bar {
        count += 1;
    }
    assert_eq!(count, 3);
}

#[test]
fn test_verses() {
    let bar =
        BARFile::open("tests/data/KJV.ibar").expect("Failed to load KJV.ibar from tests/data");

    let ge = bar.book_from_abbrev("Ge").unwrap();
    let chapt1 = ge.chapter(1).unwrap();
    let verse = chapt1.verse_text(27).unwrap();
    assert_eq!(
        verse,
        "So God created man in his own image, in the image of God created he him; male and female created he them."
    );

    let da = bar.book_from_abbrev("Da").unwrap();
    let chapt1 = da.chapter(1).unwrap();
    let verse = chapt1.verse_text(21).unwrap();
    assert_eq!(
        verse,
        "And Daniel continued even unto the first year of king Cyrus."
    );

    let eph = bar.book_from_abbrev("Eph").unwrap();
    let chapt4 = eph.chapter(4).unwrap();
    let verse = chapt4.verse_text(11).unwrap();
    assert_eq!(
        verse,
        "And he gave some, apostles; and some, prophets; and some, evangelists; and some, pastors and teachers;"
    );

    let verse = chapt4.verse_text(33);
    assert!(verse.is_err());
}

#[test]
fn test_chapter_text() {
    let bar =
        BARFile::open("tests/data/KJV.ibar").expect("Failed to load KJV.ibar from tests/data");

    let ge = bar.book_from_abbrev("Ge").unwrap();
    let chapt1 = ge.chapter(1).unwrap();

    let text = chapt1.chapter_text().unwrap();
    assert_eq!(crc32fast::hash(text.as_bytes()), 2672530595);

    let da = bar.book_from_abbrev("Da").unwrap();
    let chapt1 = da.chapter(1).unwrap();
    let text = chapt1.chapter_text().unwrap();
    assert_eq!(crc32fast::hash(text.as_bytes()), 1895967111);

    let eph = bar.book_from_abbrev("Eph").unwrap();
    let chapt4 = eph.chapter(4).unwrap();
    let text = chapt4.chapter_text().unwrap();
    assert_eq!(crc32fast::hash(text.as_bytes()), 397479874);
}

#[test]
fn test_number_of_verses() {
    let bar =
        BARFile::open("tests/data/KJV.ibar").expect("Failed to load KJV.ibar from tests/data");

    let ge = bar.book_from_abbrev("Ge").unwrap();
    let chapt1 = ge.chapter(1).unwrap();
    assert_eq!(chapt1.number_of_verses().unwrap(), 31);

    let da = bar.book_from_abbrev("Da").unwrap();
    let chapt1 = da.chapter(1).unwrap();
    assert_eq!(chapt1.number_of_verses().unwrap(), 21);

    let eph = bar.book_from_abbrev("Eph").unwrap();
    let chapt4 = eph.chapter(4).unwrap();
    assert_eq!(chapt4.number_of_verses().unwrap(), 32);
}

#[test]
fn test_verse_iterator() {
    let bar =
        BARFile::open("tests/data/KJV.ibar").expect("Failed to load KJV.ibar from tests/data");
    for book in bar.books() {
        for chapter in book.chapters() {
            if let Some(chapter) = chapter {
                for (i, verse) in chapter.verses().enumerate() {
                    if verse.contains("God") {
                        println!(
                            "{} {}:{} {}",
                            book.book_abbrev(),
                            chapter.chapter_number(),
                            i + 1,
                            verse
                        )
                    }
                }
            }
        }
    }
}

#[test]
fn test_books_in_order() {
    // Test the books are returned in the order they occur in the bible
    let bar =
        BARFile::open("tests/data/KJV.ibar").expect("Failed to load KJV.ibar from tests/data");
    let mut it = bar.books_in_order();
    assert_eq!(it.next().unwrap().book_abbrev(), "Ge");
    assert_eq!(it.next().unwrap().book_abbrev(), "Da");
    assert_eq!(it.next().unwrap().book_abbrev(), "Eph");
    assert!(it.next().is_none());
}
