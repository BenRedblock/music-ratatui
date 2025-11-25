use std::{cell::RefCell, path::PathBuf, rc::Rc};

use ratatui::widgets::ListItem;

use crate::{
    song::Song,
    utils::selecthandler::{SelectHandler, SelectHandlerItem},
};

#[derive(Clone)]
pub enum Node {
    Folder(Folder),
    Song(Song),
}

impl SelectHandlerItem for Node {
    fn list_item(&self) -> ListItem {
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
        if path == self.path {
            self.add_child(child);
        } else {
            let folder = self.get_folder_at_path_as_mut(path.clone());
            if let Some(folder) = folder {
                folder.add_child_at_path(child, path);
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
                if folder.path == path {
                    return Some(folder);
                }
            }
            None
        })
    }
    pub fn list_item(&self) -> ListItem {
        ListItem::new(format!(
            "📁 {} ({})",
            self.name.clone(),
            self.children.len()
        ))
    }
}
impl SelectHandlerItem for Folder {
    fn list_item(&self) -> ListItem {
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
        let mut path_stack = Vec::new();
        path_stack.push(root_folder.path.clone());
        Self {
            path_stack,
            root_folder,
            select_handler: SelectHandler::new(),
        }
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
}
