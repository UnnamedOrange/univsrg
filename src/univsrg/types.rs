use std::rc::Rc;

// Note: 使用 super 表示上一级模块，即 univsrg。
// Note: mod.rs 已经将所有模块引入，所以不需再引入，只需用 use 语句缩写。
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
