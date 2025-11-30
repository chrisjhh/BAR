use bar::{self, BARFile};

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
        for chapter in book {
            if chapter.is_some() {
                output.push(format!("- Chapter {}", chapter.unwrap().chapter_number()));
            }
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
