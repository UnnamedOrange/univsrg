use std::rc::Rc;

use super::file_pool;

pub struct LatinAndUnicodeString {
    latin: Option<String>,
    unicode: Option<String>,
}

pub struct Beatmap {
    pub title: LatinAndUnicodeString,
    pub artist: LatinAndUnicodeString,
    pub version: String,
    pub creator: String,
    pub background: Option<Rc<file_pool::File>>,
}
