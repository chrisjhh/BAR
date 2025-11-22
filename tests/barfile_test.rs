use bar::{self, BARFile};

#[test]
fn test_barfile() {
    // Load the testdata
    let mut bar =
        BARFile::open("tests/data/KJV.ibar").expect("Failed to load KJV.ibar from tests/data");
    assert_eq!(bar.archive_version().0, 2);
    assert_eq!(bar.archive_version().1, 1);
    assert_eq!(bar.archive_version().to_string(), "2.1".to_string());
    assert_eq!(bar.bible_version(), "KJV");
    let book = &bar.next();
    assert!(book.is_some());
    assert_eq!(book.as_ref().unwrap().book_number(), 27u8);
    assert_eq!(book.as_ref().unwrap().book_abbrev(), "Da");
    assert_eq!(book.as_ref().unwrap().book_name(), "Daniel");
    assert_eq!(book.as_ref().unwrap().number_of_chapters(), 12);
    let book = &bar.next();
    assert!(book.is_some());
    assert_eq!(book.as_ref().unwrap().book_number(), 1u8);
    assert_eq!(book.as_ref().unwrap().book_abbrev(), "Ge");
    assert_eq!(book.as_ref().unwrap().book_name(), "Genesis");
    assert_eq!(book.as_ref().unwrap().number_of_chapters(), 50);
    let book = &bar.next();
    assert!(book.is_some());
    assert_eq!(book.as_ref().unwrap().book_number(), 49u8);
    assert_eq!(book.as_ref().unwrap().book_abbrev(), "Eph");
    assert_eq!(book.as_ref().unwrap().book_name(), "Ephesians");
    assert_eq!(book.as_ref().unwrap().number_of_chapters(), 6);
    let book = &bar.next();
    assert!(book.is_none());
}
