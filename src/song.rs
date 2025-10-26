use std::path::Path;

pub struct Song {
    pub title: String,
    pub author: Option<String>,
    pub album: Option<String>,
    pub playing: bool,
    pub time_played: u32,
    pub total_time: u32,
    pub file_path: String,
}
