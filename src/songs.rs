use std::collections::HashMap;

use crate::song::Song;

pub enum SortBy {
    Title,
    Artist,
    Album,
}

pub enum SortOrder {
    ASC,
    DESC,
}

pub enum Filter {
    None,
    All(String),
    Title(String),
    Artist(String),
    Album(String),
}

pub struct SongDisplay {
    songs: HashMap<String, Song>,
    sorted_by: SortBy,
    sort_order: SortOrder,
    filter: Filter,
}
