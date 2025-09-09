use std::collections::HashMap;
use std::path::Path;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(ldd);

static NODE_REF_TAB: Lazy<Mutex<HashMap<String, NodeValue>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub const LD_LIBRARY : &'static str = "IT_IS_LD_LIBRARY";
struct NodeValue{
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

// 对一个路径进行ldd分析，返回对应的 LibraryNode
pub fn explore(path: &Path) -> Result<LibraryNode, String> {
    if !path.exists() {
        return Err(format!("File {:?} does not exist", path));
    }
    let output = std::process::Command::new("ldd")
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to execute ldd: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ldd command failed with status: {}",
            output.status
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parser = ldd::LibraryNodeParser::new();
    match parser.parse(&stdout) {
        Ok(node) => Ok(node),
        Err(e) => Err(format!("Failed to parse ldd output: {:?}", e)),
    }
}