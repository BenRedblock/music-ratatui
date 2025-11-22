use musicbrainz_rs::{
    entity::{
        artist::{Artist, ArtistSearchQuery},
        recording::{Recording, RecordingSearchQuery},
    },
    prelude::*,
};
use reqwest::Error;

use crate::utils::selecthandler::SelectHandlerItem;

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

pub async fn fetch_recording(query: &str) -> Result<Vec<Recording>, musicbrainz_rs::Error> {
    let query = RecordingSearchQuery::query_builder()
        .recording(query)
        .build();

    let query_result = Recording::search(query).execute().await?.entities;

    Ok(query_result)
}

impl SelectHandlerItem for Recording {
    fn title(&self) -> String {
        self.title.clone()
    }
}
