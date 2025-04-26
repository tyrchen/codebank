use super::FileUnit;
use std::path::PathBuf;

impl FileUnit {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            ..Default::default()
        }
    }
}
