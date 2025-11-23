use std::{error::Error, path::PathBuf};

use yt_dlp::Youtube;

use crate::config::Config;

pub struct Downloader {
    fetcher: Youtube,
}

impl Downloader {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let config = Config::new();
        let executables_dir = PathBuf::from(config.ytdl_libs.clone());
        let output_dir = PathBuf::from(config.ytdl_libs);

        let fetcher = Youtube::with_new_binaries(executables_dir, output_dir).await?;
        Ok(Downloader { fetcher })
    }

    pub async fn download(&self, url: &str, name: String) -> Result<PathBuf, Box<dyn Error>> {
        let url = String::from(url);
        let result = self
            .fetcher
            .download_audio_stream_from_url(url, format!("{name}.mp3"))
            .await?;
        Ok(result)
    }
}
