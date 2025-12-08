use std::fmt::Display;
use std::ops::{Deref, Range};
use std::rc::Rc;

#[allow(dead_code)]
pub struct RcSubstring {
    rcstring: Rc<String>,
    range: Range<usize>,
}

impl Display for RcSubstring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.rcstring[self.range.start..self.range.end])
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
