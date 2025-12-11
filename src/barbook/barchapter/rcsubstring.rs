/*!
A reference-counted substring

For returning part of a string held in an [Rc] that needs to live longer than the structure returning it.
For a more complete alternative see [ArcStr]. This is intended as a lightweight alternative where the
string is held in an [Rc] rather than an [Arc] and in simple single-threaded situations.
*/
#![warn(missing_docs)]
use std::fmt::{Debug, Display};
use std::ops::{Deref, Range};
use std::rc::Rc;

/**
A reference counted substring

Stores an Rc::clone() of a Rc<String> and a range
The deref behaviour means this can be used just like a &str
The advantage is the internal Rc handles the memory management so you don't have to worry about borrow lifetimes
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

#[allow(dead_code)]
impl RcSubstring {
    pub fn new(rcstring: Rc<String>, range: Range<usize>) -> Self {
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
    fn test_usage() {
        let text = "Line 1
Line 2
Line 3";
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
}
