use anyhow::{Result, anyhow};
use rfd::FileDialog;
use std::path::PathBuf;
//
// 工具：获取 EXE 所在目录
//
pub fn get_exe_dir() -> Result<std::path::PathBuf> {
    let exe = std::env::current_exe()?;
    Ok(exe
        .parent()
        .ok_or_else(|| anyhow!("获取目录失败"))?
        .to_path_buf())
}

/// 选择 JSON 文件
pub fn pick_json_file() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("JSON 文件", &["json"])
        .set_title("选择 JSON 文件")
        .pick_file()
}

/// 保存 JSON 文件
pub fn save_json_file() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("JSON 文件", &["json"])
        .save_file()
}

/// 选择 JAR 文件
pub fn pick_jar_file() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("JAR 文件", &["jar"])
        .set_title("选择 JAR 文件")
        .pick_file()
}

/// 选择目录
pub fn pick_directory() -> Option<PathBuf> {
    FileDialog::new().set_title("选择目录").pick_folder()
}

/// 保存 ZIP 文件
#[allow(dead_code)]
pub fn save_zip_file(default_name: &str) -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("ZIP 文件", &["zip"])
        .set_file_name(default_name)
        .save_file()
}
