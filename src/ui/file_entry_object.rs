use crate::fs::entry::{EntryKind, FileEntry};
use gtk::glib;
use gtk::glib::subclass::prelude::*;
use gtk::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct FileEntryObject {
        pub path: RefCell<PathBuf>,
        pub display_name: RefCell<String>,
        pub kind: RefCell<Option<EntryKind>>,
        pub size: RefCell<u64>,
        pub modified: RefCell<Option<std::time::SystemTime>>,
        pub is_hidden: RefCell<bool>,
        pub icon_name: RefCell<String>,
        pub exec_command: RefCell<Option<String>>,
        pub content_type: RefCell<Option<String>>,
        pub app_id: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileEntryObject {
        const NAME: &'static str = "FasdManagerFileEntryObject";
        type Type = super::FileEntryObject;
    }

    impl ObjectImpl for FileEntryObject {}
}

glib::wrapper! {
    pub struct FileEntryObject(ObjectSubclass<imp::FileEntryObject>);
}

impl FileEntryObject {
    pub fn new(entry: FileEntry) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();
        *imp.path.borrow_mut() = entry.path;
        *imp.display_name.borrow_mut() = entry.display_name;
        *imp.kind.borrow_mut() = Some(entry.kind);
        *imp.size.borrow_mut() = entry.size;
        *imp.modified.borrow_mut() = entry.modified;
        *imp.is_hidden.borrow_mut() = entry.is_hidden;
        *imp.icon_name.borrow_mut() = entry.icon_name;
        *imp.exec_command.borrow_mut() = entry.exec_command;
        *imp.content_type.borrow_mut() = entry.content_type;
        *imp.app_id.borrow_mut() = entry.app_id;
        obj
    }

    pub fn path(&self) -> PathBuf {
        self.imp().path.borrow().clone()
    }

    pub fn display_name(&self) -> String {
        self.imp().display_name.borrow().clone()
    }

    pub fn kind(&self) -> EntryKind {
        self.imp()
            .kind
            .borrow()
            .clone()
            .unwrap_or(EntryKind::File)
    }

    pub fn size(&self) -> u64 {
        *self.imp().size.borrow()
    }

    pub fn modified(&self) -> Option<std::time::SystemTime> {
        *self.imp().modified.borrow()
    }

    pub fn is_hidden(&self) -> bool {
        *self.imp().is_hidden.borrow()
    }

    pub fn icon_name(&self) -> String {
        self.imp().icon_name.borrow().clone()
    }

    pub fn exec_command(&self) -> Option<String> {
        self.imp().exec_command.borrow().clone()
    }

    pub fn content_type(&self) -> Option<String> {
        self.imp().content_type.borrow().clone()
    }

    pub fn app_id(&self) -> Option<String> {
        self.imp().app_id.borrow().clone()
    }

    pub fn to_file_entry(&self) -> FileEntry {
        FileEntry {
            path: self.path(),
            display_name: self.display_name(),
            kind: self.kind(),
            size: self.size(),
            modified: self.modified(),
            is_hidden: self.is_hidden(),
            icon_name: self.icon_name(),
            content_type: self.content_type(),
            exec_command: self.exec_command(),
            app_id: self.app_id(),
        }
    }
}
