#![cfg(test)]

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use rand::RngExt;

const BASE_PATH_NAME: &str = "unit_tests";
const MAX_FILE_BYTES: usize = 4096;

// TODO: static or const?
static BASE_PATH: OnceLock<&Path> = OnceLock::new();
static CLEANUP_MUTEX: Mutex<()> = Mutex::new(());
// TODO
// static REGISTERED_TEST_PATHS: HashMap<&'static str, bool> = HashMap::new();

fn init_base_path() -> &'static Path {
    BASE_PATH.get_or_init(|| {
        let base = Path::new(BASE_PATH_NAME);
        if base.exists() {
            fs::remove_dir_all(base).unwrap();
        }
        fs::create_dir(base).unwrap();

        base
    })
}

pub fn init_test_path(name: impl AsRef<Path>) -> PathBuf {
    let path = init_base_path().join(name);
    fs::create_dir(&path).unwrap();
    path
}

pub fn cleanup_test_path(path: PathBuf) {
    let _guard = CLEANUP_MUTEX.lock().unwrap();

    let parent = path.parent().unwrap();
    assert_eq!(parent.file_name().unwrap(), BASE_PATH_NAME);

    fs::remove_dir_all(&path).unwrap();

    let parent_is_empty = parent.read_dir().unwrap().next().is_none();
    if parent_is_empty {
        fs::remove_dir(parent).unwrap();
    }
}

pub fn random_bytes() -> Vec<u8> {
    let mut rng = rand::rng();
    let len = rng.random_range(0..=MAX_FILE_BYTES);
    (0..len).map(|_| rng.random()).collect()
}

pub fn assert_exists_and_len(path: &Path, len: usize) {
    assert!(path.exists());
    assert_eq!(path.metadata().unwrap().len() as usize, len);
}
