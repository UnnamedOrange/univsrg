// Note: 使用 super 表示上一级模块，即 univsrg。
// Note: mod.rs 已经将所有模块引入，所以不需再引入，只需用 use 语句缩写。
use super::resource::{ResourceEntry, ResourcePool};

#[derive(Debug)]
pub struct LatinAndUnicodeString {
    pub latin: Option<String>,
    pub unicode: Option<String>,
}

impl LatinAndUnicodeString {
    pub fn new() -> Self {
        Self {
            latin: None,
            unicode: None,
        }
    }

    pub fn latin_or_unicode(&self) -> Option<&String> {
        // Note: 使用 Option 的 as_ref 方法将 &Option<T> 转换为 Option<&T>。
        // Note: &self 的 self.latin 就已经是引用了。
        self.latin.as_ref().or(self.unicode.as_ref())
    }
    pub fn unicode_or_latin(&self) -> Option<&String> {
        self.unicode.as_ref().or(self.latin.as_ref())
    }
}

#[derive(Debug)]
pub struct BpmTimePoint {
    pub offset: i32,
    pub bpm: f32,
    pub beats_per_bar: u32,
}

#[derive(Debug)]
pub struct EffectTimePoint {
    pub offset: i32,
    pub velocity_multiplier: f32,
}

#[derive(Debug)]
pub enum Object {
    Note {
        column: u32,
        offset: i32,
    },
    LongNote {
        column: u32,
        offset: i32,
        end_offset: i32,
    },
}

pub struct Beatmap {
    pub title: LatinAndUnicodeString,
    pub artist: LatinAndUnicodeString,
    pub version: Option<String>,
    pub creator: Option<String>,
    pub column_count: Option<u32>,
    pub audio: Option<ResourceEntry>,
    pub audio_lead_in: Option<i32>,
    pub preview_time: Option<i32>,
    pub background: Option<ResourceEntry>,
    pub hp_difficulty: Option<f32>,
    pub acc_difficulty: Option<f32>,

    pub bpm_time_points: Vec<BpmTimePoint>,
    pub effect_time_points: Vec<EffectTimePoint>,
    pub objects: Vec<Object>,
}

pub struct Package {
    pub beatmaps: Vec<Beatmap>,
    pub resource_pool: ResourcePool,
}

impl Beatmap {
    pub fn new() -> Self {
        Self {
            title: LatinAndUnicodeString::new(),
            artist: LatinAndUnicodeString::new(),
            version: None,
            creator: None,
            column_count: None,
            audio: None,
            audio_lead_in: None,
            preview_time: None,
            background: None,
            hp_difficulty: None,
            acc_difficulty: None,
            bpm_time_points: vec![],
            effect_time_points: vec![],
            objects: vec![],
        }
    }
    pub fn make_basename(&self) -> String {
        let mut names = Vec::<&str>::new();
        self.creator.as_ref().map(|it| {
            names.push(it);
        });
        self.title.unicode_or_latin().map(|it| {
            names.push(it);
        });
        self.version.as_ref().map(|it| {
            names.push(it);
        });
        // Note: 不能对 Vec::<&String> 进行 join。因为 &String 没有提供 iter 方法。
        names.join(" - ")
    }
}

impl Package {
    pub fn new() -> Self {
        Package {
            beatmaps: Vec::new(),
            resource_pool: ResourcePool::new(),
        }
    }
}
