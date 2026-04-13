use crate::models::service_config::ServiceConfig;
use crate::utils::log_utils::write_wrapper_log;
use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;
use tokio::io::AsyncWriteExt;

// 读取配置
pub fn read_config_from_exe() -> Result<ServiceConfig> {
    write_wrapper_log("开始读取配置...");

    let exe_path_result = std::env::current_exe();
    let exe_path = match exe_path_result {
        Ok(path) => path,
        Err(e) => {
            write_wrapper_log(&format!("获取可执行文件路径失败：{}", e));
            return Err(anyhow!("获取可执行文件路径失败: {}", e));
        }
    };
    let data_result = fs::read(exe_path);
    let data = match data_result {
        Ok(data) => data,
        Err(e) => {
            write_wrapper_log(&format!("读取可执行文件失败：{}", e));
            return Err(anyhow!("读取可执行文件失败: {}", e));
        }
    };

    let json_len = u16::from_le_bytes(data[data.len() - 2..].try_into().unwrap());
    let json_bytes = &data[data.len() - 2 - json_len as usize..data.len() - 2];

    let cfg_result: Result<ServiceConfig, serde_json::Error> = serde_json::from_slice(json_bytes);

    let cfg = match cfg_result {
        Ok(c) => c,
        Err(e) => {
            write_wrapper_log(&format!("配置解析失败：{}", e));
            return Err(anyhow!("配置解析失败: {}", e));
        }
    };

    write_wrapper_log(&format!("读取配置成功：服务ID={}", cfg.service_id));
    Ok(cfg)
}

/// 将配置写入到控制器可执行文件末尾
pub async fn write_config_to_exe(dest: &Path, config_str: String) -> Result<()> {
    let exe_file_result = tokio::fs::OpenOptions::new()
        .append(true)
        .write(true)
        .open(&dest)
        .await;

    let mut exe_file = match exe_file_result {
        Ok(file) => file,
        Err(e) => {
            write_wrapper_log(&format!("打开可执行文件失败：{}", e));
            return Err(anyhow!("打开可执行文件失败: {}", e));
        }
    };

    // 写入配置数据到文件末尾
    let json_bytes = config_str.as_bytes();
    let json_len = u16::to_le_bytes(json_bytes.len() as u16);
    exe_file.write_all(json_bytes).await?;
    exe_file.write_all(&json_len).await?;

    write_wrapper_log(&format!("配置文件已写入可执行文件：{}", dest.display()));
    Ok(())
}
