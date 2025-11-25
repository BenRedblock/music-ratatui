use std::{fs::read_dir, path::PathBuf, str::FromStr};

use id3::{Tag, TagLike};
use log::info;
use vlc::{Instance, Media};

use crate::{
    display_handlers::folder_handler::Folder,
    song::{Song, SongType},
};

pub struct FileFinder {
    extensions: [String; 3],
    search_path: PathBuf,
    depth: u32,
    found_paths: Vec<PathBuf>,
    pub songs: Vec<Song>,
}

impl FileFinder {
    pub fn new(extensions: [String; 3], search_path: String, depth: Option<u32>) -> Self {
        let search_path = PathBuf::from(search_path);
        FileFinder {
            extensions,
            found_paths: Vec::new(),
            search_path: search_path.clone(),
            depth: depth.unwrap_or(3),
            songs: Vec::new(),
        }
    }

    pub fn find_paths(&mut self, path: Option<&PathBuf>, depth: Option<u32>) {
        let vlc_instance = Instance::new().unwrap();
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
                                    let song =
                                        FileFinder::create_song(&vlc_instance, &entry.path());
                                    if let Some(song) = song {
                                        self.songs.push(song);
                                    }
                                    self.found_paths.push(entry.path());
                                }
                            }
                        } else if file_type.is_dir() {
                            let path = entry.path();
                            let file_name = entry.file_name().to_string_lossy().to_string();
                            if depth > 0 && !file_name.starts_with(".") {
                                self.find_paths(Some(&path), Some(depth - 1));
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn create_song(vlc_instance: &Instance, path: &PathBuf) -> Option<Song> {
        if let Ok(tag) = Tag::read_from_path(path) {
            let media = Media::new_path(vlc_instance, path).expect("Media should be created");
            media.parse();
            let song = Song {
                artist: tag.artist().map(|s| s.to_string()),
                title: tag.title().unwrap_or("Not defiended").to_string(),
                total_time: media.duration().unwrap_or(5) as u32,
                album: tag.album().map(|s| s.to_string()),
                song_type: SongType::Local {
                    path: path.to_owned(),
                },
            };
            return Some(song);
        }
        return None;
    }
    pub fn create_songs(&mut self) -> &Vec<Song> {
        let mut vector = Vec::new();
        let vlc_instance = Instance::new().unwrap();
        for path in &self.found_paths {
            if let Ok(tag) = Tag::read_from_path(path) {
                if let Some(media) = Media::new_path(&vlc_instance, path) {
                    media.parse();
                    let song = Song {
                        artist: tag.artist().map(|s| s.to_string()),
                        title: tag.title().unwrap_or("Not defiended").to_string(),
                        total_time: media.duration().unwrap_or(5) as u32,
                        album: tag.album().map(|s| s.to_string()),
                        song_type: SongType::Local {
                            path: path.to_owned(),
                        },
                    };
                    vector.push(song);
                };
            }
        }
        self.songs = vector;
        &self.songs
    }
}
