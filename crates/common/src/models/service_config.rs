use serde::{Deserialize, Serialize};

/// Windows 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    // ==================== 必填字段 ====================
    /// 服务 ID（用于文件名和标识）
    pub service_id: String,

    /// 服务名称（Windows 服务列表显示）
    pub service_name: String,

    /// JAR 包路径
    pub jar_path: String,

    // ==================== 可选字段 - 基本信息 ====================
    /// 服务描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    // ==================== 可选字段 - 自定义命令 ====================
    /// 自定义命令
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable: Option<String>,

    // ==================== 可选字段 - Java 环境 ====================
    /// 自定义 Java 路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_executable: Option<String>,

    /// 是否打包 JRE
    #[serde(default)]
    pub bundle_jre: bool,

    /// JRE 路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jre_path: Option<String>,

    // ==================== 可选字段 - JVM 参数 ====================
    /// JVM 参数列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jvm_options: Option<Vec<String>>,

    /// 启动参数列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_args: Option<Vec<String>>,

    /// 是否启用调试
    #[serde(default)]
    pub enable_debug: bool,

    /// 调试端口
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_port: Option<u16>,

    /// 调试地址
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_host: Option<String>,

    // ==================== 可选字段 - 服务行为 ====================
    /// 启动模式：Automatic / Manual
    #[serde(default = "default_start_mode")]
    pub start_mode: String,

    /// 日志模式：rotate / reset / none
    #[serde(default = "default_log_mode")]
    pub log_mode: String,

    /// 失败恢复延迟（秒）
    #[serde(default = "default_failure_delay")]
    pub on_failure_delay: u32,

    // ==================== 可选字段 - 工作目录 ====================
    /// 工作目录
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,

    // ==================== 可选字段 - 打包选项 ====================
    /// 是否包含 JAR 包
    #[serde(default = "default_true")]
    pub include_jar: bool,

    /// 是否保存到历史
    #[serde(default)]
    pub save_to_history: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            service_id: String::new(),
            service_name: String::new(),
            jar_path: String::new(),
            description: None,
            java_executable: None,
            bundle_jre: false,
            jre_path: None,
            jvm_options: None,
            enable_debug: false,
            debug_host: None,
            debug_port: None,
            start_mode: default_start_mode(),
            log_mode: default_log_mode(),
            on_failure_delay: default_failure_delay(),
            working_directory: None,
            include_jar: true,
            save_to_history: false,
            executable: None,
            app_args: None,
        }
    }
}

fn default_start_mode() -> String {
    "Automatic".to_string()
}

fn default_log_mode() -> String {
    "rotate".to_string()
}

fn default_failure_delay() -> u32 {
    30
}

fn default_true() -> bool {
    true
}
