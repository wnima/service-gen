use common::models::service_config::ServiceConfig;
use common::utils::file_utils::pick_jar_file;
use iced::Task;
use iced::widget::{Row, button, checkbox, column, row, text, text_input};
use iced::{Element, Length};
use std::path::PathBuf;

/// 配置视图消息
#[derive(Debug, Clone)]
pub enum ConfigMessage {
    DescriptionChanged(String),
    UseCustomJavaToggled(bool),
    JavaExecutableChanged(String),
    BundleJreToggled(bool),
    JrePathChanged(String),
    JvmOptionsChanged(String),
    AppArgsChanged(String),
    EnableDebugToggled(bool),
    DebugPortChanged(String),
    StartModeChanged(String),
    LogModeChanged(String),
    FailureDelayChanged(String),
    UseWorkDirToggled(bool),
    WorkingDirectoryChanged(String),
    IncludeJarToggled(bool),
    ServiceIdChanged(String),
    ServiceNameChanged(String),
    JarPathChanged(String),
    BrowseJarPressed,
}

/// 启动模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartMode {
    Automatic,
    Manual,
}

/// 日志模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogMode {
    Rotate,
    Reset,
    None,
}

/// 配置视图
pub struct ConfigView {
    pub service_id: String,
    pub service_name: String,
    pub jar_path: String,
    pub description: String,
    pub use_custom_java: bool,
    pub java_executable: String,
    pub bundle_jre: bool,
    pub jre_path: String,
    pub jvm_options: String,
    pub app_args: String,
    pub enable_debug: bool,
    pub debug_port: String,
    pub start_mode: StartMode,
    pub log_mode: LogMode,
    pub failure_delay: String,
    pub use_work_dir: bool,
    pub working_directory: String,
    pub include_jar: bool,
}

impl ConfigView {
    pub fn new() -> Self {
        Self {
            service_id: String::new(),
            service_name: String::new(),
            jar_path: String::new(),
            description: String::new(),
            use_custom_java: false,
            java_executable: String::new(),
            bundle_jre: false,
            jre_path: String::new(),
            jvm_options: String::new(),
            app_args: String::new(),
            enable_debug: false,
            debug_port: String::new(),
            start_mode: StartMode::Automatic,
            log_mode: LogMode::Rotate,
            failure_delay: "30".to_string(),
            use_work_dir: false,
            working_directory: String::new(),
            include_jar: true,
        }
    }

