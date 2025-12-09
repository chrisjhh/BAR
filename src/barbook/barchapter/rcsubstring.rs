/*!
A reference-counted substring

For returning part of a string held in an [Rc] that needs to live longer than the structure returning it.
For a more complete alternative see [ArcStr]. This is intended as a lightweight alternative where the
string is held in an [Rc] rather than an [Arc] and in simple single-threaded situations.
*/
#![warn(missing_docs)]
use std::fmt::Display;
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
