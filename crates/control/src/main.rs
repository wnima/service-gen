#![windows_subsystem = "windows"]

use anyhow::Result;

mod gui;
//mod sc_utils;
mod service_utils;
mod service;

/// 服务模式和GUI模式的入口
fn main() -> Result<()> {

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "-service" {
        let _ = service::run_as_service();
    } else {
        let _ = gui::run_gui();
    }
    Ok(())
}
