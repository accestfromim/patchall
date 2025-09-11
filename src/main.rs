#![allow(dead_code)]
mod patcher;
use std::io::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use clap::builder::ValueParserFactory;
use clap::{Arg, Command};
use std::process::{Command as ProcessCommand};
use patcher::NODE_REF_TAB;

use crate::patcher::path_analyzer::{get_file_name_from_path,explore_path};

const BACKUP_DIR_NAME : &str = "ldd_backup_list";

// 检查文件是否是可执行文件
fn is_executable(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => {
            // 检查是否是文件
            if metadata.is_file() {
                #[cfg(unix)]
                {
                    // 使用 file 命令检查文件类型
                    let output = ProcessCommand::new("file")
                        .arg("--mime-type")
                        .arg(path)
                        .output();

                    if let Ok(output) = output {
                        let mime_type = String::from_utf8_lossy(&output.stdout);
                        return mime_type.contains("executable");
                    }
                }
            }
            false
        }
        Err(_) => false,
    }
}

fn get_library_search_path() -> Option<String>{
    let tab = NODE_REF_TAB.lock().unwrap();
    let lpath = tab.get_library_find_path();
    drop(tab);
    lpath
}

fn set_library_find_path(lpath: String){
    let mut tab = NODE_REF_TAB.lock().unwrap();
    tab.set_library_find_path(lpath);
    drop(tab);
}

fn get_library_path_from_name(name: &str, path: &str) -> String{
    let lpath = get_library_search_path();
    if lpath.is_none(){
        path.to_string() 
    }else{
        let path = PathBuf::from(lpath.unwrap());
        let mut path = fs::canonicalize(path).unwrap();
        path.push(get_file_name_from_path(name));
        path.to_string_lossy().to_string()
    }
}

fn main() -> Result<()> {
    // 解析命令行参数
    let matches = Command::new("patchall")
        .about("Patches all specified programs")
        .arg(
            Arg::new("program_name")
                .help("The name of the program to patch")
                .required(true) // 这个参数是必需的
                .index(1), // 第一个参数
        )
        .arg(
            Arg::new("path")
                .help("The path to the library output path")
                .required(true) // 这个参数是必需的
                .index(2), // 第二个参数
        )
        .arg(
            Arg::new("lpath")
                .long("lpath") // 指定长选项名
                .value_parser(String::value_parser()) // 这个选项需要一个值
                .help("The path to the library") // 选项的帮助信息
        )
        .get_matches();

    let program_name = matches.get_one::<String>("program_name").unwrap();
    let path = matches.get_one::<String>("path").unwrap();
    let lpath = matches.get_one::<String>("lpath");
    if let Some(lpath) = lpath {
        set_library_find_path(lpath.to_string());
    }

    let path = Path::new(path);

    let program_name = Path::new(program_name);
    let mut target_path = PathBuf::from(path);

    // 检查这是不是个可执行程序
    if !is_executable(&program_name) {
        eprintln!("Error: The specified program '{}' is not a valid executable.", program_name.display());
        process::exit(1);
    }

    // 创建 dependencies 目录
    target_path.push("dependencies");

    // 创建目录
    if let Err(e) = fs::create_dir_all(&target_path) {
        eprintln!("Failed to create directory {:?}: {}", target_path, e);
        std::process::exit(1);
    }

    let mut tab = NODE_REF_TAB.lock().unwrap();
    tab.base_path = fs::canonicalize(target_path.clone()).map_err(|e|{
        eprintln!("Failed to canonicalize path {:?}: {}", target_path, e);
        std::process::exit(1);
    }).unwrap();
    drop(tab); // 释放锁

    let mut program = explore_path(&program_name).map_err(|e|{
        eprintln!("Error during ldd analysis: {}", e);
        std::process::exit(1);
    }).unwrap();
    program.path = program_name.to_string_lossy().to_string();
    program.explore();
    
    let mut ldd_result_path = target_path.clone();
    ldd_result_path.push(BACKUP_DIR_NAME);
    if let Err(e) = fs::create_dir_all(&ldd_result_path) {
        eprintln!("Failed to create directory {:?}: {}", ldd_result_path, e);
        std::process::exit(1);
    }
    // println!("{:?}",program);
    program.patch();

    //println!("Program Name: {:?}", program_name);
    //println!("Path: {:?}", target_path);
    println!("Patching completed successfully.");
    Ok(())
}