use crate::utils::selecthandler::SelectHandlerItem;

#[derive(Clone)]
pub enum SongType {
    Local { path: String },
    OnlineDownloaded { url: String, path: String },
    Online { url: String },
    OnlineWithoutUrl,
}

#[derive(Clone)]
pub struct Song {
    pub title: String,
    pub author: Option<String>,
    pub album: Option<String>,
    pub total_time: u32,
    pub song_type: SongType,
}

impl Song {
    pub fn is_local(&self) -> bool {
        matches!(self.song_type, SongType::Local { .. })
    }

    pub fn is_online_only(&self) -> bool {
        matches!(self.song_type, SongType::Online { .. })
    }

    pub fn is_online_downloaded(&self) -> bool {
        matches!(self.song_type, SongType::OnlineDownloaded { .. })
    }
}

impl SelectHandlerItem for Song {
    fn title(&self) -> String {
        self.title.clone()
    }
}
