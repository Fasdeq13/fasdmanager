use crate::fs::entry::{SortDirection, SortKey};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Grid,
    List,
}

pub struct AppState {
    pub show_hidden_files: RefCell<bool>,
    pub view_mode: RefCell<ViewMode>,
    pub sort_key: RefCell<SortKey>,
    pub sort_direction: RefCell<SortDirection>,
    pub clipboard_paths: RefCell<Vec<PathBuf>>,
    pub clipboard_is_cut: RefCell<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            show_hidden_files: RefCell::new(false),
            view_mode: RefCell::new(ViewMode::Grid),
            sort_key: RefCell::new(SortKey::Name),
            sort_direction: RefCell::new(SortDirection::Ascending),
            clipboard_paths: RefCell::new(Vec::new()),
            clipboard_is_cut: RefCell::new(false),
        }
    }
}

pub type SharedAppState = Rc<AppState>;

pub fn new_shared_state() -> SharedAppState {
    Rc::new(AppState::default())
}
