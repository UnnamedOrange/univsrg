use std::hash::{Hash, Hasher};

pub enum FileType {
    Unknown,
    Jpeg,
    Png,
    Mp3,
    Ogg,
}

pub struct File {
    pub file_type: FileType,
    pub bytes: Vec<u8>,
}

impl Hash for File {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}
