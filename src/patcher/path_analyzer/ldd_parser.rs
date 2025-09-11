use crate::patcher::path_analyzer::{get_file_name_from_path, LibraryNode};
use crate::patcher::NODE_REF_TAB;
use crate::BACKUP_DIR_NAME;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(ldd);

// 接收一个path作为参数，返回一个Option<LibraryNode>类型
pub fn get_node_from_path(path: &str) -> Result<LibraryNode, anyhow::Error> {
    let tab = NODE_REF_TAB.lock().unwrap();
    if let Some(lpath) = tab.get_library_find_path() {
        drop(tab);
        let mut new_path = std::path::PathBuf::from(lpath);
        new_path.push(BACKUP_DIR_NAME);
        new_path.push(get_file_name_from_path(path));
        // 将这个path所代表的文件内容读出来交给ldd_parser解析
        if let Ok(content) = std::fs::read_to_string(new_path) {
            match ldd::LibraryNodeParser::new().parse(&content) {
                Ok(node) => Ok(node),
                Err(e) => Err(anyhow::anyhow!("Failed to parse ldd output for {}: {}", path, e)),
            }
        } else {
            Err(anyhow::anyhow!("Failed to read backup ldd file for {}", path))   
        }
    } else {
        // 使用系统的ldd命令或者是从tab所得到的动态链接器来解析
        let ld_library = tab.get_ld_library_path();
        drop(tab);
        let output = if let Some(ld_path) = ld_library {
            std::process::Command::new(ld_path)
                .arg("--list")
                .arg(path)
                .output()
        } else {
            std::process::Command::new("ldd").arg(path).output()
        };
        match output {
            Ok(output) => {
                if !output.status.success() {
                    return Err(anyhow::anyhow!(
                        "ldd command failed with status: {}",
                        output.status
                    ));
                }
                let stdout = String::from_utf8_lossy(&output.stdout);
                match ldd::LibraryNodeParser::new().parse(&stdout) {
                    Ok(node) => Ok(node),
                    Err(e) => Err(anyhow::anyhow!("Failed to parse ldd output for {}: {}", path, e)),
                }
            }
            Err(e) => Err(anyhow::anyhow!("Failed to execute ldd for {}: {}", path, e)),
        }
    }
}

// 对一个path用ld_library --list或者ldd做解析，并把结果写在输出目录下
pub fn do_ldd_and_write_to_output_library(path: &str){
    let tab = NODE_REF_TAB.lock().unwrap();
    let mut output_path = tab.base_path.clone();
    output_path.push(BACKUP_DIR_NAME);
    output_path.push(get_file_name_from_path(path));
    if let Some(ld_library_path) = tab.get_ld_library_path() {
        drop(tab);
        let output = std::process::Command::new(&ld_library_path)
            .arg("--list")
            .arg(path)
            .output().map_err(|e|{
                eprintln!("Failed to execute {} command for {}: {}",ld_library_path, path, e);
                std::process::exit(1);
            }).unwrap();
        if !output.status.success() {
            eprintln!("ldd command failed with status: {}", output.status);
            std::process::exit(1);
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        std::fs::write(&output_path, stdout.as_ref()).map_err(|e|{
            eprintln!("Failed to write ldd output to {:?}: {}", output_path, e);
            std::process::exit(1);
        }).unwrap();
    } else {
        drop(tab);
        let output = std::process::Command::new("ldd")
            .arg(path)
            .output().map_err(|e|{
                eprintln!("Failed to execute ldd command for {}: {}", path, e);
                std::process::exit(1);
            }).unwrap();
        if !output.status.success() {
            eprintln!("ldd command failed with status: {}", output.status);
            std::process::exit(1);
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        std::fs::write(&output_path, stdout.as_ref()).map_err(|e|{
            eprintln!("Failed to write ldd output to {:?}: {}", output_path, e);
            std::process::exit(1);
        }).unwrap();
    }
}
        