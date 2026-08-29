#![allow(dead_code)]

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

pub fn gather_feature_files() -> Result<Vec<PathBuf>, Error> {
    let extension: Option<&OsStr> = Some(OsStr::new(".feature"));
    let mut stack = vec![PathBuf::from("openCypher/tck/features/")];
    let mut files: Vec<PathBuf> = Vec::new();

    while let Some(current_dir) = stack.pop() {
        for entry in fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                if path.extension() == extension {
                    files.push(path);
                }
            }
        }
    }
    Ok(files)
}
