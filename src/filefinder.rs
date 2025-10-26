use std::{
    fs::{DirBuilder, DirEntry, File, FileType, ReadDir, read_dir},
    path::{Path, PathBuf},
    vec,
};

use id3::{Tag, TagLike};

use crate::song::Song;

pub struct FileFinder {
    extensions: [String; 3],
    search_path: String,
    depth: u32,
    found_paths: Vec<PathBuf>,
}

impl FileFinder {
    pub fn new(extensions: [String; 3], search_path: String, depth: Option<u32>) -> Self {
        FileFinder {
            extensions,
            found_paths: Vec::new(),
            search_path: search_path,
            depth: depth.unwrap_or(5),
        }
    }

    pub fn find_paths(&mut self, path: Option<&String>, depth: Option<u32>) {
        let path = if let Some(path) = path {
            path
        } else {
            &self.search_path
        };
        let depth = if let Some(depth) = depth {
            depth
        } else {
            self.depth
        };
        if let Ok(entries) = read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            for extension in &self.extensions {
                                if entry
                                    .file_name()
                                    .into_string()
                                    .unwrap_or_else(|_| "".to_string())
                                    .ends_with(extension)
                                {
                                    self.found_paths.push(entry.path());
                                }
                            }
                        } else if file_type.is_dir() {
                            let path = entry.path().to_string_lossy().to_string();
                            if depth > 0 {
                                self.find_paths(Some(&path), Some(depth - 1));
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn create_songs(&self) -> Result<Vec<Song>, id3::Error> {
        let mut vector = Vec::new();
        for path in &self.found_paths {
            if let Ok(tag) = Tag::read_from_path(path) {
                let song = Song {
                    author: tag.artist().map(|s| s.to_string()),
                    playing: false,
                    time_played: 0,
                    title: tag.title().unwrap_or("Not defiended").to_string(),
                    total_time: 20,
                    album: tag.album().map(|s| s.to_string()),
                    file_path: path.to_string_lossy().to_string(),
                };
                vector.push(song);
            }
        }
        Ok(vector)
    }
}