    pub fn update(&mut self, message: ConfigMessage) -> Task<crate::app_view::Message> {
        match message {
            ConfigMessage::DescriptionChanged(value) => {
                self.description = value;
            }
            ConfigMessage::UseCustomJavaToggled(value) => {
                self.use_custom_java = value;
            }
            ConfigMessage::JavaExecutableChanged(value) => {
                self.java_executable = value;
            }
            ConfigMessage::BundleJreToggled(value) => {
                self.bundle_jre = value;
            }
            ConfigMessage::JrePathChanged(value) => {
                self.jre_path = value;
            }
            ConfigMessage::JvmOptionsChanged(value) => {
                self.jvm_options = value;
            }
            ConfigMessage::AppArgsChanged(value) => {
                self.app_args = value;
            }
            ConfigMessage::EnableDebugToggled(value) => {
                self.enable_debug = value;
            }
            ConfigMessage::DebugPortChanged(value) => {
                self.debug_port = value;
            }
            ConfigMessage::StartModeChanged(value) => {
                self.start_mode = if value == "Automatic" {
                    StartMode::Automatic
                } else {
                    StartMode::Manual
                };
            }
            ConfigMessage::LogModeChanged(value) => {
                self.log_mode = match value.as_str() {
                    "reset" => LogMode::Reset,
                    "none" => LogMode::None,
                    _ => LogMode::Rotate,
                };
            }
            ConfigMessage::FailureDelayChanged(value) => {
                self.failure_delay = value;
            }
            ConfigMessage::UseWorkDirToggled(value) => {
                self.use_work_dir = value;
            }
            ConfigMessage::WorkingDirectoryChanged(value) => {
                self.working_directory = value;
            }
            ConfigMessage::IncludeJarToggled(value) => {
                self.include_jar = value;
            }
            ConfigMessage::ServiceIdChanged(value) => {
                self.service_id = value;
            }
            ConfigMessage::ServiceNameChanged(value) => {
                self.service_name = value;
            }
            ConfigMessage::JarPathChanged(value) => {
                self.jar_path = value;
            }
            ConfigMessage::BrowseJarPressed => {
                // 打开文件对话框选择 JAR 文件
                return Task::perform(
                    async { pick_jar_file().map(|p: PathBuf| p.to_string_lossy().to_string()) },
                    |result| crate::app_view::Message::JarFileSelected(result),
                );
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, ConfigMessage> {
        let mut left_content = column![];
        let width_num = Length::Fill;
        let service_id_input = text_input("服务 ID (例如：my-service)", &self.service_id)
            .on_input(ConfigMessage::ServiceIdChanged)
            .padding(10)
            .width(width_num);

        let service_name_input = text_input(
            "服务名称 (例如：My Application Service)",
            &self.service_name,
        )
        .on_input(ConfigMessage::ServiceNameChanged)
        .padding(10)
        .width(width_num);

        let jar_path_input = text_input("JAR 包路径", &self.jar_path)
            .on_input(ConfigMessage::JarPathChanged)
            .padding(10);
        //.width(width_num - 55); // 留出空间给浏览按钮

        let browse_button = button(text("浏览"))
            .on_press(ConfigMessage::BrowseJarPressed)
            .padding(10);

        let jar_row = Row::new()
            .push(jar_path_input)
            .push(browse_button)
            .spacing(10)
            .width(width_num);

        // 服务描述
        let description_input = text_input("服务描述 (可选)", &self.description)
            .on_input(ConfigMessage::DescriptionChanged)
            .padding(10)
            .width(width_num);

        left_content = left_content.push(text("服务 ID").size(12));
        left_content = left_content.push(service_id_input);
        left_content = left_content.push(text("服务名称").size(12));
        left_content = left_content.push(service_name_input);
        left_content = left_content.push(text("JAR 包路径").size(12));
        left_content = left_content.push(jar_row);
        left_content = left_content.push(text("服务描述").size(12));
        left_content = left_content.push(description_input);

        let mut center_content = column![];

        center_content = center_content.push(text("Java 运行环境").size(12));

        // Java 运行环境
        let custom_java_checkbox = checkbox(self.use_custom_java)
            .label("使用自定义 Java 路径")
            .on_toggle(ConfigMessage::UseCustomJavaToggled);
        center_content = center_content.push(custom_java_checkbox);

        if self.use_custom_java {
            let java_input = text_input("Java 可执行文件路径", &self.java_executable)
                .on_input(ConfigMessage::JavaExecutableChanged)
                .padding(10)
                .width(width_num);
            center_content = center_content.push(java_input);
        }

        // 打包 JRE
        let bundle_jre_checkbox = checkbox(self.bundle_jre)
            .label("打包 JRE（将整个 JRE 目录包含到输出包）")
            .on_toggle(ConfigMessage::BundleJreToggled);
        center_content = center_content.push(bundle_jre_checkbox);

        if self.bundle_jre {
            let jre_input = text_input("JRE 路径", &self.jre_path)
                .on_input(ConfigMessage::JrePathChanged)
                .padding(10)
                .width(width_num);
            center_content = center_content.push(jre_input);
        }

        // JVM 参数
        let jvm_input = text_input("JVM 参数 (例如：-Xmx512m -Xms256m)", &self.jvm_options)
            .on_input(ConfigMessage::JvmOptionsChanged)
            .padding(10)
            .width(width_num);
        center_content = center_content.push(text("JVM 参数").size(12));
        center_content = center_content.push(jvm_input);

        // app参数
        let app_args_input = text_input("应用启动参数 (例如：--server.port=8080)", &self.app_args)
            .on_input(ConfigMessage::AppArgsChanged)
            .padding(10)
            .width(width_num);
        center_content = center_content.push(text("应用启动参数").size(12));
        center_content = center_content.push(app_args_input);

        // 远程调试
        let debug_checkbox = checkbox(self.enable_debug)
            .label("启用远程调试")
            .on_toggle(ConfigMessage::EnableDebugToggled);
        center_content = center_content.push(debug_checkbox);

        if self.enable_debug {
            let port_input = text_input("调试端口", &self.debug_port)
                .on_input(ConfigMessage::DebugPortChanged)
                .padding(10)
                .width(Length::Fixed(150.0));
            center_content = center_content.push(port_input);
        }

        // 启动模式
        center_content = center_content.push(text("启动模式").size(12));
        center_content = center_content.push(
            checkbox(matches!(self.start_mode, StartMode::Automatic))
                .label("Automatic (自动启动)")
                .on_toggle(|_| ConfigMessage::StartModeChanged("Automatic".to_string())),
        );
        center_content = center_content.push(
            checkbox(matches!(self.start_mode, StartMode::Manual))
                .label("Manual (手动启动)")
                .on_toggle(|_| ConfigMessage::StartModeChanged("Manual".to_string())),
        );

        let mut right_content = column![];
        // 日志模式
        right_content = right_content.push(text("日志模式").size(12));
        right_content = right_content.push(
            checkbox(matches!(self.log_mode, LogMode::Rotate))
                .label("rotate (自动滚动)")
                .on_toggle(|_| ConfigMessage::LogModeChanged("rotate".to_string())),
        );
        right_content = right_content.push(
            checkbox(matches!(self.log_mode, LogMode::Reset))
                .label("reset (覆盖旧日志)")
                .on_toggle(|_| ConfigMessage::LogModeChanged("reset".to_string())),
        );
        right_content = right_content.push(
            checkbox(matches!(self.log_mode, LogMode::None))
                .label("none (不记录日志)")
                .on_toggle(|_| ConfigMessage::LogModeChanged("none".to_string())),
        );

        // 失败恢复延迟
        let delay_input = text_input("失败恢复延迟（秒）", &self.failure_delay)
            .on_input(ConfigMessage::FailureDelayChanged)
            .padding(10)
            .width(Length::Fixed(150.0));
        right_content = right_content.push(text("失败恢复延迟（秒）").size(12));
        right_content = right_content.push(delay_input);

        // 工作目录
        let work_dir_checkbox = checkbox(self.use_work_dir)
            .label("使用自定义工作目录")
            .on_toggle(ConfigMessage::UseWorkDirToggled);
        right_content = right_content.push(work_dir_checkbox);

        if self.use_work_dir {
            let work_dir_input = text_input("工作目录", &self.working_directory)
                .on_input(ConfigMessage::WorkingDirectoryChanged)
                .padding(10)
                .width(width_num);
            right_content = right_content.push(work_dir_input);
        }

        // 打包选项
        let include_jar_checkbox = checkbox(self.include_jar)
            .label("包含 JAR 包到输出文件")
            .on_toggle(ConfigMessage::IncludeJarToggled);
        right_content = right_content.push(include_jar_checkbox);

        row![
            left_content.spacing(10).width(200),
            center_content.spacing(10).width(200),
            right_content.spacing(10).width(200),
        ]
        .spacing(20)
        .into()
    }

    /// 更新配置对象
    pub fn update_config(&self, config: &mut ServiceConfig) {
        config.service_id.clone_from(&self.service_id);
        config.service_name.clone_from(&self.service_name);
        config.jar_path.clone_from(&self.jar_path);

        if !self.description.is_empty() {
            config.description = Some(self.description.clone());
        }

        if self.use_custom_java && !self.java_executable.is_empty() {
            config.java_executable = Some(self.java_executable.clone());
        }

        config.bundle_jre = self.bundle_jre;
        if self.bundle_jre && !self.jre_path.is_empty() {
            config.jre_path = Some(self.jre_path.clone());
        }

        if !self.jvm_options.is_empty() {
            config.jvm_options = Some(
                self.jvm_options
                    .split_whitespace()
                    .map(String::from)
                    .collect(),
            );
        }

        config.enable_debug = self.enable_debug;
        if self.enable_debug {
            config.debug_port = self.debug_port.parse().ok();
        }

        config.start_mode = match self.start_mode {
            StartMode::Automatic => "Automatic".to_string(),
            StartMode::Manual => "Manual".to_string(),
        };

        config.log_mode = match self.log_mode {
            LogMode::Rotate => "rotate".to_string(),
            LogMode::Reset => "reset".to_string(),
            LogMode::None => "none".to_string(),
        };

        config.on_failure_delay = self.failure_delay.parse().unwrap_or(30);

        if self.use_work_dir && !self.working_directory.is_empty() {
            config.working_directory = Some(self.working_directory.clone());
        }

        config.include_jar = self.include_jar;
    }

    /// 从配置对象加载
    pub fn load_from_config(&mut self, config: &ServiceConfig) {
        self.service_id = config.service_id.clone();
        self.service_name = config.service_name.clone();
        self.jar_path = config.jar_path.clone();

        self.description = config.description.clone().unwrap_or_default();

        if let Some(ref java) = config.java_executable {
            self.use_custom_java = true;
            self.java_executable = java.clone();
        }

        self.bundle_jre = config.bundle_jre;
        if let Some(ref jre) = config.jre_path {
            self.jre_path = jre.clone();
        }

        if let Some(ref options) = config.jvm_options {
            self.jvm_options = options.join(" ");
        }

        self.enable_debug = config.enable_debug;
        if let Some(port) = config.debug_port {
            self.debug_port = port.to_string();
        }

        self.start_mode = if config.start_mode == "Automatic" {
            StartMode::Automatic
        } else {
            StartMode::Manual
        };

        self.log_mode = match config.log_mode.as_str() {
            "reset" => LogMode::Reset,
            "none" => LogMode::None,
            _ => LogMode::Rotate,
        };

        self.failure_delay = config.on_failure_delay.to_string();

        if let Some(ref work_dir) = config.working_directory {
            self.use_work_dir = true;
            self.working_directory = work_dir.clone();
        }

        self.include_jar = config.include_jar;
    }
}

impl Default for ConfigView {
    fn default() -> Self {
        Self::new()
    }
}
