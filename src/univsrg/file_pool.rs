use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    rc::Rc,
};

// Note: Eq 表示告诉编译器等号具有自反性。只需编写 PartialEq 的代码即可，Eq 总是一个标记。
#[derive(Debug, Eq)]
pub struct File {
    pub original_path: PathBuf,
    pub bytes: Vec<u8>,
}
impl Hash for File {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.original_path.hash(state);
    }
}
impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

pub struct FilePool {
    file_to_id: HashMap<Rc<File>, u32>,
    id_to_file: HashMap<u32, Rc<File>>,
    path_to_id: HashMap<PathBuf, u32>,
}

impl FilePool {
    fn new() -> Self {
        Self {
            file_to_id: HashMap::new(),
            id_to_file: HashMap::new(),
            path_to_id: HashMap::new(),
        }
    }

    fn insert(&mut self, file_with_path: File) -> u32 {
        // Note: *Map 的 get 方法返回的是引用的 Option，用 Option 的 copied 方法将引用去掉。
        // Note: *Map 的 get 方法传入的是引用。类比 const auto&。
        let option_id = self.file_to_id.get(&file_with_path).copied();
        match option_id {
            Some(id) => {
                // If duplicated bytes are found, also update path_to_file,
                // but do not create another copy.
                self.path_to_id
                    .insert(file_with_path.original_path.clone(), id);
                id
            }
            None => {
                let rc = Rc::new(file_with_path);
                let id = self.file_to_id.len() as u32;
                self.file_to_id.insert(rc.clone(), id);
                self.id_to_file.insert(id, rc.clone());
                self.path_to_id.insert(rc.original_path.clone(), id);
                id
            }
        }
    }
    fn get_id_from_path(&self, path: &Path) -> Option<u32> {
        self.path_to_id.get(path).copied()
    }
    fn get_file_from_id(&self, id: u32) -> Option<Rc<File>> {
        // Note: 使用 Option 的 cloned 方法将引用去掉。
        self.id_to_file.get(&id).cloned()
    }
    fn clear_path(&mut self) {
        self.path_to_id.clear();
    }
}
