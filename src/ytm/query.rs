use rustypipe::client::RustyPipeQuery;

pub struct YTMusicQuery {
    query_builder: RustyPipeQuery,
}

impl YTMusicQuery {
    pub fn new(query_builder: RustyPipeQuery) -> Self {
        Self { query_builder }
    }

    pub fn search(&self, query: &str) {
        self.query_builder.search(query);
    }
}
