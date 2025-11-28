use std::sync::mpsc;

use rustypipe::client::RustyPipe;

use crate::{events::ApplicationEvent, ytm::query::YTMusicQuery};

pub mod query;

pub struct YTMusic {
    event_tx: mpsc::Sender<ApplicationEvent>,
    rustypipe_client: RustyPipe,
    pub query: YTMusicQuery,
}

impl YTMusic {
    pub fn new(event_tx: mpsc::Sender<ApplicationEvent>) -> Self {
        let rustypipe_client = RustyPipe::new();
        let query = YTMusicQuery::new(rustypipe_client.query());
        Self {
            event_tx,
            rustypipe_client,
            query,
        }
    }
}
