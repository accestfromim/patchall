#![allow(dead_code)]
mod path_analyzer;
use std::io::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use clap::{Arg, Command};
use std::process::{Command as ProcessCommand};

use crate::path_analyzer::explore;



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
        .get_matches();

    let program_name = matches.get_one::<String>("program_name").unwrap();
    let path = matches.get_one::<String>("path").unwrap();

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

    let test = explore(&program_name).map_err(|e|{
        eprintln!("Error during ldd analysis: {}", e);
        std::process::exit(1);
    }).unwrap();

    println!("{:?}",test);

    println!("Program Name: {:?}", program_name);
    println!("Path: {:?}", target_path);
    /*
    let mut args = args();
    args.next();
    let mode = args.next().unwrap();
    let input_file = args.next().unwrap();
    args.next();
    let output = args.next().unwrap();

    // 读取输入文件
    let i_input = read_to_string(&input_file)?;
    let input = i_input
        .replace("i = i + 1;", "i++;")
        .replace("j = j + 1;", "j++;");
    //let mut outfile = File::create(&input_file.replace("/", "_"))?;
    //outfile.write_fmt(format_args!("{input}"))?;

    // 调用 lalrpop 生成的 parser 解析输入文件
    let mut ast = ldd::LibraryNodeParser::new().parse(&input).unwrap();
*/
    Ok(())
}