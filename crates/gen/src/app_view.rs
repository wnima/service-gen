use crate::config_view::{ConfigMessage, ConfigView};
use crate::package_export::PackageExportService;
use common::models::service_config::ServiceConfig;
use common::utils::file_utils::{self, pick_directory};
use iced::widget::rule;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::window::{Icon, icon};
use iced::{Application, Element, Font, Length, Padding, Pixels, Program, Task, Theme, window};
use image::GenericImageView;
use std::path::PathBuf;
use tracing::error;
use tracing::info;

/// 应用状态
pub struct AppState {
    // 高级配置视图（包含基础配置）
    advanced_config: ConfigView,

    // UI 状态
    is_generating: bool,
    status_message: Option<String>,

    // 输出目录
    output_dir: Option<PathBuf>,

    // 配置对象
    config: ServiceConfig,
}

/// 应用消息
#[derive(Debug, Clone)]
pub enum Message {
    // 高级配置（包含基础配置）
    AdvancedConfig(ConfigMessage),

    // UI 控制
    GenerateButtonPressed,
    ResetButtonPressed,
    SaveConfigPressed,
    LoadConfigPressed,
    SelectOutputDirPressed,

    // 文件对话框结果
    JarFileSelected(Option<String>),
    ConfigFileSelected(Option<String>),
    ConfigSaved(Option<String>),
    OutputDirSelected(Option<String>),
    ConfigLoaded(Option<ServiceConfig>),

    // 异步结果
    PackageGenerated(Result<String, String>),
}

