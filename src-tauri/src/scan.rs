use rayon::prelude::*;
use serde::Serialize;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone, Serialize)]
pub struct FileSystemItem {
    pub path: String,
    pub size: u64,
    pub size_str: String,
    pub creation_date: SystemTime,
    pub modified_date: SystemTime,
    pub is_file: bool, // 用于区分文件和文件夹
}

impl FileSystemItem {
    fn new(
        path: String,
        size: u64,
        size_str: String,
        creation_date: SystemTime,
        modified_date: SystemTime,
        is_file: bool,
    ) -> Self {
        FileSystemItem {
            path,
            size,
            size_str,
            creation_date,
            modified_date,
            is_file,
        }
    }
}

// 获取文件大小
fn get_file_size(file_path: &Path) -> io::Result<u64> {
    let metadata = fs::metadata(file_path)?;
    Ok(metadata.len())
}

// 获取文件创建日期
fn get_creation_date(file_path: &Path) -> io::Result<SystemTime> {
    let metadata = fs::metadata(file_path)?;
    metadata.created()
}

// 获取文件最后修改日期
fn get_modified_date(file_path: &Path) -> io::Result<SystemTime> {
    let metadata = fs::metadata(file_path)?;
    metadata.modified()
}

// 计算文件夹大小
fn get_folder_size(folder_path: &Path) -> io::Result<u64> {
    let mut total_size = 0;

    let entries: Vec<DirEntry> = WalkDir::new(folder_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    total_size = entries
        .into_par_iter()
        .filter_map(|entry| {
            if entry.file_type().is_file() {
                Some(get_file_size(entry.path()).unwrap_or(0))
            } else {
                None
            }
        })
        .sum(); // 汇总所有文件大小

    Ok(total_size)
}

// 生成文件大小的可读字符串
fn human_readable_size(size: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < units.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, units[unit_index])
}

// 打印大文件（超过指定大小的文件）
fn scan(dir: &Path, size_limit: u64) -> Result<Vec<FileSystemItem>, String> {
    // let mut item_list = Vec::new();
    let item_list = Arc::new(Mutex::new(Vec::new())); // 使用 Arc<Mutex<Vec>> 来包装 item_list

    let entries: Vec<DirEntry> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    entries.into_par_iter().for_each(|entry| {
        let file_size = get_file_size(entry.path()).unwrap_or(0);
        let file_path = entry.path().to_string_lossy().to_string(); // file_path 在此处转移所有权
        let creation_date = get_creation_date(entry.path()).unwrap_or(SystemTime::now());
        let modified_date = get_modified_date(entry.path()).unwrap_or(SystemTime::now());
        let file_size_str = human_readable_size(file_size);

        // 对于文件，检查其大小是否满足条件
        if file_size >= size_limit {
            if entry.file_type().is_file() {
                let file_item = FileSystemItem::new(
                    file_path,
                    file_size,
                    file_size_str,
                    creation_date,
                    modified_date,
                    true,
                ); // file_path 被移动
                   // item_list.push(file_item);
                let mut list = item_list.lock().unwrap(); // 锁定 Mutex，获取可变引用
                list.push(file_item); // 修改 item_list
            }
        }

        // 对于文件夹，计算其大小并加入列表
        if entry.file_type().is_dir() {
            let file_path = entry.path().to_string_lossy().to_string(); // file_path 在此处转移所有权

            let folder_size = get_folder_size(entry.path()).unwrap_or(0);
            if folder_size >= size_limit {
                let folder_size_str = human_readable_size(file_size);

                let file_item = FileSystemItem::new(
                    file_path,
                    folder_size,
                    folder_size_str,
                    creation_date,
                    modified_date,
                    false,
                ); // file_path 被移动
                   // item_list.push(file_item);
                let mut list = item_list.lock().unwrap(); // 锁定 Mutex，获取可变引用
                list.push(file_item); // 修改 item_list
            }
        }
    });

    // 等待所有并行任务完成后，返回 item_list
    let item_list = Arc::try_unwrap(item_list).unwrap().into_inner().unwrap();
    Ok(item_list)
}

// Tauri 命令：分析目录
#[tauri::command]
pub fn analyze_directory(path: String, size_limit: u64) -> Result<Vec<FileSystemItem>, String> {
    // 将路径转换为 Path
    let path = Path::new(&path);

    // if !path.exists() {
    //     return Err("指定的路径不存在".to_string());
    // }
    if !path.exists() {
        panic!("指定的路径不存在");
    }

    if !path.exists() {
        return Err("指定的路径不存在".to_string());
    }

    // 调用 scan 函数进行目录分析
    scan(path, size_limit)
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    println!("Hello, {:?}!", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod tests {
    use super::*;

    #[test]
    fn test_analyze_directory() {
        let path = "/Users/leimao/Documents/nfclean";
        let path = r"C:\Users\maol\Documents\githubProject\Tkinter-Designer";
        let size_limit = 1000 * 1024; // 设置一个大小限制，例如1000字节
        let result = analyze_directory(path.to_string(), size_limit);
        // 打印出来
        match result {
            Ok(files) => {
                for file in files.clone().into_iter().filter(|f| f.is_file) {
                    println!("{:?} - {:?} -{}", file.path, file.size_str, file.is_file);
                }
                println!("Total files: {}", &files.len());
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        // assert!(result.is_ok());
    }
}
