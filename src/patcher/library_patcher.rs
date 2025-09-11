use super::*;
use crate::patcher::path_analyzer::get_file_name_from_path;
use std::process::{Command as ProcessCommand};
use crate::patcher::path_analyzer::ldd_parser::do_ldd_and_write_to_output_library;

impl LibraryNode{
    // 递归地使用patchelf工具对依赖进行patch
    // 一定先被copy, 才会被调用patch
    pub fn patch(&self) {
        // 检查自己有没有被patch过，有就直接返回
        let mut tab = NODE_REF_TAB.lock().unwrap();
        let new_ld_library_path = tab.tab.get(&tab.get_ld_library_path().unwrap()).unwrap().path.clone();
        let self_path = tab.tab.get(&self.path).unwrap().path.clone();
        if let Some(node) = tab.tab.get_mut(&self.path) {
            if node.has_patched {
                drop(tab);
                return;
            }else{
                node.has_patched = true;
            }
        }else{
            eprintln!("Error: Library {} not found in NODE_REF_TAB", self.path);
            std::process::exit(1);
        }
        // 先copy所有的依赖
        for dep in &self.dependencies {
            if let Some(node) = tab.tab.get_mut(&dep.path) {
                if !node.has_copied {
                    let output = ProcessCommand::new("cp")
                        .arg(&dep.path)
                        .arg(&node.path)
                        .output().map_err(|e|{
                            eprintln!("Failed to execute cp command for {}: {}", dep.path, e);
                            std::process::exit(1);
                        }).unwrap();
                    if !output.status.success() {
                        eprintln!("Failed to copy {} to {}: {}", dep.path, node.path, String::from_utf8_lossy(&output.stderr));
                        std::process::exit(1);
                    } else {
                        println!("Copied {} to {}", dep.path, node.path);
                        node.has_copied = true;
                    }
                }
            }else{
                eprintln!("Error: Dependency {} not found in NODE_REF_TAB", dep.path);
                std::process::exit(1);
            }
        }
        // 再patch自己
        for dep in &self.dependencies {
            if let Some(node) = tab.tab.get_mut(&dep.path) {

                if get_file_name_from_path(dep.name.as_ref()).starts_with("ld-") {
                    let output = ProcessCommand::new("patchelf")
                        .arg("--set-interpreter")
                        .arg(&new_ld_library_path)
                        .arg(&self_path)
                        .output().map_err(|e|{
                            eprintln!("Failed to execute patchelf command for {}: {}", self.path, e);
                            std::process::exit(1);
                        }).unwrap();
                    if !output.status.success() {
                        if !self.name.is_empty(){
                            continue; // 不是主程序可能没有.interp段, 就不管
                        }
                        eprintln!("Failed to set interpreter for {}: {}", self_path, String::from_utf8_lossy(&output.stderr));
                        std::process::exit(1);
                    } else {    
                        println!("Set interpreter for {}: to {}", self_path, node.path);
                    }
                    continue;
                }

                let output = ProcessCommand::new("patchelf")
                    .arg("--replace-needed")
                    .arg(&dep.name)
                    .arg(&node.path)
                    .arg(&self_path)
                    .output().map_err(|e|{
                        eprintln!("Failed to execute patchelf command for {}: {}", self.path, e);
                        std::process::exit(1);
                    }).unwrap();
                if !output.status.success() {
                    eprintln!("Failed to patch {}: {}", self_path, String::from_utf8_lossy(&output.stderr));
                    std::process::exit(1);
                } else {
                    println!("Patched {}: replaced {} with {}", self_path, dep.name, node.path);
                }
            }else{
                eprintln!("Error: Dependency {} not found in NODE_REF_TAB during patching", dep.path);
                std::process::exit(1);
            }
        }
        drop(tab);
        for dep in &self.dependencies {
            dep.patch();
        }
        do_ldd_and_write_to_output_library(&self_path);
    }
}