use std::rc::Rc;

// Note: 使用 super 表示上一级模块，即 univsrg。
// Note: mod.rs 已经将所有模块引入，所以不需再引入，只需用 use 语句缩写。
use super::file_pool::{self, FilePool};

#[derive(Debug)]
pub struct LatinAndUnicodeString {
    latin: Option<String>,
    unicode: Option<String>,
}

#[derive(Debug)]
pub struct BpmTimePoint {
    pub offset: u32,
    pub bpm: f32,
    pub beats_per_bar: u32,
}

#[derive(Debug)]
pub struct EffectTimePoint {
    pub offset: u32,
    pub velocity_multiplier: f32,
}

#[derive(Debug)]
pub enum Object {
    Note { offset: u32 },
    LongNote { offset: u32, end_offset: i32 },
}

pub struct Beatmap {
    pub title: LatinAndUnicodeString,
    pub artist: LatinAndUnicodeString,
    pub version: Option<String>,
    pub creator: Option<String>,
    pub column_count: u32,
    pub audio: Rc<file_pool::File>,
    pub background: Option<Rc<file_pool::File>>,
    pub hp_difficulty: Option<f32>,
    pub acc_difficulty: Option<f32>,

    pub bpm_time_points: Vec<BpmTimePoint>,
    pub effect_time_points: Vec<EffectTimePoint>,
    pub objects: Vec<Object>,
}

pub struct Package {
    pub beatmaps: Vec<Beatmap>,
    pub file_pool: FilePool,
}

impl Package {
    pub fn new() -> Self {
        Package {
            beatmaps: Vec::new(),
            file_pool: FilePool::new(),
        }
    }
}
