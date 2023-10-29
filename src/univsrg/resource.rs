use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, Read, Write},
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResourceEntity {
    pub original_path: PathBuf,
    pub bytes: Vec<u8>,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResourceEntry(Rc<ResourceEntity>);

impl ResourceEntry {
    fn new(original_path: PathBuf, bytes: Vec<u8>) -> Self {
        Self {
            0: Rc::from(ResourceEntity {
                original_path,
                bytes,
            }),
        }
    }

    pub fn new_from_file_in_bundle(bundle_base: &Path, original_path: PathBuf) -> io::Result<Self> {
        let mut file = File::open([bundle_base, &original_path].iter().collect::<PathBuf>())?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;
        Ok(Self::new(original_path, bytes))
    }
}

impl Deref for ResourceEntry {
    type Target = ResourceEntity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ResourcePool {
    entries: HashSet<ResourceEntry>,
    path_to_entry: HashMap<PathBuf, ResourceEntry>,
}

pub struct ResourceOut {
    entry_to_path: HashMap<ResourceEntry, PathBuf>,
}

impl ResourcePool {
    pub fn new() -> Self {
        Self {
            entries: HashSet::new(),
            path_to_entry: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entry: ResourceEntry) -> bool {
        if !self.entries.insert(entry.clone()) {
            return false;
        }
        self.path_to_entry
            .insert(entry.original_path.clone(), entry.clone());
        true
    }
    pub fn get_entry_from_path(&self, path: &Path) -> Option<ResourceEntry> {
        self.path_to_entry.get(path).cloned()
    }
    pub fn clear_path(&mut self) {
        self.path_to_entry.clear();
    }
}

impl ResourceOut {
    pub fn new() -> Self {
        Self {
            entry_to_path: HashMap::new(),
        }
    }

    pub fn inflate(&mut self, dir: PathBuf, pool: &ResourcePool) -> io::Result<()> {
        if dir.read_dir()?.next().is_some() {
            // Note: io::Result 的意思是 Result 配 io::Error。
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("dir ({}) should be an empty folder.", dir.to_str().unwrap()),
            ));
        }
        for entry in &pool.entries {
            // Get `inflated_path`.
            let original_path = &entry.original_path;
            let mut inflated_path: PathBuf = [&dir, original_path].iter().collect();
            let mut basename = inflated_path
                .file_stem()
                .unwrap() // Assume the basename is always valid.
                .to_str()
                .unwrap() // Assume there is no obscure character.
                .to_string();
            while inflated_path.exists() {
                inflated_path = inflated_path.parent().unwrap().to_owned();
                basename += "c";
                inflated_path.push(PathBuf::from(&basename));
                inflated_path.set_extension(original_path.extension().unwrap_or_default());
            }

            // Save the path.
            self.entry_to_path
                .insert(entry.clone(), original_path.clone());

            // Inflate the path.

            // Note: 文件写操作。
            let mut output = File::create(inflated_path).unwrap(); // Assume the creation is always valid.

            // Note: write 和 write_all 的区别。
            output.write_all(&entry.bytes).unwrap(); // Assume always written successfully.

            // Note: File 实现了 Drop 特征。
        }
        Ok(())
    }

    pub fn get_path_from_entry(&self, entry: &ResourceEntry) -> Option<&PathBuf> {
        self.entry_to_path.get(entry)
    }
}

#[cfg(test)]
mod test {
    // Note: 测试模块常用 use super::* 引入要测试的所有内容。
    use super::*;

    // Note: 注意编译时常量的规则，基本上与 C++ 是完全一样的，但写起来比 C++ 简单多了。
    const TEST_FILE_PATH_1: &str = "test_file_1.mp3";
    const TEST_FILE_BYTES_1: &[u8] = &[1, 1, 4];
    const TEST_FILE_PATH_2: &str = "test_file_2.mp3";
    const TEST_FILE_BYTES_2: &[u8] = &[5, 1, 4];
    const _TEST_FILE_BYTES_3: &[u8] = &[1, 1, 4, 5, 1, 4];

    fn new_example_resource_pool() -> ResourcePool {
        let mut resource_pool = ResourcePool::new();
        let test_file_1 = ResourceEntry::new(
            std::path::PathBuf::from(TEST_FILE_PATH_1),
            TEST_FILE_BYTES_1.to_vec(),
        );
        let test_file_2 = ResourceEntry::new(
            std::path::PathBuf::from(TEST_FILE_PATH_2),
            TEST_FILE_BYTES_2.to_vec(),
        );
        resource_pool.insert(test_file_1);
        resource_pool.insert(test_file_2);
        resource_pool
    }

    #[test]
    fn resource_pool_get_entry_from_path() {
        let resource_pool = new_example_resource_pool();
        let entry_1 = resource_pool
            .get_entry_from_path(Path::new(TEST_FILE_PATH_1))
            .unwrap();
        let entry_2 = resource_pool
            .get_entry_from_path(Path::new(TEST_FILE_PATH_2))
            .unwrap();
        assert_ne!(entry_1, entry_2);
    }

    #[test]
    fn resource_pool_duplicated_resource() {
        let mut resource_pool = new_example_resource_pool();
        let entry_1 = ResourceEntry::new(PathBuf::from("114514"), "114514".as_bytes().to_vec());
        let entry_2 = ResourceEntry::new(PathBuf::from("114514"), "114514".as_bytes().to_vec());
        resource_pool.insert(entry_1);
        assert_eq!(resource_pool.insert(entry_2), false);
    }

    #[test]
    fn resource_pool_clear_path() {
        let mut resource_pool = new_example_resource_pool();
        assert!(resource_pool
            .get_entry_from_path(Path::new(TEST_FILE_PATH_1))
            .is_some());
        resource_pool.clear_path();
        assert!(resource_pool
            .get_entry_from_path(Path::new(TEST_FILE_PATH_1))
            .is_none());
    }

    #[test]
    fn resource_out_inflate() {
        let resource_pool = new_example_resource_pool();
        let root = tempfile::tempdir().unwrap();
        let pathbuf = root.path().to_owned();
        let mut resource_out = ResourceOut::new();
        resource_out.inflate(pathbuf, &resource_pool).unwrap();

        let path: PathBuf = [root.path(), Path::new(TEST_FILE_PATH_1)].iter().collect();
        assert!(path.exists());
        let path: PathBuf = [root.path(), Path::new(TEST_FILE_PATH_2)].iter().collect();
        assert!(path.exists());
    }
}
