use common::{models::service_config::ServiceConfig, utils::text_utils};
use std::{
    error::Error, process::{Command, Output}
};
use core::result::Result;

use common::utils::log_utils::write_wrapper_log;

/// 服务安装卸载
pub fn install_service(cfg: &ServiceConfig) {
    let exe_path = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let start_type = if cfg.start_mode.eq_ignore_ascii_case("automatic") {
        "auto"
    } else {
        "demand"
    };

    option_service(
        "安装",
        "sc",
        &[
            "create".to_string(),
            cfg.service_id.to_string(),
            "binPath=".to_string(),
            format!("{} -service", exe_path),
            "start=".to_string(),
            start_type.to_string(),
            "DisplayName=".to_string(),
            cfg.service_name.to_string(),
        ],
    );

    option_service(
        "设置描述",
        "sc",
        &[
            "description".to_string(),
            cfg.service_id.to_string(),
            cfg.description.as_ref().unwrap_or(&"".into()).to_string(),
        ],
    );
}

/// 卸载服务
pub fn uninstall_service(cfg: &ServiceConfig) {
    stop_service(cfg);
    option_service(
        "卸载",
        "sc",
        &["delete".to_string(), cfg.service_id.to_string()],
    );
}

/// 启动服务
pub fn start_service(cfg: &ServiceConfig) {
    option_service(
        "启动",
        "sc",
        &["start".to_string(), cfg.service_id.to_string()],
    );
}

/// 停止服务
pub fn stop_service(cfg: &ServiceConfig) {
    option_service(
        "停止",
        "sc",
        &["stop".to_string(), cfg.service_id.to_string()],
    );
}

/// GUI 操作封装
pub fn option_service(title: &str, program: &str, params: &[String]) {
    match option_service_raw(program, params) {
        Ok(output) => {
            if output.status.success() {
                write_wrapper_log(&format!(
                    "服务{}成功：{}",
                    title,
                    String::from_utf8_lossy(&output.stdout)
                ));
            } else {
                write_wrapper_log(&format!(
                    "服务{}失败：{}",
                    title,
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
        Err(e) => {
            write_wrapper_log(format!("服务{}失败：{}", title, e).as_str());
        }
    }
}

/// GUI 操作封装
pub fn option_service_raw(program: &str, params: &[String]) -> Result<Output, Box<dyn Error>> {
    // 执行命令
    let command_output = Command::new(program)
        .args(params)
        .output()?;

    // 解码输出（关键修复：先生成 String，再转 Vec<u8>）
    let stdout = text_utils::decode_system_output(&command_output.stdout).into_bytes();
    let stderr = text_utils::decode_system_output(&command_output.stderr).into_bytes();

    let result = Ok(Output {
        status: command_output.status,
        stdout,
        stderr,
    });

    // 日志打印
    match &result {
        Ok(output) => {
            write_wrapper_log(&format!(
                "命令：[{}] 参数：[{}] 状态：[{}] 输出：[{}] 错误：[{}]",
                program,
                params.join(" "),
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Err(e) => {
            write_wrapper_log(&format!(
                "命令：[{}] 参数：[{}] 执行错误：{}",
                program,
                params.join(" "),
                e
            ));
        }
    }
    result
}

/// 查询服务状态
pub fn query_service_status(id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let result = option_service_raw("sc", &["query".to_string(), id.to_string()]);
    return match result {
        Ok(output) => {
            let output_msg = String::from_utf8_lossy(&output.stdout);
            for line in output_msg.lines() {
                let line = line.trim_start();
                if line.starts_with("STATE") {
                    let parts: Vec<&str> = line.split(':').skip(1).collect();
                    return Ok(parts.join(":").trim().to_string());
                }
            }
            Ok("未安装".into())
        }
        Err(_) => Ok("未安装".into()),
    };
}
