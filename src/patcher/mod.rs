pub mod path_analyzer;
pub mod library_patcher;
use std::collections::HashMap;
use std::path::PathBuf;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub struct NodeRefTab{
    pub base_path: PathBuf,
    pub tab: HashMap<String, NodeValue>,
}

impl NodeRefTab {
    fn new() -> Self {
        NodeRefTab {
            base_path: PathBuf::from("/"),
            tab: HashMap::new(),
        }
    }
}

pub static NODE_REF_TAB: Lazy<Mutex<NodeRefTab>> = Lazy::new(|| {
    Mutex::new(NodeRefTab::new())
});

pub const LD_LIBRARY : &'static str = "ld.so.1";
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

