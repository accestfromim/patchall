pub mod path_analyzer;
pub mod library_patcher;
use std::collections::HashMap;
use std::path::PathBuf;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub struct NodeRefTab{
    pub base_path: PathBuf,
    pub ld_library_path: Option<String>,
    pub tab: HashMap<String, NodeValue>,
    pub library_find_path: Option<String>,
}

impl NodeRefTab {
    fn new() -> Self {
        NodeRefTab {
            base_path: PathBuf::from("/"),
            ld_library_path: None,
            tab: HashMap::new(),
            library_find_path: None,
        }
    }
    pub fn get_library_find_path(&self) -> Option<String> {
        self.library_find_path.clone()
    }
    pub fn set_library_find_path(&mut self, path: String) {
        self.library_find_path = Some(path);
    }
    fn get(&self, key: String) -> Option<&NodeValue> {
        self.tab.get(&key)
    }
    fn insert(&mut self, key: String, value: NodeValue) {
        self.tab.insert(key, value);
    }
    fn get_ld_library_path(&self) -> Option<String> {
        self.ld_library_path.clone()
    }
    fn set_ld_library_path(&mut self, path: String) {
        self.ld_library_path = Some(path);
    }
}

pub static NODE_REF_TAB: Lazy<Mutex<NodeRefTab>> = Lazy::new(|| {
    Mutex::new(NodeRefTab::new())
});

// pub const LD_LIBRARY : &'static str = "ld.so.1";
pub struct NodeValue{
    path: String,
    has_patched: bool,
    has_copied: bool,
}

impl NodeValue {
    fn new(path: String) -> Self {
        NodeValue {
            path,
            has_patched: false,
            has_copied: false,
        }
    }
}

#[derive(Debug)]
pub struct LibraryNode {
    pub name: String,
    pub path: String,
    pub dependencies: Vec<LibraryNode>,
}

