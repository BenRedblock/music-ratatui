use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

use log::info;
use ratatui::widgets::{ListItem, ListState};

use crate::{
    song::{Song, SongType},
    utils::selecthandler::{SelectHandler, SelectHandlerItem},
};

#[derive(Clone)]
pub enum Node {
    Folder(Folder),
    Song(Song),
}

impl SelectHandlerItem for Node {
    fn list_item(&self) -> ListItem<'_> {
        match self {
            Node::Folder(folder) => folder.list_item(),
            Node::Song(song) => song.list_item(),
        }
    }
}

#[derive(Clone)]
pub struct Folder {
    name: String,
    path: PathBuf,
    children: Vec<Node>,
}

impl Folder {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            children: Vec::new(),
        }
    }
    pub fn add_child(&mut self, child: Node) {
        match child {
            Node::Folder(folder) => self.children.insert(0, Node::Folder(folder)),
            Node::Song(song) => self.children.push(Node::Song(song)),
        }
    }
    pub fn add_child_at_path(&mut self, child: Node, path: PathBuf) {
        if let Some(parent) = path.parent()
            && parent == self.path
        {
            self.add_child(child);
        } else {
            let folder = self.get_folder_at_path_as_mut(path.clone());
            if let Some(folder) = folder {
                folder.add_child_at_path(child, path);
            } else {
                if let Some(name) = path.file_name() {
                    self.add_child(Node::Folder(Folder::new(
                        name.to_string_lossy().into_owned(),
                        path,
                    )));
                }
            }
        }
    }
    pub fn get_children(&self) -> &Vec<Node> {
        &self.children
    }
    pub fn get_folder_at_path_as_mut(&mut self, path: PathBuf) -> Option<&mut Folder> {
        self.children.iter_mut().find_map(|child| {
            if let Node::Folder(folder) = child {
                if folder.path == path {
                    return Some(folder);
                }
            }
            None
        })
    }
    pub fn get_folder_at_path(&self, path: PathBuf) -> Option<&Folder> {
        self.children.iter().find_map(|child| {
            if let Node::Folder(folder) = child {
                if folder.path.eq(&path) {
                    return Some(folder);
                }
            }
            None
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}
impl SelectHandlerItem for Folder {
    fn list_item(&self) -> ListItem<'_> {
        ListItem::new(format!(
            "📁 {} ({})",
            self.name.clone(),
            self.children.len()
        ))
    }
}

pub struct FolderHandler {
    root_folder: Folder,
    path_stack: Vec<PathBuf>,
    pub select_handler: SelectHandler<Node>,
}

impl FolderHandler {
    pub fn new(root_folder: Folder) -> Self {
        let path_stack = Vec::new();
        let mut s = Self {
            path_stack,
            root_folder,
            select_handler: SelectHandler::new(),
        };
        s.populate_select_handler();
        s.visualize_tree();
        s
    }
    pub fn insert_songs(&mut self, songs: Vec<Song>) {
        let mut map: HashMap<PathBuf, Folder> = HashMap::new();
        for song in songs {
            if let SongType::Local { ref path } = song.song_type {
                let parent_path = path.parent().expect("Should exist").to_path_buf();
                let parent_name = &parent_path
                    .file_name()
                    .expect("Should exist")
                    .to_string_lossy()
                    .to_string();
                let folder = map
                    .entry(parent_path.clone())
                    .or_insert(Folder::new(parent_name.to_owned(), parent_path));
                folder.add_child(Node::Song(song));
            }
        }
        for folder in map.values() {
            self.root_folder.add_child(Node::Folder(folder.clone()));
        }
        self.populate_select_handler();
    }
    pub fn current_folder(&self) -> &Folder {
        let mut current_folder = &self.root_folder;
        self.path_stack.iter().for_each(|path| {
            current_folder = current_folder.get_folder_at_path(path.clone()).unwrap();
        });
        return current_folder;
    }
    pub fn go_to_parent(&mut self) {
        if self.path_stack.len() > 0 {
            self.path_stack.pop();
            self.populate_select_handler();
        }
    }
    pub fn go_to_child(&mut self, path: &PathBuf) {
        self.path_stack.push(path.clone());
        self.populate_select_handler();
    }
    pub fn go_to_root(&mut self) {
        self.path_stack.clear();
        self.populate_select_handler();
    }
    pub fn select_handler_up(&mut self) {
        self.select_handler.up();
    }
    pub fn select_handler_down(&mut self) {
        self.select_handler.down();
    }
    pub fn select_handler_selected(&self) -> Option<&Node> {
        self.select_handler.select()
    }
    pub fn select_handler_select(&mut self) -> Option<Song> {
        match self.select_handler.select() {
            Some(node) => match node {
                Node::Song(song) => Some(song.clone()),
                Node::Folder(folder) => {
                    self.go_to_child(&folder.path.clone());

                    None
                }
            },
            _ => None,
        }
    }

    fn populate_select_handler(&mut self) {
        let current_folder = self.current_folder();
        let children = current_folder.get_children();
        self.select_handler.set_items(children.clone());
    }
    pub fn visualize_tree(&self) {
        info!("{}", self.root_folder.name());
        self.visualize_node_recursive(&self.root_folder.children, 1);
    }

    fn visualize_node_recursive(&self, nodes: &Vec<Node>, depth: usize) {
        for node in nodes {
            let indent = "  ".repeat(depth);
            match node {
                Node::Folder(folder) => {
                    info!("{}📁 {} ({})", indent, folder.name(), folder.children.len());
                    self.visualize_node_recursive(&folder.children, depth + 1);
                }
                Node::Song(song) => {
                    info!("{}🎵 {}", indent, song.title);
                }
            }
        }
    }
}
