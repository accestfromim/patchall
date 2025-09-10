use super::*;
use std::path::Path;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(ldd);

// 输入一个字符串表示的路径，返回文件名
pub fn get_file_name_from_path(path: &str) -> String {
    match Path::new(path).file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => String::new(),
    }
}

impl LibraryNode {

    // 递归探索依赖树，填充 NODE_REF_TAB
    pub fn explore(&mut self) {
        if self.dependencies.is_empty() {
            let tab = NODE_REF_TAB.lock().unwrap();
            if tab.tab.contains_key(&self.path) {
                drop(tab); // 释放锁
                return;
            }
            drop(tab);
            explore_path(self.path.as_ref()).map(|node| {
                self.dependencies = node.dependencies;
                let mut tab = NODE_REF_TAB.lock().unwrap();
                let mut path_buf = tab.base_path.clone();
                let self_name = get_file_name_from_path(&self.name);
                path_buf.push(self_name);
                tab.tab.insert(self.path.clone(), NodeValue::new(path_buf.to_string_lossy().to_string()));
                drop(tab); // 释放锁

                for dep in &mut self.dependencies {
                    dep.explore();
                }

            }).map_err(|e|{
                eprintln!("Error during ldd analysis of {}: {}",self.path, e);
                std::process::exit(1);
            }).unwrap();
        }else{
            // 是要patch的主程序
            let mut tab = NODE_REF_TAB.lock().unwrap();
            tab.tab.insert(self.path.clone(), NodeValue { path: self.path.clone(), has_patched: false, has_copied: true });
            drop(tab); // 释放锁
            for dep in &mut self.dependencies {
                if get_file_name_from_path(dep.name.as_ref()).starts_with("ld-") {
                    let mut tab = NODE_REF_TAB.lock().unwrap();
                    tab.ld_library_path = Some(dep.path.clone());
                    drop(tab); // 释放锁
                    break;
                }
            }
            for dep in &mut self.dependencies {
                dep.explore();
            }
        }
    }
    
}

// 对一个路径进行ldd分析，返回对应的 LibraryNode
pub fn explore_main_program_path(path: &Path) -> Result<LibraryNode, String> {
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

// 对一个路径进行ldd分析，返回对应的 LibraryNode
pub fn explore_path(path: &Path) -> Result<LibraryNode, String> {
    if !path.exists() {
        return Err(format!("File {:?} does not exist", path));
    }
    let tab = NODE_REF_TAB.lock().unwrap();
    let ld_library_path = tab.ld_library_path.clone();
    drop(tab); // 释放锁
    if ld_library_path.is_none(){
        return Err(format!("not found ld in ldd of the input program"));
    }
    let ld_library_path = ld_library_path.unwrap();
    let output = std::process::Command::new(ld_library_path)
        .arg("--list")
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