impl AppState {
    pub fn application() -> Application<impl Program<Message = Message, Theme = Theme>> {
        let window_settings = window::Settings {
            size: iced::Size::new(650.0, 420.0),
            //max_size: Some(iced::Size::new(700.0, 500.0)),
            //min_size: Some(iced::Size::new(700.0, 500.0)),
            icon: Self::load_window_icon(),
            //resizable: false,
            ..Default::default()
        };
        iced::application(AppState::new, AppState::update, AppState::view)
            .title(AppState::title)
            .default_font(Font::with_name("Microsoft YaHei"))
            .window(window_settings)
            .settings(iced::Settings {
                default_text_size: Pixels(12.0),
                ..Default::default()
            })
    }

    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            advanced_config: ConfigView::new(),
            is_generating: false,
            status_message: None,
            output_dir: None,
            config: ServiceConfig::default(),
        };
        (app, Task::none())
    }

    pub fn title(&self) -> String {
        "服务包装器生成".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AdvancedConfig(msg) => {
                return self.advanced_config.update(msg);
            }

            Message::GenerateButtonPressed => {
                if self.is_generating {
                    return Task::none();
                }

                // 更新配置对象
                self.advanced_config.update_config(&mut self.config);

                // 获取输出目录
                let output_dir = self
                    .output_dir
                    .clone()
                    .unwrap_or_else(|| std::env::current_dir().unwrap());

                info!("开始生成包装器：{}", self.config.service_id);

                self.is_generating = true;
                self.status_message = Some("正在生成包装器...".to_string());

                // 调用后端服务生成包
                let mut config = self.config.clone();
                return Task::perform(
                    async move {
                        PackageExportService::new()
                            .export_package(&mut config, &output_dir)
                            .await
                            .map(|p| p.to_string_lossy().to_string())
                            .map_err(|e| e.to_string())
                    },
                    Message::PackageGenerated,
                );
            }

            Message::JarFileSelected(file_path) => {
                if let Some(path) = file_path {
                    self.advanced_config.jar_path = path.clone();
                    info!("选择了 JAR 文件：{}", path);
                }
            }

            Message::ResetButtonPressed => {
                self.advanced_config = ConfigView::new();
                self.config = ServiceConfig::default();
                self.status_message = Some("表单已重置".to_string());
            }

            Message::PackageGenerated(result) => {
                self.is_generating = false;
                match result {
                    Ok(path) => {
                        self.status_message = Some(format!("包装器已生成：{}", path));
                        info!("包装器生成成功：{}", path);
                    }
                    Err(e) => {
                        self.status_message = Some(format!("生成失败：{}", e));
                        tracing::error!("包装器生成失败：{}", e);
                    }
                }
            }

            Message::SaveConfigPressed => {
                // 更新配置对象
                self.advanced_config.update_config(&mut self.config);

                return Task::perform(
                    async { file_utils::save_json_file().map(|p| p.to_string_lossy().to_string()) },
                    Message::ConfigSaved,
                );
            }

            Message::LoadConfigPressed => {
                return Task::perform(
                    async { file_utils::pick_json_file().map(|p| p.to_string_lossy().to_string()) },
                    Message::ConfigFileSelected,
                );
            }

            Message::SelectOutputDirPressed => {
                return Task::perform(
                    async { pick_directory().map(|p: PathBuf| p.to_string_lossy().to_string()) },
                    Message::OutputDirSelected,
                );
            }

            Message::ConfigSaved(path) => {
                if let Some(p) = path {
                    if let Err(e) = self.save_config_to_file(&p) {
                        self.status_message = Some(format!("保存配置失败：{}", e));
                    } else {
                        self.status_message = Some("配置已保存".to_string());
                    }
                }
            }

            Message::ConfigFileSelected(path) => {
                if let Some(p) = path {
                    return Task::perform(
                        async move { Self::load_config_from_file(&p).ok() },
                        Message::ConfigLoaded,
                    );
                }
            }

            Message::OutputDirSelected(path) => {
                if let Some(p) = path {
                    self.output_dir = Some(PathBuf::from(p.clone()));
                    self.status_message = Some(format!("输出目录已设置为：{}", p));
                }
            }

            Message::ConfigLoaded(config) => {
                if let Some(c) = config {
                    self.config = c;
                    self.advanced_config.load_from_config(&self.config);
                    self.status_message = Some("配置已加载".to_string());
                } else {
                    self.status_message = Some("加载配置失败".to_string());
                }
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        // 配置区域（包含基础和高级配置）
        let config_section = container(scrollable(
            self.advanced_config.view().map(Message::AdvancedConfig),
        ))
        .padding(0)
        .height(Length::Fill)
        .width(Length::Fill);

        // 操作按钮
        let generate_button = button(text(if self.is_generating {
            "生成中..."
        } else {
            "生  成"
        }).center())
        .on_press_maybe(if self.is_generating {
            None
        } else {
            Some(Message::GenerateButtonPressed)
        })
        .padding(10)
        .width(Length::Fill);

        let reset_button = button(text("重置表单").center())
            .on_press(Message::ResetButtonPressed)
            .padding(10)
            .width(Length::Fill);

        let save_button = button(text("保存配置").center())
            .on_press(Message::SaveConfigPressed)
            .padding(10)
            .width(Length::Fill);

        let load_button = button(text("加载配置").center())
            .on_press(Message::LoadConfigPressed)
            .padding(10)
            .width(Length::Fill);

        let select_output_button = button(text("选择输出目录").center())
            .on_press(Message::SelectOutputDirPressed)
            .padding(10)
            .width(Length::Fill);

        let button_row = row![
            generate_button,
            reset_button,
            save_button,
            load_button,
            select_output_button
        ]
        .spacing(10)
        .width(Length::Fill)
        .padding(0);

        // 状态信息
        let status_view = if let Some(ref msg) = self.status_message {
            container(text(msg).size(12)).padding(Padding {
                top: 0.0,
                bottom: 0.0,
                left: 10.0,
                right: 10.0,
            })
        } else {
            container(text(""))
        };

        let content = column![config_section, button_row].spacing(0).padding(0);

        container(column![
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10),
            container(rule::horizontal(1)).padding(0),
            status_view
        ])
        .into()
    }

    /// 保存配置到文件
    fn save_config_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(path, json)?;
        info!("配置已保存到：{}", path);
        Ok(())
    }

    /// 从文件加载配置
    fn load_config_from_file(path: &str) -> Result<ServiceConfig, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: ServiceConfig = serde_json::from_str(&content)?;
        info!("配置已加载从：{}", path);
        Ok(config)
    }

    // 从内嵌资源中加载窗口图标
    fn load_window_icon() -> Option<Icon> {
        // 使用include_bytes!宏内嵌图标数据，路径相对于src目录
        let icon_data: &'static [u8] = include_bytes!("../../../assets/icon.ico");
        if let Ok(image) = image::load_from_memory(icon_data) {
            let rgba = image.to_rgba8();
            let (width, height) = image.dimensions();
            Some(icon::from_rgba(rgba.into_raw(), width, height).unwrap())
        } else {
            error!("无法从内嵌资源加载图标");
            None
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            advanced_config: ConfigView::default(),
            is_generating: false,
            status_message: None,
            output_dir: None,
            config: ServiceConfig::default(),
        }
    }
}
