#![windows_subsystem = "windows"]
mod app_view;
mod config_view;
mod package_export;

use tracing;

use crate::app_view::AppState;

pub fn main() -> iced::Result {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("debug".parse().unwrap()),
        )
        .init();

    tracing::info!("Starting Service Generator Desktop");

    AppState::application().run()
}
