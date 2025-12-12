/*!
A reference-counted substring

For returning part of a string held in an [Rc] that needs to live longer than the source of the string itself.
For a more complete alternative see [ArcStr](https://crates.io/crates/arcstr). This is intended as a lightweight alternative where the
string is held in an [Rc] rather than an [Arc][std::sync::Arc] and in simple single-threaded situations.

# Example
```rust
# use bar::barbook::barchapter::rcsubstring::RcSubstring;
# use std::rc::Rc;
let shared_text: Rc<String> = Rc::new(String::from("Some text"));
let shared_substring = RcSubstring::new(Rc::clone(&shared_text), 5..9);
drop(shared_text);
assert_eq!(shared_substring, "text");
```
*/
#![warn(missing_docs)]
use std::fmt::{Debug, Display};
use std::ops::{Deref, Range};
use std::rc::Rc;

/**
A reference counted substring

Stores an `Rc<String>` and a range
The deref behaviour means this can be used just like a &str
The advantage is the internal [Rc] handles the memory management so you don't have to worry about borrow lifetimes
Useful for returning parts of a string that should live longer than the struct that returned them
eg. from an iterator over a string stored in the iterator itself
*/

#[derive(Debug)]
pub struct RcSubstring {
    rcstring: Rc<String>,
    range: Range<usize>,
}

impl Display for RcSubstring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.deref())
    }
}

impl PartialEq<&str> for RcSubstring {
    fn eq(&self, other: &&str) -> bool {
        self.deref() == *other
    }
}

#[allow(dead_code)]
impl RcSubstring {
    /// Construct a new RcSubstring
    ///
    /// Takes the `Rc<String>` to wrap and the range for the substring in this text
    ///
    /// # Panics
    ///
    /// Panics if `range` is invalid
    ///  - begin < end
    ///  - either begin or end > length of `Rc<String>` wrapped
    ///
    /// If it didn't panic here it would panic during the slice when the RcSubstring is used
    /// so it is better to catch the issues at source.
    pub fn new(rcstring: Rc<String>, range: Range<usize>) -> Self {
        assert!(
            range.end >= range.start,
            "begin < end ({} < {}) when creating RcSubstring",
            range.start,
            range.end
        );
        assert!(
            range.start <= rcstring.len(),
            "start index {} out of bounds when creating RcSubstring",
            range.start
        );
        assert!(
            range.end <= rcstring.len(),
            "end index {} out of bounds when creating RcSubstring",
            range.end
        );
        RcSubstring { rcstring, range }
    }
}

impl Deref for RcSubstring {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.rcstring[self.range.start..self.range.end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_usage() {
        let text = "Line 1\nLine 2\nLine 3";
        let rcstring = Rc::new(text.to_string());
        let pos = text.find("\n").unwrap();
        let rcsubstring = RcSubstring::new(rcstring.clone(), 0..pos);
        let string_rep = format!("{}", rcsubstring);
        assert_eq!(string_rep, "Line 1");
        let debug_rep = format!("{:?}", rcsubstring);
        assert_eq!(
            debug_rep,
            "RcSubstring { rcstring: \"Line 1\\nLine 2\\nLine 3\", range: 0..6 }"
        );
        let pretty_rep = format!("{:#?}", rcsubstring);
        assert_eq!(
            pretty_rep,
            "RcSubstring {\n    rcstring: \"Line 1\\nLine 2\\nLine 3\",\n    range: 0..6,\n}"
        );
        assert_eq!(&rcsubstring[1..2], "i");
    }

    #[test]
    fn test_intended_usage() {
        struct WordIterator {
            rcstring: Rc<String>,
            start_pos: usize,
        }
        impl Iterator for WordIterator {
            type Item = RcSubstring;
            fn next(&mut self) -> Option<Self::Item> {
                let pos = self.start_pos + self.rcstring[self.start_pos..].find(" ")?;
                let value = RcSubstring::new(Rc::clone(&self.rcstring), self.start_pos..pos);
                self.start_pos = pos + 1;
                return Some(value);
            }
        }

        fn generate_text(values: Vec<usize>) -> String {
            let words = vec!["zero", "one", "two", "three", "four", "five"];
            let mut result = String::new();
            for i in values {
                result.push_str(words[i]);
                result.push_str(" ");
            }
            result
        }

        fn give_me_an_iterator() -> WordIterator {
            let text = generate_text(vec![2, 3, 1, 0, 5]);
            WordIterator {
                rcstring: Rc::new(text),
                start_pos: 0,
            }
        }

        let mut it = give_me_an_iterator();
        assert_eq!(it.next().unwrap(), "two");
        assert_eq!(it.next().unwrap(), "three");
        assert_eq!(it.next().unwrap(), "one");
        assert_eq!(it.next().unwrap(), "zero");
        assert_eq!(it.next().unwrap(), "five");
        assert!(it.next().is_none());
    }

    #[test]
    fn test_empty() {
        let rcsubstring = RcSubstring::new(Rc::new(String::from("Random text")), 3..3);
        assert_eq!(rcsubstring.len(), 0);
        assert_eq!(rcsubstring, "");
    }

    // Test these bad uses panic with our own message - ie. not in some other downstream code

    #[test]
    #[should_panic(expected = "RcSubstring")]
    fn test_end_before_start() {
        let _ = RcSubstring::new(Rc::new(String::from("Random text")), 3..0);
    }

    #[test]
    #[should_panic(expected = "RcSubstring")]
    fn test_start_out_of_range() {
        let _ = RcSubstring::new(Rc::new(String::from("Random text")), 100..101);
    }

    #[test]
    #[should_panic(expected = "RcSubstring")]
    fn test_end_out_of_range() {
        let _ = RcSubstring::new(Rc::new(String::from("Random text")), 0..101);
    }
}
