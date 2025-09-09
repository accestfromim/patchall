use super::*;
use std::path::Path;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(ldd);

impl LibraryNode {

    // 递归探索依赖树，填充 NODE_REF_TAB
    pub fn explore(&mut self) {
        if self.dependencies.is_empty() {
            let mut tab = NODE_REF_TAB.lock().unwrap();
            if tab.tab.contains_key(&self.path) {
                drop(tab); // 释放锁
                return;
            }
            explore_path(self.path.as_ref()).map(|node| {
                self.dependencies = node.dependencies;
                let mut path_buf = tab.base_path.clone();
                path_buf.push(self.name.clone());
                tab.tab.insert(self.path.clone(), NodeValue::new(path_buf.to_string_lossy().to_string()));
                drop(tab); // 释放锁

                for dep in &mut self.dependencies {
                    dep.explore();
                }

            }).map_err(|e|{
                eprintln!("Error during ldd analysis: {}", e);
                std::process::exit(1);
            }).unwrap();
        }else{
            // 是要patch的主程序
            let mut tab = NODE_REF_TAB.lock().unwrap();
            tab.tab.insert(self.path.clone(), NodeValue { path: self.path.clone(), has_patched: false, has_copied: true });
            drop(tab); // 释放锁
            for dep in &mut self.dependencies {
                dep.explore();
            }
        }
    }
    
}

// 对一个路径进行ldd分析，返回对应的 LibraryNode
pub fn explore_path(path: &Path) -> Result<LibraryNode, String> {
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