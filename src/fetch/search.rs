use std::fmt::format;

use musicbrainz_rs::{
    entity::{
        artist::{Artist, ArtistSearchQuery},
        recording::{Recording, RecordingSearchQuery, RecordingSearchQueryLuceneQueryBuilder},
    },
    prelude::*,
};
use reqwest::Error;
use rusty_ytdl::search::{SearchOptions, SearchResult, YouTube};

use crate::{
    song::{Song, SongType},
    utils::selecthandler::SelectHandlerItem,
};

pub async fn fetch_artists_manual() -> Result<(), Error> {
    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.append(reqwest::header::USER_AGENT, "anonymous".parse().unwrap());
    let res = client
        .get("http://musicbrainz.org/ws/2/artist/?query=artist:giant_rooks")
        .headers(headers)
        .send()
        .await?;
    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());

    let body = res.text().await?;
    println!("Body:\n{}", body);
    Ok(())
}

fn create_query(input: &str, max_results: u32) -> String {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let mut base_string = format!("query=recording:\"{}\" OR artist:\"{}\"", input, input);
    if parts.is_empty() {
        return "".to_string();
    }
    if parts.len() > 1 {
        // Split: "Title" ... "Artist"
        for i in 1..parts.len() {
            let (left, right) = parts.split_at(i);
            base_string = format!(
                "{} OR (recording:\"{}\" AND artist:\"{}\")",
                base_string,
                &left.join(" "),
                &right.join(" ")
            );
        }

        // Reverse Split: "Artist" ... "Title"
        for i in 1..parts.len() {
            let (left, right) = parts.split_at(i);
            base_string = format!(
                "{} OR (artist:\"{}\" AND recording:\"{}\")",
                base_string,
                &left.join(" "),
                &right.join(" ")
            );
        }
    }
    return format!("{}&limit={}", base_string, max_results);
}

pub async fn fetch_recording(query: &str) -> Result<Vec<Song>, Box<dyn std::error::Error>> {
    let search_string = create_query(query, 10);

    let youtube = YouTube::new().unwrap();
    let mut query_result: Vec<Song> = Recording::search(search_string)
        .execute()
        .await?
        .entities
        .iter()
        .map(|recording| {
            let artist = recording
                .artist_credit
                .as_ref()
                .and_then(|r| r.first())
                .map(|r| Some(r.artist.name.clone()))
                .unwrap_or(None);
            let album = recording
                .releases
                .as_ref()
                .and_then(|rels| rels.get(0))
                .and_then(|release| release.release_group.as_ref())
                .map(|rg| Some(rg.title.clone()))
                .unwrap_or(None);
            let title = recording.title.clone();
            let total_time = recording.length.unwrap_or(0) / 1000;

            Song {
                album,
                author: artist,
                title,
                total_time,
                song_type: SongType::OnlineWithoutUrl,
            }
        })
        .collect();

    for song in &mut query_result {
        let yt_search_result = youtube
            .search_one(
                format!("{}", song.title),
                Some(&SearchOptions {
                    search_type: rusty_ytdl::search::SearchType::Video,
                    ..Default::default()
                }),
            )
            .await;
        if let Ok(Some(SearchResult::Video(video))) = yt_search_result {
            song.song_type = SongType::Online { url: video.url }
        }
    }

    Ok(query_result)
}

impl SelectHandlerItem for Recording {
    fn title(&self) -> String {
        self.title.clone()
    }
}
