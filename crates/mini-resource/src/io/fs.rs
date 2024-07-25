use std::{fs::File, io::Read, path::PathBuf};

use super::FileLoadError;

pub async fn load_file(path: &PathBuf) -> Result<Vec<u8>, FileLoadError> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub async fn exists(path: &PathBuf) -> bool {
    path.as_path().exists()
}

pub async fn is_file(path: &PathBuf) -> bool {
    path.is_file()
}

pub async fn is_dir(path: &PathBuf) -> bool {
    path.is_dir()
}
